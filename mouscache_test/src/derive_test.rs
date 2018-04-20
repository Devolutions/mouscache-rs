use mouscache::{MemoryCache, RedisCache};

#[derive(Cacheable, Clone, Debug)]
struct DataTestDerive {
    field1: u16,
    field2: String,
    field_uuid: String
}

#[test]
fn memory_cache_test_derive() {
    let data = DataTestDerive {
        field1: 42,
        field2: String::from("Hello, World!"),
        field_uuid: String::from("a2f5c0d6-6191-4172-81c9-c3531df19407"),
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
        field_uuid: String::from("a2f5c0d6-6191-4172-81c9-c3531df19407"),
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