pub mod cache_error;

use cache_error::CacheError;

pub type Result<T> = std::result::Result<T, CacheError>;
