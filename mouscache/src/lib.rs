extern crate r2d2;
extern crate redis;
extern crate dns_lookup;
extern crate parking_lot;

pub use redis::Value;
pub use redis::FromRedisValue as FromValue;
pub use redis::ToRedisArgs as ToArgs;

mod error;
mod memory_cache;
mod redis_cache;

use std::{any::Any, collections::HashMap};

use memory_cache::MemoryCache;
use redis_cache::RedisCache;
pub use error::CacheError;

pub type Result<T> = std::result::Result<T, CacheError>;

pub trait Cacheable {
    fn model_name() -> &'static str where Self: Sized;
    fn to_redis_obj(&self) -> Vec<(String, String)>;
    fn from_redis_obj(obj: HashMap<String, String>) -> Result<Self> where Self: Sized;
    fn expires_after(&self) -> Option<usize>;
    fn as_any(&self) -> &Any;
}

use std::str::FromStr;

pub trait CacheFunc {
    // Redis-like HashSet related functions
    fn hash_delete(&self, key: &str, fields: &[&str]) -> Result<bool>;
    fn hash_exists(&self, key: &str, field: &str) -> Result<bool>;
    fn hash_get<T: FromStr>(&self, key: &str, field: &str) -> Result<Option<T>>;
    fn hash_get_all<T: Cacheable + Clone + 'static>(&self, key: &str) -> Result<Option<T>>;
    fn hash_keys(&self, key: &str) -> Result<Vec<String>>;
    fn hash_len(&self, key: &str) -> Result<usize>;
    fn hash_multiple_get(&self, key: &str, fields: &[&str]) -> Result<Vec<Option<String>>>;
    fn hash_multiple_set<V: ToString>(&self, key: &str, fv_pairs: &[(&str, V)] ) -> Result<bool>;
    fn hash_set<V: ToString>(&self, key: &str, field: &str, value: V) -> Result<bool>;
    fn hash_set_all<T: Cacheable + Clone + 'static>(&self, key: &str, cacheable: T) -> Result<bool>;
    fn hash_set_if_not_exists<V: ToString>(&self, key: &str, field: &str, value: V) -> Result<bool>;
    fn hash_values(&self, key: &str) -> Result<Vec<String>>;
    // Redis-like Set related functions
    fn set_add<V: ToString>(&self, key: &str, members: &[V]) -> Result<bool>;
    fn set_card(&self, key: &str) -> Result<u64>;
    fn set_diff(&self, keys: &[&str]) -> Result<Vec<String>>;
    fn set_diffstore(&self, diff_name: &str, keys: &[&str]) -> Result<u64>;
    fn set_inter(&self, keys: &[&str]) -> Result<Vec<String>>;
    fn set_interstore(&self, inter_name: &str, keys: &[&str]) -> Result<u64>;
    fn set_ismember<V: ToString>(&self, key: &str, member: V) -> Result<bool>;
    fn set_members(&self, key: &str) -> Result<Vec<String>>;
    fn set_move<V: ToString>(&self, key1: &str, key2: &str, member: V) -> Result<bool>;
    fn set_rem<V: ToString>(&self, key: &str, member: V) -> Result<bool>;
    fn set_union(&self, keys: &[&str]) -> Result<Vec<String>>;
    fn set_unionstore(&self, union_name: &str, keys: &[&str]) -> Result<u64>;

}

trait CacheAccess {
    fn insert<K: ToString, O: Cacheable + Clone + 'static>(&self, key: K, obj: O) -> Result<()>;
    fn insert_with<K: ToString, O: Cacheable + Clone + 'static>(&self, key: K, obj: O, expires_after: Option<usize>) -> Result<()>;
    fn get<K: ToString, O: Cacheable + Clone + 'static>(&self, key: K) -> Result<Option<O>>;
    fn contains_key<K: ToString, O: Cacheable + Clone + 'static>(&self, key: K) -> Result<bool>;
    fn remove<K: ToString, O: Cacheable>(&self, key: K) -> Result<()>;
}

