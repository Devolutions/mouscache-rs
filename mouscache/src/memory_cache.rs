use std::collections::hash_map::HashMap;
use Result;
use Cache;
use Cacheable;
use CacheAccess;

pub struct MemoryCache {
    cache: HashMap<String, HashMap<String, Box<Cacheable>>>
}

impl Default for MemoryCache {
    fn default() -> Self {
        MemoryCache {
            cache: HashMap::new(),
        }
    }
}

impl MemoryCache {
    pub fn new() -> Cache<Self> {
        Cache::new(MemoryCache::default())
    }

    fn get_type_cache<O: Cacheable>(&mut self) -> &HashMap<String, Box<Cacheable>> {
        let model_name = O::model_name().to_string();
        if !self.cache.contains_key(&model_name) {
            let c: HashMap<String, Box<Cacheable>> = HashMap::new();
            self.cache.insert(model_name.clone(), c);
        }

        return &self.cache.get(&model_name).unwrap();
    }

    fn get_type_cache_mut<O: Cacheable>(&mut self) -> &mut HashMap<String, Box<Cacheable>> {
        let model_name = O::model_name().to_string();
        if !self.cache.contains_key(&model_name) {
            let c: HashMap<String, Box<Cacheable>> = HashMap::new();
            self.cache.insert(model_name.clone(), c);
        }

        return self.cache.get_mut(&model_name).unwrap();
    }
}

impl CacheAccess for MemoryCache {
    fn insert<K: ToString, O: Cacheable + Clone + 'static>(&mut self, key: K, obj: O) -> Result<()> {
        let c = self.get_type_cache_mut::<O>();
        c.insert(key.to_string(), Box::new(obj));
        Ok(())
    }

    fn get<K: ToString, O: Cacheable + Clone + 'static>(&mut self, key: K) -> Option<O> {
        let c = self.get_type_cache::<O>();

        if let Some(obj) = c.get(&key.to_string()) {
            let struct_obj: O = match obj.as_any().downcast_ref::<O>() {
                Some(struct_obj) => struct_obj.clone(),
                None => panic!("Invalid type in mouscache")
            };

            return Some(struct_obj);
        }

        None
    }

    fn remove<K: ToString, O: Cacheable>(&mut self, key: K) -> Result<()> {
        let c = self.get_type_cache_mut::<O>();
        c.remove(&key.to_string());
        Ok(())
    }
}