use std::time::{Instant, Duration};
use std::collections::hash_map::HashMap;
use std::collections::hash_set::HashSet;
use Result;
use Cacheable;
use CacheAccess;
use CacheFunc;
use parking_lot::RwLock;
use std::sync::Arc;
use std::str::FromStr;

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
    pub obj_cache: RwLock<HashMap<String, MemCacheable>>,
    pub hashsets: RwLock<HashMap<String, RwLock<HashMap<String, String>>>>,
    pub sets: RwLock<HashMap<String, RwLock<HashSet<String>>>>,
}

impl Inner {
    pub fn new() -> Self {
        Inner {
            obj_cache: RwLock::new(HashMap::new()),
            hashsets: RwLock::new(HashMap::new()),
            sets: RwLock::new(HashMap::new()),
        }
    }
    
    pub fn ensure_hash_exists(&self, key: &str) -> Result<()> {
        if let Some(_) = self.hashsets.read().get(key) {
            return Ok(());
        } else { 
            if let None = self.hashsets.write().insert(key.to_string(), RwLock::new(HashMap::new())) {
                return Err(::CacheError::Other("Unable to insert a new hashmap".to_string()));
            }
        }
        Ok(())
    }

    pub fn ensure_set_exists(&self, key: &str) -> Result<()> {
        if let Some(_) = self.sets.read().get(key) {
            return Ok(());
        } else {
            if let None = self.sets.write().insert(key.to_string(), RwLock::new(HashSet::new())) {
                return Err(::CacheError::Other("Unable to insert a new hashset".to_string()));
            }
        }
        Ok(())
    }

    pub fn set_exists(&self, key: &str) -> bool {
        self.sets.read().get(key).is_some()
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
    pub fn new() -> MemoryCache {
        MemoryCache {
            inner: Arc::new(Inner::new())
        }
    }
}

impl CacheAccess for MemoryCache {
    fn insert<K: ToString, O: Cacheable + Clone + 'static>(&self, key: K, obj: O) -> Result<()> {
        let tkey = gen_key::<K, O>(key);

        let exp = obj.expires_after().map(|ttl| {Expiration::new(ttl)});

        self.inner.obj_cache.write().insert(tkey, (Box::new(obj), exp));
        Ok(())
    }

    fn get<K: ToString, O: Cacheable + Clone + 'static>(&self, key: K) -> Result<Option<O>> {
        let tkey = gen_key::<K, O>(key);

        let mut delete_entry = false;

        {
            let cache = self.inner.obj_cache.read();
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
            let mut cache = self.inner.obj_cache.write();
            cache.remove(&tkey);
        }

        Ok(None)
    }

    fn contains_key<K: ToString, O: Cacheable + Clone + 'static>(&self, key: K) -> Result<bool> {
        let cache = self.inner.obj_cache.read();
        let tkey = gen_key::<K, O>(key);
        Ok(cache.contains_key(&tkey))
    }

    fn remove<K: ToString, O: Cacheable>(&self, key: K) -> Result<()> {
        let tkey = gen_key::<K, O>(key);
        self.inner.obj_cache.write().remove(&tkey);
        Ok(())
    }
}

fn gen_key<K: ToString, O: Cacheable>(key: K) -> String {
    format!("{}:{}", O::model_name(), key.to_string())
}

impl CacheFunc for MemoryCache {
    fn hash_delete(&self, key: &str, fields: &[&str]) -> Result<bool> {
        let map = self.inner.hashsets.read();
        if let Some(hash) = map.get(key) {
            for f in fields {
                hash.write().remove(&f.to_string());
            }
        }
        Ok(true)
    }

    fn hash_exists(&self, key: &str, field: &str) -> Result<bool> {
        let map = self.inner.hashsets.read();
        if let Some(hash) = map.get(key) {
            Ok(hash.read().contains_key(field))
        } else {
            Ok(false)
        }
    }

    fn hash_get<T: FromStr>(&self, key: &str, field: &str) -> Result<Option<T>> {
        let map = self.inner.hashsets.read();
        if let Some(hash) = map.get(key) {
            if let Some(val) = hash.read().get(field) {
                return T::from_str(val).map(|t| Some(t)).map_err(|_| ::CacheError::Other("Unable to parse value into disired type".to_string()))
            }
        }
        Ok(None)
    }

