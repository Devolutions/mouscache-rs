use std::net;
use std::mem::discriminant;
use std::collections::hash_map::HashMap;
use crate::Result;
use crate::CacheError;
use crate::Cacheable;
use crate::CacheAccess;
use crate::CacheFunc;
use crate::redis;
use redis::Commands;
use dns_lookup::lookup_host;
use crate::FromValue;

use r2d2::Pool;
use std::str::FromStr;

const DB_CONNECTION_TIMEOUT_MS: i64 = 5000;

mod r2d2_test {
    use crate::redis;
    use redis::{cmd, RedisError};
    use r2d2;
    use std::error;
    use std::fmt;

    /// A unified enum of errors returned by redis::Client
    #[derive(Debug)]
    pub enum Error {
        /// A redis::RedisError
        RedisCacheError(RedisError),
        Other(String),
    }

    impl fmt::Display for Error {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            match self {
                Error::RedisCacheError(e) => write!(f, "Redis cache error: {}", e),
                Error::Other(desc) => write!(f, "Other: {}", desc),
            }
        }
    }

    impl From<redis::RedisError> for Error {
        fn from(e: RedisError) -> Self {
            Error::RedisCacheError(e)
        }
    }

    impl error::Error for Error {}

    #[derive(Debug)]
    pub struct RedisConnectionManager {
        connection_info: redis::ConnectionInfo,
        password: Option<String>,
        db: Option<u16>,
    }

    impl RedisConnectionManager {
        pub fn new<T: redis::IntoConnectionInfo>(host: T, password: Option<&str>, db: Option<u16>)
                                                 -> Result<RedisConnectionManager, redis::RedisError> {
            Ok(RedisConnectionManager {
                connection_info: host.into_connection_info()?,
                password: password.map(|s| { s.to_string() }),
                db,
            })
        }
    }

    impl r2d2::ManageConnection for RedisConnectionManager {
        type Connection = redis::Connection;
        type Error = Error;

        fn connect(&self) -> Result<redis::Connection, Error> {
            let client = redis::Client::open(self.connection_info.clone())?;
            let conn = client.get_connection()?;

            if let Some(ref p) = self.password {
                if !cmd("AUTH").arg(p).query::<bool>(&conn)? {
                    return Err(Error::Other("Redis authentication failed: Bad password".to_string()));
                }
            }

            if let Some(db) = self.db {
                if !cmd("SELECT").arg(db).query::<bool>(&conn)? {
                    return Err(Error::Other(format!("Redis server refused to switch database: Bad index ({:?})", db)));
                }
            }

            Ok(conn)
        }

        fn is_valid(&self, conn: &mut redis::Connection) -> Result<(), Error> {
            Ok(redis::cmd("PING").query(conn)?)
        }

        fn has_broken(&self, _conn: &mut redis::Connection) -> bool {
            false
        }
    }
}


#[allow(dead_code)]
pub struct RedisCache {
    connection_pool: Pool<r2d2_test::RedisConnectionManager>,
}

impl Clone for RedisCache {
    fn clone(&self) -> Self {
        RedisCache {
            connection_pool: self.connection_pool.clone()
        }
    }
}

impl RedisCache {
    pub fn new(host: &str, password: Option<&str>, db: Option<u16>) -> Result<RedisCache> {
        let host_vec: Vec<&str> = host.split(":").collect();

        let ips: Vec<net::IpAddr> = match lookup_host(host_vec[0]) {
            Ok(hosts) => hosts,
            Err(e) => return Err(CacheError::Other(e.to_string())),
        };

        if let Some((_, ip_v4)) = ips.iter()
            .enumerate()
            .find(|&(_index, ip)| {
                discriminant(ip) == discriminant(&net::IpAddr::V4(net::Ipv4Addr::new(0, 0, 0, 0)))
            }) {
            let ip_host = if host_vec.len() > 1 {
                format!("{}:{}", ip_v4.to_string(), host_vec[1])
            } else {
                ip_v4.to_string()
            };

            let url = format!("redis://{}", ip_host);

            let manager = match r2d2_test::RedisConnectionManager::new(url.as_str(), password, db) {
                Ok(m) => m,
                Err(e) => return Err(CacheError::Other(e.to_string())),
            };

            let connection_pool = match Pool::builder()
                .max_size(15)
                .connection_timeout(::std::time::Duration::from_millis(DB_CONNECTION_TIMEOUT_MS as u64))
                .build(manager) {
                Ok(cp) => cp,
                Err(e) => return Err(CacheError::Other(e.to_string())),
            };

            return Ok(RedisCache {
                connection_pool,
            });
        }

        Err(CacheError::Other(format!("Could'n find any valid IP for host {} ", host)))
    }
}

