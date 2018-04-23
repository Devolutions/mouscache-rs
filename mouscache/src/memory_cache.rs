use std::time::{Instant, Duration};
use std::collections::hash_map::HashMap;
use Result;
use Cache;
use Cache::Memory;
use Cacheable;
use CacheAccess;

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

pub struct MemoryCache {
    cache: HashMap<String, HashMap<String, (Box<Cacheable>, Option<Expiration>)>>
}

impl Default for MemoryCache {
    fn default() -> Self {
        MemoryCache {
            cache: HashMap::new(),
        }
    }
}

impl MemoryCache {
    pub fn new() -> Cache {
        Memory(MemoryCache::default())
    }

    fn _get_type_cache<O: Cacheable>(&mut self) -> &HashMap<String, (Box<Cacheable>, Option<Expiration>)> {
        let model_name = O::model_name().to_string();
        if !self.cache.contains_key(&model_name) {
            let c: HashMap<String, (Box<Cacheable>, Option<Expiration>)> = HashMap::new();
            self.cache.insert(model_name.clone(), c);
        }

        return &self.cache.get(&model_name).unwrap();
    }

    fn get_type_cache_mut<O: Cacheable>(&mut self) -> &mut HashMap<String, (Box<Cacheable>, Option<Expiration>)> {
        let model_name = O::model_name().to_string();
        if !self.cache.contains_key(&model_name) {
            let c: HashMap<String, (Box<Cacheable>, Option<Expiration>)> = HashMap::new();
            self.cache.insert(model_name.clone(), c);
        }

        return self.cache.get_mut(&model_name).unwrap();
    }
}

impl CacheAccess for MemoryCache {
    fn insert<K: ToString, O: Cacheable + Clone + 'static>(&mut self, key: K, obj: O) -> Result<()> {
        let c = self.get_type_cache_mut::<O>();

        let exp = obj.expires_after().map(|ttl| {Expiration::new(ttl)});

        c.insert(key.to_string(), (Box::new(obj), exp));
        Ok(())
    }

    fn get<K: ToString, O: Cacheable + Clone + 'static>(&mut self, key: K) -> Option<O> {
        let c = self.get_type_cache_mut::<O>();

        let mut delete_entry = false;

        if let Some(&(ref obj, ref exp)) = c.get(&key.to_string()) {
            if let &Some(ref exp) = exp {
                if exp.is_expired() {
                    delete_entry = true;
                }
            } else {
                let struct_obj: O = match obj.as_any().downcast_ref::<O>() {
                    Some(struct_obj) => struct_obj.clone(),
                    None => panic!("Invalid type in mouscache")
                };

                return Some(struct_obj);
            }
        }

        if delete_entry {
            c.remove(&key.to_string());
        }

        None
    }

    fn remove<K: ToString, O: Cacheable>(&mut self, key: K) -> Result<()> {
        let c = self.get_type_cache_mut::<O>();
        c.remove(&key.to_string());
        Ok(())
    }
}