    fn hash_get_all<T: Cacheable + Clone + 'static>(&self, key: &str) -> Result<Option<T>> {
        self.get::<&str, T>(key)
    }

    fn hash_incr_by(&self, key: &str, field: &str, incr: i64) -> Result<i64> {
        unimplemented!()
    }

    fn hash_incr_by_float(&self, key: &str, field: &str, fincr: f64) -> Result<f64> {
        unimplemented!()
    }

    fn hash_keys(&self, key: &str) -> Result<Vec<String>> {
        let map = self.inner.hashsets.read();
        if let Some(hash) = map.get(key) {
            let res = hash.read().keys().map(|k| k.clone()).collect();
            return Ok(res);
        }
        Ok(vec!())
    }

    fn hash_len(&self, key: &str) -> Result<usize> {
        let map = self.inner.hashsets.read();
        if let Some(hash) = map.get(key) {
            return Ok(hash.read().len());
        }
        Ok(0)
    }

    fn hash_multiple_get(&self, key: &str, fields: &[&str]) -> Result<Vec<Option<String>>> {
        let mut vec = Vec::new();
        let map = self.inner.hashsets.read();
        if let Some(hash) = map.get(key) {
            let reader = hash.read();
            for f in fields {
                vec.push(reader.get(f.clone()).map(|s| s.clone()));
            }
        }

        Ok(vec)
    }

    fn hash_multiple_set<V: ToString>(&self, key: &str, fv_pairs: &[(&str, V)]) -> Result<bool> {
        self.inner.ensure_hash_exists(key)?;
        let map = self.inner.hashsets.read();
        if let Some(hash) = map.get(key) {
            let mut writer = hash.write();
            for pair in fv_pairs {
                writer.insert(pair.0.to_string(), pair.1.to_string());
            }
            Ok(true)
        } else {
            Err(::CacheError::Other("Unable to retrive hash from key".to_string()))
        }
    }

    fn hash_set<V: ToString>(&self, key: &str, field: &str, value: V) -> Result<bool> {
        self.inner.ensure_hash_exists(key)?;
        let map = self.inner.hashsets.read();
        if let Some(hash) = map.get(key) {
            hash.write().insert(field.to_string(), value.to_string());
            Ok(true)
        } else {
            Err(::CacheError::Other("Unable to retrive hash from key".to_string()))
        }
    }

    fn hash_set_all<T: Cacheable + Clone + 'static>(&self, key: &str, cacheable: T) -> Result<bool> {
        self.insert(key, cacheable).map(|_| true)
    }

    fn hash_set_if_not_exists<V: ToString>(&self, key: &str, field: &str, value: V) -> Result<bool> {
        self.inner.ensure_hash_exists(key)?;
        let map = self.inner.hashsets.read();
        if let Some(hash) = map.get(key) {
            {
                if hash.read().contains_key(field) {
                    return Ok(false);
                }
            }
            {
                hash.write().insert(field.to_string(), value.to_string());
                Ok(true)
            }
        } else {
            Err(::CacheError::Other("Unable to retrive hash from key".to_string()))
        }
    }

    fn hash_str_len(&self, key: &str, field: &str) -> Result<u64> {
        unimplemented!()
    }

    fn hash_values(&self, key: &str) -> Result<Vec<String>> {
        let map = self.inner.hashsets.read();
        let vec = if let Some(hash) = map.get(key) {
            hash.read().values().map(|s| s.clone()).collect()
        } else {
            Vec::new()
        };

        Ok(vec)
    }

    fn set_add<V: ToString>(&self, key: &str, members: &[V]) -> Result<bool> {
        self.inner.ensure_set_exists(key)?;
        let sets = self.inner.sets.read();
        if let Some(set) = sets.get(key) {
            let mut writer = set.write();
            for m in members {
                writer.insert(m.to_string());
            }
            Ok(true)
        } else {
            Err(::CacheError::Other("Unable to retrive set from key".to_string()))
        }
    }

    fn set_card(&self, key: &str) -> Result<u64> {
        let sets = self.inner.sets.read();
        if let Some(set) = sets.get(key) {
            return Ok(set.read().len() as u64);
        }
        Ok(0)
    }

    fn set_diff(&self, keys: &[&str]) -> Result<Vec<String>> {
        let mut set_vec = Vec::new();
        for key in keys {
            if self.inner.set_exists(key) {
               set_vec.push(key.clone())
            }
        }
        unimplemented!()
    }

    fn set_diffstore(&self, diff_name: &str, keys: &[&str]) -> Result<u64> {
        let mut set_vec = Vec::new();
        for key in keys {
            if self.inner.set_exists(key) {
                set_vec.push(key.clone())
            }
        }
        self.inner.ensure_set_exists(diff_name)?;
        unimplemented!()
    }

    fn set_inter(&self, keys: &[&str]) -> Result<Vec<String>> {
        let mut set_vec = Vec::new();
        for key in keys {
            if self.inner.set_exists(key) {
                set_vec.push(key.clone())
            }
        }
        unimplemented!()
    }

    fn set_interstore(&self, inter_name: &str, keys: &[&str]) -> Result<u64> {
        let mut set_vec = Vec::new();
        for key in keys {
            if self.inner.set_exists(key) {
                set_vec.push(key.clone())
            }
        }
        self.inner.ensure_set_exists(inter_name)?;
        unimplemented!()
    }

    fn set_ismember<V: ToString>(&self, key: &str, member: V) -> Result<bool> {
        self.inner.ensure_set_exists(key)?;
        unimplemented!()
    }

    fn set_members(&self, key: &str) -> Result<Vec<String>> {
        self.inner.ensure_set_exists(key)?;
        unimplemented!()
    }

    fn set_move<V: ToString>(&self, key1: &str, key2: &str, member: V) -> Result<bool> {
        self.inner.ensure_set_exists(key1)?;
        self.inner.ensure_set_exists(key2)?;
        unimplemented!()
    }

    fn set_rem<V: ToString>(&self, key: &str, member: V) -> Result<bool> {
        self.inner.ensure_set_exists(key)?;
        unimplemented!()
    }

    fn set_union(&self, keys: &[&str]) -> Result<Vec<String>> {
        let mut set_vec = Vec::new();
        for key in keys {
            if self.inner.set_exists(key) {
                set_vec.push(key.clone())
            }
        }
        unimplemented!()
    }

    fn set_unionstore(&self, union_name: &str, keys: &[&str]) -> Result<u64> {
        let mut set_vec = Vec::new();
        for key in keys {
            if self.inner.set_exists(key) {
                set_vec.push(key.clone())
            }
        }
        self.inner.ensure_set_exists(union_name)?;
        unimplemented!()
    }
}