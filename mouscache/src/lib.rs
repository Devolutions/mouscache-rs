extern crate r2d2;
extern crate redis;
extern crate dns_lookup;

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

trait CacheFunc {
    // Redis-like HashSet related functions
    fn hash_delete(&self, key: &str, fields: &[&str]) -> Result<bool>;
    fn hash_exists(&self, key: &str, field: &str) -> Result<bool>;
    fn hash_get<T: FromStr>(&self, key: &str, field: &str) -> Result<T>;
    fn hash_get_all<T: Cacheable>(&self, key: &str) -> Result<T>;
    fn hash_incr_by(&self, key: &str, field: &str, incr: i64) -> Result<i64>;
    fn hash_incr_by_float(&self, key: &str, field: &str, fincr: f64) -> Result<f64>;
    fn hash_keys(&self, key: &str) -> Result<Vec<String>>;
    fn hash_len(&self, key: &str) -> Result<usize>;
    fn hash_multiple_get(&self, key: &str, fields: &[&str]) -> Result<Vec<Option<String>>>;
    fn hash_multiple_set<V: ToString>(&self, key: &str, fv_pairs: &[(&str, V)] ) -> Result<bool>;
    fn hash_set<V: ToString>(&self, key: &str, field: &str, value: V) -> Result<bool>;
    fn hash_set_all<T: Cacheable>(&self, key: &str, cacheable: T) -> Result<bool>;
    fn hash_set_if_not_exists<V: ToString>(&self, key: &str, field: &str, value: V) -> Result<bool>;
    fn hash_str_len(&self, key: &str, field: &str) -> Result<u64>;
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