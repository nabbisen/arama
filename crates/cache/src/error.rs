use crate::CacheError;

pub mod cache_error;

pub type Result<T> = std::result::Result<T, CacheError>;
