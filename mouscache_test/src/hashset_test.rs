use mouscache;

#[test]
fn memory_cache_test_hash_set() {
    let cache = mouscache::memory();

    let _ = cache.set_insert("test_group", "123321");

    assert!(cache.set_contains("test_group", "123321").unwrap());
}

#[test]
fn redis_cache_test_hash_set() {
    let cache = match mouscache::redis("localhost", Some("123456"), None) {
        Ok(c) => c,
        Err(e) => {
            println!("{:?}", e);
            return;
        }
    };

    let _ = cache.set_insert("test_group", "123321");

    assert!(cache.set_contains("test_group", "123321").unwrap());
}