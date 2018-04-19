use mouscache::{MemoryCache, RedisCache};

#[derive(Cacheable, Clone, Debug)]
struct DataTestDerive {
    field1: u16,
    field2: String,
}

#[test]
fn memory_cache_test_derive() {
    let data = DataTestDerive {
        field1: 42,
        field2: String::from("Hello, World!"),
    };

    let mut cache = MemoryCache::new();

    let _ = cache.insert("test", data.clone());

    let data2: DataTestDerive = cache.get("test").unwrap();

    assert_eq!(data.field1, data2.field1);
    assert_eq!(data.field2, data2.field2);
}

#[test]
fn redis_cache_test_derive() {
    let data = DataTestDerive {
        field1: 42,
        field2: String::from("Hello, World!"),
    };

    println!("Initial data {:?}", data);

    if let Ok(mut cache) = RedisCache::new("localhost", None) {
        let _ = cache.insert("test", data.clone());

        println!("data inserted");

        let data2: DataTestDerive = cache.get("test").unwrap();

        println!("Data retrived {:?}", data);

        assert_eq!(data.field1, data2.field1);
        assert_eq!(data.field2, data2.field2);
    }
}