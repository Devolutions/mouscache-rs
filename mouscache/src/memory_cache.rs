use std::time::{Instant, Duration};
use std::collections::hash_map::HashMap;
use Result;
use Cache;
use Cache::Memory;
use Cacheable;
use CacheAccess;
use std::sync::RwLock;
use std::sync::Arc;

struct Expiration {
    insertion_time: Instant,
    ttl: Duration,
}

impl Expiration {
    pub fn new(ttl: usize) -> Self {
        Expiration {
            insertion_time: Instant::now(),
            ttl: Duration::from_secs(ttl as u64)
        }
    }

    pub fn is_expired(&self) -> bool {
        let time_since_insertion = Instant::now().duration_since(self.insertion_time);
        time_since_insertion >= self.ttl
    }
}

type MemCacheable = (Box<Cacheable>, Option<Expiration>);

struct Inner {
    pub cache: RwLock<HashMap<String, MemCacheable>>
}

impl Inner {
    pub fn new() -> Self {
        Inner {
            cache: RwLock::new(HashMap::new())
        }
    }
}

pub struct MemoryCache {
    inner: Arc<Inner>
}

impl Clone for MemoryCache {
    fn clone(&self) -> Self {
        MemoryCache {
            inner: self.inner.clone(),
        }
    }
}

impl MemoryCache {
    pub fn new() -> Cache {
        Memory(MemoryCache {
            inner: Arc::new(Inner::new())
        })
    }
}

impl CacheAccess for MemoryCache {
    fn insert<K: ToString, O: Cacheable + Clone + 'static>(&mut self, key: K, obj: O) -> Result<()> {
        let tkey = gen_key::<K, O>(key);

        let exp = obj.expires_after().map(|ttl| {Expiration::new(ttl)});

        self.inner.cache.write().unwrap().insert(tkey, (Box::new(obj), exp));
        Ok(())
    }

    fn get<K: ToString, O: Cacheable + Clone + 'static>(&mut self, key: K) -> Result<Option<O>> {
        let tkey = gen_key::<K, O>(key);

        let mut delete_entry = false;

        {
            let cache = self.inner.cache.read().unwrap();
            if let Some(&(ref obj, ref exp)) = cache.get(&tkey) {
                if let &Some(ref exp) = exp {
                    if exp.is_expired() {
                        delete_entry = true;
                    }
                }

                if !delete_entry {
                    let struct_obj: O = match obj.as_any().downcast_ref::<O>() {
                        Some(struct_obj) => struct_obj.clone(),
                        None => panic!("Invalid type in mouscache")
                    };

                    return Ok(Some(struct_obj));
                }
            }
        }

        if delete_entry {
            let mut cache = self.inner.cache.write().unwrap();
            cache.remove(&tkey);
        }

        Ok(None)
    }

    fn remove<K: ToString, O: Cacheable>(&mut self, key: K) -> Result<()> {
        let tkey = gen_key::<K, O>(key);
        self.inner.cache.write().unwrap().remove(&tkey);
        Ok(())
    }
}

fn gen_key<K: ToString, O: Cacheable>(key: K) -> String {
    let mut new_key = String::from(O::model_name());
    new_key.push_str(":");
    new_key.push_str(key.to_string().as_str());
    new_key
}