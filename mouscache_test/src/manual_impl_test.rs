use std::any::Any;
use mouscache;
use mouscache::{Cacheable, CacheError, Result};
use std::collections::hash_map::HashMap;

#[derive(Clone, Debug)]
struct DataTest {
    field1: u16,
    field2: String,
}

impl Cacheable for DataTest {
    fn model_name() -> &'static str where Self: Sized {
        "DataTest"
    }

    fn to_redis_obj(&self) -> Vec<(String, String)> {
        let mut temp_vec = Vec::new();
        temp_vec.push((String::from("field1"), self.field1.to_string()));
        temp_vec.push((String::from("field2"), self.field2.clone()));
        temp_vec
    }

    fn from_redis_obj(map: HashMap<String, String>) -> Result<Self> where Self: Sized {
        if map.len() > 0 {
            let field1 : u16 = map["field1"].parse().unwrap();
            let field2 = map["field2"].clone();

            Ok(DataTest {
                field1,
                field2,
            })
        } else {
            Err(CacheError::Other(String::new()))
        }
    }

    fn as_any(&self) -> &Any {
        self
    }

    fn expires_after(&self) -> Option<usize> {
        None
    }
}

#[test]
fn memory_cache_test() {
    let data = DataTest {
        field1: 42,
        field2: String::from("Hello, World!"),
    };

    let cache = mouscache::memory();

    let _ = cache.insert("test", data.clone());

    let data2: DataTest = cache.get("test").unwrap().unwrap();

    assert_eq!(data.field1, data2.field1);
    assert_eq!(data.field2, data2.field2);
}

#[test]
fn redis_cache_test() {
    let data = DataTest {
        field1: 42,
        field2: String::from("Hello, World!"),
    };

    if let Ok(cache) = mouscache::redis("localhost", None) {
        let _ = cache.insert("test", data.clone());

        let data2: DataTest = cache.get("test").unwrap().unwrap();

        assert_eq!(data.field1, data2.field1);
        assert_eq!(data.field2, data2.field2);
    }
}