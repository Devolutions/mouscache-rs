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
        unimplemented!()
    }

    fn hash_exists(&self, key: &str, field: &str) -> Result<bool> {
        unimplemented!()
    }

    fn hash_get<T: FromStr>(&self, key: &str, field: &str) -> Result<T> {
        unimplemented!()
    }

    fn hash_get_all<T: Cacheable>(&self, key: &str) -> Result<T> {
        unimplemented!()
    }

    fn hash_incr_by(&self, key: &str, field: &str, incr: i64) -> Result<i64> {
        unimplemented!()
    }

    fn hash_incr_by_float(&self, key: &str, field: &str, fincr: f64) -> Result<f64> {
        unimplemented!()
    }

    fn hash_keys(&self, key: &str) -> Result<Vec<String>> {
        unimplemented!()
    }

    fn hash_len(&self, key: &str) -> Result<usize> {
        unimplemented!()
    }

    fn hash_multiple_get(&self, key: &str, fields: &[&str]) -> Result<Vec<Option<String>>> {
        unimplemented!()
    }

    fn hash_multiple_set<V: ToString>(&self, key: &str, fv_pairs: &[(&str, V)]) -> Result<bool> {
        unimplemented!()
    }

    fn hash_set<V: ToString>(&self, key: &str, field: &str, value: V) -> Result<bool> {
        unimplemented!()
    }

    fn hash_set_all<T: Cacheable>(&self, key: &str, cacheable: T) -> Result<bool> {
        unimplemented!()
    }

    fn hash_set_if_not_exists<V: ToString>(&self, key: &str, field: &str, value: V) -> Result<bool> {
        unimplemented!()
    }

    fn hash_str_len(&self, key: &str, field: &str) -> Result<u64> {
        unimplemented!()
    }

    fn hash_values(&self, key: &str) -> Result<Vec<String>> {
        unimplemented!()
    }

    fn set_add<V: ToString>(&self, key: &str, members: &[V]) -> Result<bool> {
        unimplemented!()
    }

    fn set_card(&self, key: &str) -> Result<u64> {
        unimplemented!()
    }

    fn set_diff(&self, keys: &[&str]) -> Result<Vec<String>> {
        unimplemented!()
    }

    fn set_diffstore(&self, diff_name: &str, keys: &[&str]) -> Result<u64> {
        unimplemented!()
    }

    fn set_inter(&self, keys: &[&str]) -> Result<Vec<String>> {
        unimplemented!()
    }

    fn set_interstore(&self, inter_name: &str, keys: &[&str]) -> Result<u64> {
        unimplemented!()
    }

    fn set_ismember<V: ToString>(&self, key: &str, member: V) -> Result<bool> {
        unimplemented!()
    }

    fn set_members(&self, key: &str) -> Result<Vec<String>> {
        unimplemented!()
    }

    fn set_move<V: ToString>(&self, key1: &str, key2: &str, member: V) -> Result<bool> {
        unimplemented!()
    }

    fn set_rem<V: ToString>(&self, key: &str, member: V) -> Result<bool> {
        unimplemented!()
    }

    fn set_union(&self, keys: &[&str]) -> Result<Vec<String>> {
        unimplemented!()
    }

    fn set_unionstore(&self, union_name: &str, keys: &[&str]) -> Result<u64> {
        unimplemented!()
    }
}