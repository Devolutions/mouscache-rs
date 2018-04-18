# mouscache-rs
A small lib to manipulate object with redis or an in-memory cache

## How to
```rust
use mouscache::{CacheAccess, MemoryCache, RedisCache};

#[derive(Cacheable, Clone, Debug)]
struct YourData {
    field1: u16,
    field2: String,
}

fn main() {
    let data = YourData {
        field1: 42,
        field2: String::from("Hello, World!"),
    };

    println!("Initial data {:?}", data);

    if let Ok(mut cache) = RedisCache::new("localhost", None) {
        let _ = cache.insert("test", data.clone());

        println!("data inserted");

        let data2: YourData = cache.get("test").unwrap();

        println!("Data retrived {:?}", data);

        assert_eq!(data.field1, data2.field1);
        assert_eq!(data.field2, data2.field2);
    }
}
```

##TODO
- [x] Add support for `struct` with named field
- [ ] Add support for unnamed field
- [ ] Add support for `enum`
- [ ] Add support for `union`