pub enum Cache {
    Memory(MemoryCache),
    Redis(RedisCache),
}

impl Clone for Cache {
    fn clone(&self) -> Self {
        match *self {
            Memory(ref c) => Memory(c.clone()),
            Redis(ref c) => Redis(c.clone()),
        }
    }
}

use Cache::*;

impl Cache {
    pub fn insert<K: ToString, O: Cacheable + Clone + 'static>(&self, key: K, obj: O) -> Result<()> {
        match *self {
            Memory(ref c) => c.insert(key, obj),
            Redis(ref c) => c.insert(key, obj),
        }
    }

    pub fn insert_with<K: ToString, O: Cacheable + Clone + 'static>(&self, key: K, obj: O, expires_after: Option<usize>) -> Result<()> {
        match *self {
            Memory(ref c) => c.insert_with(key, obj, expires_after),
            Redis(ref c) => c.insert_with(key, obj, expires_after),
        }
    }

    pub fn get<K: ToString, O: Cacheable + Clone + 'static>(&self, key: K) -> Result<Option<O>> {
        match *self {
            Memory(ref c) => c.get::<K, O>(key),
            Redis(ref c) => c.get::<K, O>(key),
        }
    }

    pub fn remove<K: ToString, O: Cacheable>(&self, key: K) -> Result<()> {
        match *self {
            Memory(ref c) => c.remove::<K, O>(key),
            Redis(ref c) => c.remove::<K, O>(key),
        }
    }
}

impl CacheFunc for Cache {
    fn hash_delete(&self, key: &str, fields: &[&str]) -> Result<bool> {
        match *self {
            Memory(ref m) => m.hash_delete(key, fields),
            Redis(ref r) => r.hash_delete(key, fields),
        }
    }

    fn hash_exists(&self, key: &str, field: &str) -> Result<bool> {
        match *self {
            Memory(ref m) => m.hash_exists(key, field),
            Redis(ref r) => r.hash_exists(key, field),
        }
    }

    fn hash_get<T: FromStr>(&self, key: &str, field: &str) -> Result<Option<T>> {
        match *self {
            Memory(ref m) => m.hash_get(key, field),
            Redis(ref r) => r.hash_get(key, field),
        }
    }

