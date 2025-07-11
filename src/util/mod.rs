pub mod error_utils;
pub mod regex;
pub mod memory_pool;

// Re-export commonly used types for convenience
pub use memory_pool::{
    get_string_vec, get_row_data, get_byte_vec, get_regex_vec,
    get_pool_stats, clear_all_pools, PoolConfig, MemoryPoolManager,
    PooledStringVec, PooledRowData, PooledByteVec, PooledRegexVec
};
