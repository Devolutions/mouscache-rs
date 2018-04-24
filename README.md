# mouscache-rs
[![Mouscache doc badge](https://docs.rs/mouscache/badge.svg)](https://docs.rs/mouscache/)

A small lib to manipulate object with redis or an in-memory cache

## Basic Usage
```rust
use mouscache::{MemoryCache, RedisCache};

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

    if let Ok(mut cache) = RedisCache::new("localhost", None) {
        let _ = cache.insert("test", data.clone());

        let data2: YourData = cache.get("test").unwrap();

        assert_eq!(data.field1, data2.field1);
        assert_eq!(data.field2, data2.field2);
    }
}
```

## Customizing What's Being Cached
Mouscache now support 2 custom attribute to customize entry :

### `expires` Attribute
Specifies a duration in sec after which the entry is invalid
```rust
use mouscache::{MemoryCache, RedisCache};

#[derive(Cacheable, Clone, Debug)]
#[cache(expires="10")] // each entry of type YouCustomDataType will be valid 10 sec.
struct YouCustomDataType {
    yourPrecious_field: String
}
```

### `rename` Attribute
Specifies the name which will be used to insert the entry
```rust
use mouscache::{MemoryCache, RedisCache};

#[derive(Cacheable, Clone, Debug)]
#[cache(rename="ThisNameIsCooler")] // each entry of type YouCustomDataType will be inserted with ThisNameIsCooler
struct YouCustomDataType {
    yourPrecious_field: String
}
```

##TODO
- [x] Add support for `struct` with named field
- [x] Add Data Attribute
- [ ] Add support for unnamed field
- [ ] Add support for `enum`
