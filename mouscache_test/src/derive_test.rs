use std;
use mouscache;

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

    let mut cache = mouscache::memory();

    let _ = cache.insert("test", data.clone());

    let data2: DataTestDerive = cache.get("test").unwrap().unwrap();

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

    if let Ok(mut cache) = mouscache::redis("localhost", None) {
        let _ = cache.insert("test", data.clone());

        println!("data inserted");

        let data2: DataTestDerive = cache.get("test").unwrap().unwrap();

        println!("Data retrived {:?}", data);

        assert_eq!(data.field1, data2.field1);
        assert_eq!(data.field2, data2.field2);
    }
}

#[derive(Cacheable, Clone, Debug)]
#[cache(expires="1")]
struct DataTestExpires {
    field_temp: String,
}

#[test]
fn redis_cache_test_derive_expires() {
    let data = DataTestExpires {
        field_temp: String::from("Hello, World!"),
    };

    println!("Initial data {:?}", data);

    if let Ok(mut cache) = mouscache::redis("localhost", None) {
        let _ = cache.insert("exp", data.clone());

        println!("data inserted");

        if let Ok(Some(data2)) = cache.get::<&str, DataTestExpires>("exp") {
            assert_eq!(data.field_temp, data2.field_temp);
        }

        std::thread::sleep(std::time::Duration::from_secs(1));

        if let Ok(Some(_)) = cache.get::<&str, DataTestExpires>("exp") {
            assert!(false);
        }
    }
}


#[test]
fn redis_cache_test_db() {
    let data = DataTestDerive {
        field1: 42,
        field2: String::from("Hello, World!"),
        field_uuid: String::from("a2f5c0d6-6191-4172-81c9-c3531df19407"),
    };

    println!("Initial data {:?}", data);

    if let Ok(mut cache) = mouscache::redis("localhost:6379/3", None) {
        let _ = cache.insert("test", data.clone());

        println!("data inserted");

        let data2: DataTestDerive = cache.get("test").unwrap().unwrap();

        println!("Data retrived {:?}", data);

        assert_eq!(data.field1, data2.field1);
        assert_eq!(data.field2, data2.field2);
    }
}