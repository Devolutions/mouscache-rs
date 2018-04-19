use std::net;
use std::collections::hash_map::HashMap;
use Cache;
use Cache::Redis;
use Result;
use CacheError;
use Cacheable;
use CacheAccess;
use redis;
use redis::Commands;
use dns_lookup::lookup_host;

#[allow(dead_code)]
pub struct RedisCache {
    client: redis::Client,
    connection: redis::Connection,
}

impl RedisCache {
    pub fn new(host: &str, password: Option<&str>) -> Result<Cache> {
        let host_vec: Vec<&str> = host.split(":").collect();

        let ips: Vec<net::IpAddr> = match lookup_host(host_vec[0]) {
            Ok(hosts) => hosts,
            Err(e) => return Err(CacheError::Other(e.to_string())),
        };

        let ip_host = if host_vec.len() > 1 {
            format!("{}:{}", ips[0].to_string(), host_vec[1])
        } else {
            ips[0].to_string()
        };

        let url = match password {
            Some(p) => format!("redis://:{}@{}", p, ip_host),
            None => format!("redis://{}", ip_host),
        };

        let client = match redis::Client::open(url.as_str()) {
            Ok(c) => c,
            Err(e) => return Err(CacheError::Other(e.to_string())),
        };

        let connection = match client.get_connection() {
            Ok(c) => c,
            Err(e) => return Err(CacheError::Other(e.to_string())),
        };

        Ok(Redis(RedisCache {
            client,
            connection
        }))



    }
}

impl CacheAccess for RedisCache {
    fn insert<K: ToString, O: Cacheable + 'static>(&mut self, key: K, obj: O) -> Result<()> {
        let redis_key = redis_key_create::<K, O>(key);
        let data = obj.to_redis_obj();
        redis_hash_set_multiple(&self.connection, redis_key, &data)
    }

    fn get<K: ToString, O: Cacheable + 'static>(&mut self, key: K) -> Option<O> {
        let redis_key = redis_key_create::<K, O>(key);
        if let Ok(val) = redis_hash_get_all(&self.connection, redis_key) {
            if let Ok(c) = O::from_redis_obj(val) {
                Some(c)
            } else {
                None
            }
        } else {
            None
        }
    }

    fn remove<K: ToString, O: Cacheable>(&mut self, key: K) -> Result<()> {
        let redis_key = redis_key_create::<K, O>(key);
        redis_delete(&self.connection, redis_key)
    }
}

fn redis_key_create<K: ToString, O: Cacheable>(key: K) -> String {
    let mut redis_key = String::from(O::model_name());
    redis_key.push_str(":");
    redis_key.push_str(key.to_string().as_str());
    redis_key
}

fn redis_hash_set_multiple<F: redis::ToRedisArgs, V: redis::ToRedisArgs>(con: &redis::Connection, key: String, v: &[(F, V)]) -> Result<()> {
    match con.hset_multiple::<String, F, V, ()>(key, v) {
        Ok(_) => Ok(()),
        Err(_) => Err(CacheError::InsertionError(String::new())),
    }
}

fn redis_hash_get_all(con: &redis::Connection, key: String) -> Result<HashMap<String, String>> {
    match con.hgetall::<String, HashMap<String, String>>(key) {
        Ok(v) => Ok(v),
        Err(e) => Err(CacheError::Other(e.to_string())),
    }
}

fn redis_delete(con: &redis::Connection, key: String) -> Result<()> {
    match con.del::<String, ()>(key) {
        Ok(_) => Ok(()),
        Err(_) => Err(CacheError::DeletionError(String::new())),
    }
}