impl CacheAccess for RedisCache {
    fn insert<K: ToString, O: Cacheable + Clone + 'static>(&self, key: K, obj: O) -> Result<()> {
        let exp = obj.expires_after();
        self.insert_with(key, obj, exp)
    }

    fn insert_with<K: ToString, O: Cacheable + Clone + 'static>(&self, key: K, obj: O, expires_after: Option<usize>) -> Result<()> {
        let connection = match self.connection_pool.get() {
            Ok(con) => con,
            Err(e) => return Err(CacheError::ConnectionError(e.to_string())),
        };

        let redis_key = redis_key_create::<K, O>(key);
        let data = obj.to_redis_obj();
        if let Some(ttl) = expires_after {
            redis_hash_set_multiple_with_expire(&connection, redis_key, &data, ttl)
        } else {
            redis_hash_set_multiple(&connection, redis_key, &data)
        }
    }

    fn get<K: ToString, O: Cacheable + 'static>(&self, key: K) -> Result<Option<O>> {
        let connection = match self.connection_pool.get() {
            Ok(con) => con,
            Err(e) => return Err(CacheError::ConnectionError(e.to_string())),
        };

        let redis_key = redis_key_create::<K, O>(key);
        if let Ok(val) = redis_hash_get_all(&connection, redis_key) {
            if let Ok(c) = O::from_redis_obj(val) {
                Ok(Some(c))
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }

    fn contains_key<K: ToString, O: Cacheable + Clone + 'static>(&self, key: K) -> Result<bool> {
        let connection = match self.connection_pool.get() {
            Ok(con) => con,
            Err(e) => return Err(CacheError::ConnectionError(e.to_string())),
        };

        let redis_key = redis_key_create::<K, O>(key);

        redis_key_exists(&connection, redis_key)
    }

    fn remove<K: ToString, O: Cacheable>(&self, key: K) -> Result<()> {
        let connection = match self.connection_pool.get() {
            Ok(con) => con,
            Err(e) => return Err(CacheError::ConnectionError(e.to_string())),
        };

        let redis_key = redis_key_create::<K, O>(key);
        redis_delete(&connection, redis_key)
    }
}

fn redis_key_create<K: ToString, O: Cacheable>(key: K) -> String {
    format!("{}:{}", O::model_name(), key.to_string())
}

fn redis_hash_set_multiple_with_expire<F: redis::ToRedisArgs, V: redis::ToRedisArgs>(con: &redis::Connection, key: String, v: &[(F, V)], ttl_sec: usize) -> Result<()> {
    redis_hash_set_multiple(con, key.clone(), v)?;
    con.expire(key, ttl_sec).map(|_: ::redis::Value| ()).map_err( |e| e.into())
}

fn redis_hash_set_multiple<F: redis::ToRedisArgs, V: redis::ToRedisArgs>(con: &redis::Connection, key: String, v: &[(F, V)]) -> Result<()> {
    con.hset_multiple::<String, F, V, ()>(key, v).map_err( |e| e.into())
}

fn redis_hash_get_all(con: &redis::Connection, key: String) -> Result<HashMap<String, String>> {
    con.hgetall::<String, HashMap<String, String>>(key).map_err( |e| e.into())
}

fn redis_delete(con: &redis::Connection, key: String) -> Result<()> {
    con.del::<String, ()>(key).map_err( |e| e.into())
}

fn redis_key_exists(con: &redis::Connection, key: String) -> Result<bool> {
    con.exists::<String, bool>(key).map_err( |e| e.into())
}

impl CacheFunc for RedisCache {
    fn hash_delete(&self, key: &str, fields: &[&str]) -> Result<bool> {
        let connection = match self.connection_pool.get() {
            Ok(con) => con,
            Err(e) => return Err(CacheError::ConnectionError(e.to_string())),
        };

        connection.hdel(key, fields).map_err(|e| e.into())
    }

