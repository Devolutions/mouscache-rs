use std;
use mouscache::*;

#[test]
fn test_memory_hash_functions() {
    let cache = memory();

//    fn hash_delete(&self, key: &str, fields: &[&str]) -> Result<bool>;
//    fn hash_exists(&self, key: &str, field: &str) -> Result<bool>;
//    fn hash_get<T: FromStr>(&self, key: &str, field: &str) -> Result<Option<T>>;
//    fn hash_get_all<T: Cacheable + Clone + 'static>(&self, key: &str) -> Result<Option<T>>;
//    fn hash_incr_by(&self, key: &str, field: &str, incr: i64) -> Result<i64>;
//    fn hash_incr_by_float(&self, key: &str, field: &str, fincr: f64) -> Result<f64>;
//    fn hash_keys(&self, key: &str) -> Result<Vec<String>>;
//    fn hash_len(&self, key: &str) -> Result<usize>;
//    fn hash_multiple_get(&self, key: &str, fields: &[&str]) -> Result<Vec<Option<String>>>;
//    fn hash_multiple_set<V: ToString>(&self, key: &str, fv_pairs: &[(&str, V)] ) -> Result<bool>;
//    fn hash_set<V: ToString>(&self, key: &str, field: &str, value: V) -> Result<bool>;
//    fn hash_set_all<T: Cacheable + Clone + 'static>(&self, key: &str, cacheable: T) -> Result<bool>;
//    fn hash_set_if_not_exists<V: ToString>(&self, key: &str, field: &str, value: V) -> Result<bool>;
//    fn hash_str_len(&self, key: &str, field: &str) -> Result<u64>;
//    fn hash_values(&self, key: &str) -> Result<Vec<String>>;
}