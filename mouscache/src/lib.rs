extern crate r2d2;
extern crate r2d2_redis;
extern crate redis;
extern crate dns_lookup;

mod error;
mod memory_cache;
mod redis_cache;

use std::{any::Any, collections::HashMap};

pub use memory_cache::MemoryCache;
pub use redis_cache::RedisCache;
pub use error::CacheError;

pub type Result<T> = std::result::Result<T, CacheError>;

pub trait Cacheable {
    fn model_name() -> &'static str where Self: Sized;
    fn to_redis_obj(&self) -> Vec<(String, String)>;
    fn from_redis_obj(obj: HashMap<String, String>) -> Result<Self> where Self: Sized;
    fn expires_after(&self) -> Option<usize>;
    fn as_any(&self) -> &Any;
}

pub trait CacheAccess {
    fn insert<K: ToString, O: Cacheable + Clone + 'static>(&mut self, key: K, obj: O) -> Result<()>;
    fn get<K: ToString, O: Cacheable + Clone + 'static>(&mut self, key: K) -> Result<Option<O>>;
    fn remove<K: ToString, O: Cacheable>(&mut self, key: K) -> Result<()>;
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
    pub fn insert<K: ToString, O: Cacheable + Clone + 'static>(&mut self, key: K, obj: O) -> Result<()> {
        match *self {
            Memory(ref mut c) => c.insert(key, obj),
            Redis(ref mut c) => c.insert(key, obj),
        }
    }

    pub fn get<K: ToString, O: Cacheable + Clone + 'static>(&mut self, key: K) -> Result<Option<O>> {
        match *self {
            Memory(ref mut c) => c.get::<K, O>(key),
            Redis(ref mut c) => c.get::<K, O>(key),
        }
    }

    pub fn remove<K: ToString, O: Cacheable>(&mut self, key: K) -> Result<()> {
        match *self {
            Memory(ref mut c) => c.remove::<K, O>(key),
            Redis(ref mut c) => c.remove::<K, O>(key),
        }
    }
}

unsafe impl Send for Cache {}
unsafe impl Sync for Cache {}