    fn hash_get_all<T: Cacheable + Clone + 'static>(&self, key: &str) -> Result<Option<T>> {
        match *self {
            Memory(ref m) => m.hash_get_all(key),
            Redis(ref r) => r.hash_get_all(key),
        }
    }

    fn hash_keys(&self, key: &str) -> Result<Vec<String>> {
        match *self {
            Memory(ref m) => m.hash_keys(key),
            Redis(ref r) => r.hash_keys(key),
        }
    }

    fn hash_len(&self, key: &str) -> Result<usize> {
        match *self {
            Memory(ref m) => m.hash_len(key),
            Redis(ref r) => r.hash_len(key),
        }
    }

    fn hash_multiple_get(&self, key: &str, fields: &[&str]) -> Result<Vec<Option<String>>> {
        match *self {
            Memory(ref m) => m.hash_multiple_get(key, fields),
            Redis(ref r) => r.hash_multiple_get(key, fields),
        }
    }

    fn hash_multiple_set<V: ToString>(&self, key: &str, fv_pairs: &[(&str, V)]) -> Result<bool> {
        match *self {
            Memory(ref m) => m.hash_multiple_set(key, fv_pairs),
            Redis(ref r) => r.hash_multiple_set(key, fv_pairs),
        }
    }

    fn hash_set<V: ToString>(&self, key: &str, field: &str, value: V) -> Result<bool> {
        match *self {
            Memory(ref m) => m.hash_set(key, field, value),
            Redis(ref r) => r.hash_set(key, field, value),
        }
    }

    fn hash_set_all<T: Cacheable + Clone + 'static>(&self, key: &str, cacheable: T) -> Result<bool> {
        match *self {
            Memory(ref m) => m.hash_set_all(key, cacheable),
            Redis(ref r) => r.hash_set_all(key, cacheable),
        }
    }

    fn hash_set_if_not_exists<V: ToString>(&self, key: &str, field: &str, value: V) -> Result<bool> {
        match *self {
            Memory(ref m) => m.hash_set_if_not_exists(key, field, value),
            Redis(ref r) => r.hash_set_if_not_exists(key, field, value),
        }
    }

    fn hash_values(&self, key: &str) -> Result<Vec<String>> {
        match *self {
            Memory(ref m) => m.hash_values(key),
            Redis(ref r) => r.hash_values(key),
        }
    }

    fn set_add<V: ToString>(&self, key: &str, members: &[V]) -> Result<bool> {
        match *self {
            Memory(ref m) => m.set_add(key, members),
            Redis(ref r) => r.set_add(key, members),
        }
    }

    fn set_card(&self, key: &str) -> Result<u64> {
        match *self {
            Memory(ref m) => m.set_card(key),
            Redis(ref r) => r.set_card(key),
        }
    }

    fn set_diff(&self, keys: &[&str]) -> Result<Vec<String>> {
        match *self {
            Memory(ref m) => m.set_diff(keys),
            Redis(ref r) => r.set_diff(keys),
        }
    }

    fn set_diffstore(&self, diff_name: &str, keys: &[&str]) -> Result<u64> {
        match *self {
            Memory(ref m) => m.set_diffstore(diff_name, keys),
            Redis(ref r) => r.set_diffstore(diff_name, keys),
        }
    }

    fn set_inter(&self, keys: &[&str]) -> Result<Vec<String>> {
        match *self {
            Memory(ref m) => m.set_inter(keys),
            Redis(ref r) => r.set_inter(keys),
        }
    }

    fn set_interstore(&self, inter_name: &str, keys: &[&str]) -> Result<u64> {
        match *self {
            Memory(ref m) => m.set_interstore(inter_name, keys),
            Redis(ref r) => r.set_interstore(inter_name, keys),
        }
    }

    fn set_ismember<V: ToString>(&self, key: &str, member: V) -> Result<bool> {
        match *self {
            Memory(ref m) => m.set_ismember(key, member),
            Redis(ref r) => r.set_ismember(key, member),
        }
    }

    fn set_members(&self, key: &str) -> Result<Vec<String>> {
        match *self {
            Memory(ref m) => m.set_members(key),
            Redis(ref r) => r.set_members(key),
        }
    }

    fn set_move<V: ToString>(&self, key1: &str, key2: &str, member: V) -> Result<bool> {
        match *self {
            Memory(ref m) => m.set_move(key1, key2, member),
            Redis(ref r) => r.set_move(key1, key2, member),
        }
    }

    fn set_rem<V: ToString>(&self, key: &str, member: V) -> Result<bool> {
        match *self {
            Memory(ref m) => m.set_rem(key, member),
            Redis(ref r) => r.set_rem(key, member),
        }
    }

    fn set_union(&self, keys: &[&str]) -> Result<Vec<String>> {
        match *self {
            Memory(ref m) => m.set_union(keys),
            Redis(ref r) => r.set_union(keys),
        }
    }

    fn set_unionstore(&self, union_name: &str, keys: &[&str]) -> Result<u64> {
        match *self {
            Memory(ref m) => m.set_unionstore(union_name, keys),
            Redis(ref r) => r.set_unionstore(union_name, keys),
        }
    }
}

unsafe impl Send for Cache {}
unsafe impl Sync for Cache {}

pub fn memory() -> Cache {
    Memory(MemoryCache::new())
}

pub fn redis(host: &str, password: Option<&str>, db: Option<u16>) -> Result<Cache> {
    match RedisCache::new(host, password, db) {
        Ok(rc) => Ok(Redis(rc)),
        Err(e) => Err(e),
    }
}