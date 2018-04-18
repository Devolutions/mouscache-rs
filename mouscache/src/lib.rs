extern crate redis;
extern crate dns_lookup;

mod memory_cache;
mod redis_cache;

use std::any::Any;
use std::collections::hash_map::HashMap;

pub use memory_cache::MemoryCache;
pub use redis_cache::RedisCache;

pub enum CacheError {
    InsertionError(String),
    DeletionError(String),
    AccessError(String),
    Other(String)
}

pub type Result<T> = std::result::Result<T, CacheError>;

pub trait Cacheable {
    fn model_name() -> &'static str where Self: Sized;
    fn to_redis_obj(&self) -> Vec<(String, String)>;
    fn from_redis_obj(obj: HashMap<String, String>) -> Result<Self> where Self: Sized;
    fn as_any(&self) -> &Any;
}

pub trait CacheAccess {
    fn insert<K: ToString, O: Cacheable + Clone + 'static>(&mut self, key: K, obj: O) -> Result<()>;
    fn get<K: ToString, O: Cacheable + Clone + 'static>(&mut self, key: K) -> Option<O>;
    fn remove<K: ToString, O: Cacheable>(&mut self, key: K) -> Result<()>;
}

pub struct Cache<T: CacheAccess>(T);

impl<T: CacheAccess> Cache<T> {
    pub fn new(cache_obj: T) -> Self {
        Cache(cache_obj)
    }
}

impl<T: CacheAccess> CacheAccess for Cache<T> {
    fn insert<K: ToString, O: Cacheable + Clone + 'static>(&mut self, key: K, obj: O) -> Result<()> {
        self.0.insert(key, obj)
    }

    fn get<K: ToString, O: Cacheable + Clone + 'static>(&mut self, key: K) -> Option<O> {
        self.0.get::<K, O>(key)
    }

    fn remove<K: ToString, O: Cacheable>(&mut self, key: K) -> Result<()> {
        self.0.remove::<K, O>(key)
    }
}