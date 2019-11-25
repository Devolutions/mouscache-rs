use std;
use redis::RedisError;


#[derive(Debug)]
pub enum CacheError {
    RedisCacheError(RedisError),
    InsertionError(String),
    DeletionError(String),
    AccessError(String),
    ConnectionError(String),
    Other(String),
}

use crate::CacheError::*;

impl std::fmt::Display for CacheError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result<> {
        match *self {
            RedisCacheError(ref e) => e.fmt(f),
            InsertionError(ref desc) => write!(f, "Insertion error: {}", desc),
            DeletionError(ref desc) => write!(f, "Deletion error: {}", desc),
            AccessError(ref desc) => write!(f, "Access error: {}", desc),
            ConnectionError(ref desc) => write!(f, "Connection error: {}", desc),
            Other(ref desc) => write!(f, "Unknown error: {}", desc),
        }
    }
}

impl std::error::Error for CacheError {
    fn description(&self) -> &str {
        match *self {
            RedisCacheError(ref e) => e.description(),
            InsertionError(_) => "Insertion error",
            DeletionError(_) => "Deletion error",
            AccessError(_) => "Access error",
            ConnectionError(_) => "Connection error",
            Other(_) => "Unknown error",
        }
    }
}

impl From<RedisError> for CacheError {
    fn from(e: RedisError) -> Self {
        CacheError::RedisCacheError(e)
    }
}