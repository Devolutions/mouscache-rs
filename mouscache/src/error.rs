use std;

#[derive(Debug)]
pub enum CacheError {
    InsertionError(String),
    DeletionError(String),
    AccessError(String),
    Other(String),
}

use CacheError::*;

impl std::fmt::Display for CacheError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result<> {
        match *self {
            InsertionError(ref desc) => write!(f, "Insertion error: {}", desc),
            DeletionError(ref desc) => write!(f, "Deletion error: {}", desc),
            AccessError(ref desc) => write!(f, "Access error: {}", desc),
            Other(ref desc) => write!(f, "Unknown error: {}", desc),
        }
    }
}

impl std::error::Error for CacheError {
    fn description(&self) -> &str {
        match *self {
            InsertionError(_) => "Insertion error",
            DeletionError(_) => "Deletion error",
            AccessError(_) => "Access error",
            Other(_) => "Unknown error",
        }
    }
}