    fn hash_exists(&self, key: &str, field: &str) -> Result<bool> {
        let connection = match self.connection_pool.get() {
            Ok(con) => con,
            Err(e) => return Err(CacheError::ConnectionError(e.to_string())),
        };
        connection.hexists(key, field).map_err(|e| e.into())
    }

    fn hash_get<T: FromStr>(&self, key: &str, field: &str) -> Result<Option<T>> {
        let connection = match self.connection_pool.get() {
            Ok(con) => con,
            Err(e) => return Err(CacheError::ConnectionError(e.to_string())),
        };

        let redis_val: ::redis::Value = connection.hget(key, field)?;
        if let ::redis::Value::Nil = redis_val {
            return Ok(None);
        }

        let val = String::from_redis_value(&redis_val)?;
        return T::from_str(val.as_ref()).map(|t| Some(t)).map_err(|_| CacheError::Other("An error occured while parsing a redis value".to_string()))
    }

    fn hash_get_all<T: Cacheable + Clone + 'static>(&self, key: &str) -> Result<Option<T>> {
        let connection = match self.connection_pool.get() {
            Ok(con) => con,
            Err(e) => return Err(CacheError::ConnectionError(e.to_string())),
        };
        let map: HashMap<String, String> = connection.hgetall(key)?;
        T::from_redis_obj(map).map(|t| Some(t))
    }

    fn hash_keys(&self, key: &str) -> Result<Vec<String>> {
        let connection = match self.connection_pool.get() {
            Ok(con) => con,
            Err(e) => return Err(CacheError::ConnectionError(e.to_string())),
        };
        connection.hkeys(key).map_err(|e| e.into())
    }

    fn hash_len(&self, key: &str) -> Result<usize> {
        let connection = match self.connection_pool.get() {
            Ok(con) => con,
            Err(e) => return Err(CacheError::ConnectionError(e.to_string())),
        };
        connection.hlen(key).map_err(|e| e.into())
    }

    fn hash_multiple_get(&self, _key: &str, _fields: &[&str]) -> Result<Vec<Option<String>>> {
        unimplemented!()
    }

    fn hash_multiple_set<V: ToString>(&self, key: &str, fv_pairs: &[(&str, V)]) -> Result<bool> {
        let connection = match self.connection_pool.get() {
            Ok(con) => con,
            Err(e) => return Err(CacheError::ConnectionError(e.to_string())),
        };
        let intermediate_vec = fv_pairs.iter().map(|&(ref s, ref v)| (s.to_string(), v.to_string())).collect::<Vec<(String, String)>>();
        connection.hset_multiple(key, &intermediate_vec).map_err(|e| e.into())
    }

    fn hash_set<V: ToString>(&self, key: &str, field: &str, value: V) -> Result<bool> {
        let connection = match self.connection_pool.get() {
            Ok(con) => con,
            Err(e) => return Err(CacheError::ConnectionError(e.to_string())),
        };
        connection.hset(key, field, value.to_string()).map_err(|e| e.into())
    }

    fn hash_set_all<T: Cacheable + Clone + 'static>(&self, key: &str, cacheable: T) -> Result<bool> {
        let connection = match self.connection_pool.get() {
            Ok(con) => con,
            Err(e) => return Err(CacheError::ConnectionError(e.to_string())),
        };
        let fv_pairs = cacheable.to_redis_obj();
        connection.hset_multiple(key, &fv_pairs).map_err(|e| e.into())
    }

    fn hash_set_if_not_exists<V: ToString>(&self, key: &str, field: &str, value: V) -> Result<bool> {
        let connection = match self.connection_pool.get() {
            Ok(con) => con,
            Err(e) => return Err(CacheError::ConnectionError(e.to_string())),
        };
        connection.hset_nx(key, field, value.to_string()).map_err(|e| e.into())
    }

    fn hash_values(&self, key: &str) -> Result<Vec<String>> {
        let connection = match self.connection_pool.get() {
            Ok(con) => con,
            Err(e) => return Err(CacheError::ConnectionError(e.to_string())),
        };
        connection.hvals(key).map_err(|e| e.into())
    }

    fn set_add<V: ToString>(&self, key: &str, members: &[V]) -> Result<bool> {
        let connection = match self.connection_pool.get() {
            Ok(con) => con,
            Err(e) => return Err(CacheError::ConnectionError(e.to_string())),
        };
        let string_members = members.iter().map(|m| m.to_string()).collect::<Vec<String>>();
        connection.sadd(key, string_members).map_err(|e| e.into())
    }

    fn set_card(&self, key: &str) -> Result<u64> {
        let connection = match self.connection_pool.get() {
            Ok(con) => con,
            Err(e) => return Err(CacheError::ConnectionError(e.to_string())),
        };
        connection.scard(key).map_err(|e| e.into())
    }

    fn set_diff(&self, keys: &[&str]) -> Result<Vec<String>> {
        let connection = match self.connection_pool.get() {
            Ok(con) => con,
            Err(e) => return Err(CacheError::ConnectionError(e.to_string())),
        };
        connection.sdiff(keys).map_err(|e| e.into())
    }

    fn set_diffstore(&self, diff_name: &str, keys: &[&str]) -> Result<u64> {
        let connection = match self.connection_pool.get() {
            Ok(con) => con,
            Err(e) => return Err(CacheError::ConnectionError(e.to_string())),
        };
        ::redis::cmd("SDIFFSTORE").arg(diff_name).arg(keys).query(&*connection).map_err(|e| e.into())
    }

    fn set_inter(&self, keys: &[&str]) -> Result<Vec<String>> {
        let connection = match self.connection_pool.get() {
            Ok(con) => con,
            Err(e) => return Err(CacheError::ConnectionError(e.to_string())),
        };
        connection.sinter(keys).map_err(|e| e.into())
    }

    fn set_interstore(&self, inter_name: &str, keys: &[&str]) -> Result<u64> {
        let connection = match self.connection_pool.get() {
            Ok(con) => con,
            Err(e) => return Err(CacheError::ConnectionError(e.to_string())),
        };
        ::redis::cmd("SINTERSTORE").arg(inter_name).arg(keys).query(&*connection).map_err(|e| e.into())
    }

    fn set_ismember<V: ToString>(&self, key: &str, member: V) -> Result<bool> {
        let connection = match self.connection_pool.get() {
            Ok(con) => con,
            Err(e) => return Err(CacheError::ConnectionError(e.to_string())),
        };
        connection.sismember(key, member.to_string()).map_err(|e|e.into())
    }

    fn set_members(&self, key: &str) -> Result<Vec<String>> {
        let connection = match self.connection_pool.get() {
            Ok(con) => con,
            Err(e) => return Err(CacheError::ConnectionError(e.to_string())),
        };
        connection.smembers(key).map_err(|e|e.into())
    }

    fn set_move<V: ToString>(&self, key1: &str, key2: &str, member: V) -> Result<bool> {
        let connection = match self.connection_pool.get() {
            Ok(con) => con,
            Err(e) => return Err(CacheError::ConnectionError(e.to_string())),
        };
        connection.smove(key1, key2, member.to_string()).map_err(|e|e.into())
    }

    fn set_rem<V: ToString>(&self, key: &str, member: V) -> Result<bool> {
        let connection = match self.connection_pool.get() {
            Ok(con) => con,
            Err(e) => return Err(CacheError::ConnectionError(e.to_string())),
        };
        connection.srem(key, member.to_string()).map_err(|e|e.into())
    }

    fn set_union(&self, keys: &[&str]) -> Result<Vec<String>> {
        let connection = match self.connection_pool.get() {
            Ok(con) => con,
            Err(e) => return Err(CacheError::ConnectionError(e.to_string())),
        };
        connection.sunion(keys).map_err(|e| e.into())
    }

    fn set_unionstore(&self, union_name: &str, keys: &[&str]) -> Result<u64> {
        let connection = match self.connection_pool.get() {
            Ok(con) => con,
            Err(e) => return Err(CacheError::ConnectionError(e.to_string())),
        };
        ::redis::cmd("SUNIONSTORE").arg(union_name).arg(keys).query(&*connection).map_err(|e| e.into())
    }
}