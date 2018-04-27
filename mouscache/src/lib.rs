extern crate r2d2;
extern crate r2d2_redis;
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

#[cfg(feature = "hashset")]
trait HashSetAccess {
    fn set_insert<G: ToString, K: ToString>(&self, group_id: G, member: K) -> Result<()>;
    fn set_contains<G: ToString, K: ToString>(&self, group_id: G, member: K) -> Result<bool>;
    fn set_remove<G: ToString, K: ToString>(&self, group_id: G, member: K) -> Result<()>;
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

    #[cfg(feature = "hashset")]
    pub fn set_insert<G: ToString, K: ToString>(&self, group_id: G, member: K) -> Result<()> {
        match *self {
            Memory(ref c) => c.set_insert(group_id, member),
            Redis(ref c) => c.set_insert(group_id, member),
        }
    }

    #[cfg(feature = "hashset")]
    pub fn set_contains<G: ToString, K: ToString>(&self, group_id: G, member: K) -> Result<bool> {
        match *self {
            Memory(ref c) => c.set_contains(group_id, member),
            Redis(ref c) => c.set_contains(group_id, member),
        }
    }

    #[cfg(feature = "hashset")]
    pub fn set_remove<G: ToString, K: ToString>(&self, group_id: G, member: K) -> Result<()> {
        match *self {
            Memory(ref c) => c.set_remove(group_id, member),
            Redis(ref c) => c.set_remove(group_id, member),
        }
    }
}

unsafe impl Send for Cache {}
unsafe impl Sync for Cache {}

pub fn memory() -> Cache {
    Memory(MemoryCache::new())
}

pub fn redis(host: &str, password: Option<&str>) -> Result<Cache> {
    match RedisCache::new(host, password) {
        Ok(rc) => Ok(Redis(rc)),
        Err(e) => Err(e),
    }
}