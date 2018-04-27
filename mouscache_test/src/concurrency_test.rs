use mouscache::{MemoryCache, RedisCache};

#[derive(Cacheable, Clone, Debug)]
struct ConcurrentData {
    field1: u16,
    field2: u32,
}

#[test]
fn memory_cache_concurrency_test() {
    use std::thread;

    let data = ConcurrentData {
        field1: 42,
        field2: 123456789
    };

    let mut cache = MemoryCache::new();

    let _ = cache.insert("test", data.clone());

    let mut handle_vec = Vec::new();

    for _i in 1..10 {
        let assert_data = data.clone();
        let mut cache_clone = cache.clone();

        let h = thread::spawn(move || {
            let rdata: ConcurrentData = cache_clone.get("test").unwrap().unwrap();

            assert_eq!(assert_data.field1, rdata.field1);
            assert_eq!(assert_data.field2, rdata.field2);
        });

        handle_vec.push(h);
    }

    for handle in handle_vec {
        let _ = handle.join();
        println!("thread joined");
    }
}

#[test]
fn redis_cache_concurrency_test() {
    use std::thread;

    let data = ConcurrentData {
        field1: 42,
        field2: 123456789
    };

    if let Ok(mut cache) = RedisCache::new("localhost", None) {
        let _ = cache.insert("test", data.clone());

        let mut handle_vec = Vec::new();

        for _i in 1..10 {
            let assert_data = data.clone();
            let mut cache_clone = cache.clone();

            let h = thread::spawn(move || {
                let rdata: ConcurrentData = cache_clone.get("test").unwrap().unwrap();

                assert_eq!(assert_data.field1, rdata.field1);
                assert_eq!(assert_data.field2, rdata.field2);
            });

            handle_vec.push(h);
        }

        for handle in handle_vec {
            let _ = handle.join();
            println!("thread joined");
        }
    }
}