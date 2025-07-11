//! Memory Pool System for High-Performance Object Reuse
//!
//! This module provides a comprehensive memory pooling system to reduce allocation overhead
//! and improve performance in high-frequency memory allocation scenarios.

use crossbeam_queue::SegQueue;
use once_cell::sync::Lazy;
use parking_lot::Mutex;
use std::sync::Arc;
use regex::Regex;


/// Configuration for memory pools
#[derive(Debug, Clone)]
pub struct PoolConfig {
    /// Initial capacity for string vectors
    pub string_vec_initial_capacity: usize,
    /// Maximum number of string vectors in pool
    pub string_vec_max_pool_size: usize,
    /// Maximum number of queries in pool
    pub query_max_pool_size: usize,
    /// Maximum number of row data structures in pool
    pub row_data_max_pool_size: usize,
    /// Initial capacity for byte vectors
    pub byte_vec_initial_capacity: usize,
    /// Maximum number of byte vectors in pool
    pub byte_vec_max_pool_size: usize,
}

impl Default for PoolConfig {
    fn default() -> Self {
        Self {
            string_vec_initial_capacity: 16,
            string_vec_max_pool_size: 1000,
            query_max_pool_size: 500,
            row_data_max_pool_size: 100,
            byte_vec_initial_capacity: 1024,
            byte_vec_max_pool_size: 200,
        }
    }
}

/// Global memory pool manager singleton
pub static MEMORY_POOL_MANAGER: Lazy<MemoryPoolManager> = Lazy::new(|| {
    MemoryPoolManager::new(PoolConfig::default())
});

/// Central memory pool manager
pub struct MemoryPoolManager {
    pub(crate) string_vec_pool: StringVecPool,
    pub(crate) row_data_pool: RowDataPool,
    pub(crate) byte_vec_pool: ByteVecPool,
    pub(crate) regex_vec_pool: RegexVecPool,
}

impl MemoryPoolManager {
    /// Create a new memory pool manager with the given configuration
    pub fn new(config: PoolConfig) -> Self {
        Self {
            string_vec_pool: StringVecPool::new(
                config.string_vec_initial_capacity,
                config.string_vec_max_pool_size,
            ),
            row_data_pool: RowDataPool::new(config.row_data_max_pool_size),
            byte_vec_pool: ByteVecPool::new(
                config.byte_vec_initial_capacity,
                config.byte_vec_max_pool_size,
            ),
            regex_vec_pool: RegexVecPool::new(config.string_vec_max_pool_size),
        }
    }

    /// Get a pooled string vector
    pub fn get_string_vec(&self) -> PooledStringVec {
        self.string_vec_pool.get()
    }

    /// Get a pooled row data structure (Vec<Vec<String>>)
    pub fn get_row_data(&self) -> PooledRowData {
        self.row_data_pool.get()
    }

    /// Get a pooled byte vector
    pub fn get_byte_vec(&self) -> PooledByteVec {
        self.byte_vec_pool.get()
    }

    /// Get a pooled regex vector
    pub fn get_regex_vec(&self) -> PooledRegexVec {
        self.regex_vec_pool.get()
    }

    /// Get pool statistics for monitoring
    pub fn get_stats(&self) -> PoolStats {
        PoolStats {
            string_vec_pool_size: self.string_vec_pool.pool_size(),
            row_data_pool_size: self.row_data_pool.pool_size(),
            byte_vec_pool_size: self.byte_vec_pool.pool_size(),
            regex_vec_pool_size: self.regex_vec_pool.pool_size(),
        }
    }

    /// Clear all pools (useful for testing)
    pub fn clear_all(&self) {
        self.string_vec_pool.clear();
        self.row_data_pool.clear();
        self.byte_vec_pool.clear();
        self.regex_vec_pool.clear();
    }
}

/// Pool statistics for monitoring and debugging
#[derive(Debug, Clone)]
pub struct PoolStats {
    pub string_vec_pool_size: usize,
    pub row_data_pool_size: usize,
    pub byte_vec_pool_size: usize,
    pub regex_vec_pool_size: usize,
}

/// String vector pool implementation
pub struct StringVecPool {
    pool: SegQueue<Vec<String>>,
    initial_capacity: usize,
    max_pool_size: usize,
    current_size: Arc<Mutex<usize>>,
}

impl StringVecPool {
    fn new(initial_capacity: usize, max_pool_size: usize) -> Self {
        Self {
            pool: SegQueue::new(),
            initial_capacity,
            max_pool_size,
            current_size: Arc::new(Mutex::new(0)),
        }
    }

    pub fn get(&self) -> PooledStringVec {
        match self.pool.pop() {
            Some(mut vec) => {
                vec.clear();
                let mut size = self.current_size.lock();
                *size = size.saturating_sub(1);
                PooledStringVec::new(vec, self)
            }
            None => {
                let vec = Vec::with_capacity(self.initial_capacity);
                PooledStringVec::new(vec, self)
            }
        }
    }

    pub(crate) fn return_vec(&self, vec: Vec<String>) {
        let size = {
            let mut current_size = self.current_size.lock();
            *current_size += 1;
            *current_size
        };

        if size <= self.max_pool_size {
            self.pool.push(vec);
        }
        // If pool is full, let the vector drop normally
    }

    fn pool_size(&self) -> usize {
        *self.current_size.lock()
    }

    fn clear(&self) {
        while self.pool.pop().is_some() {}
        let mut size = self.current_size.lock();
        *size = 0;
    }
}

/// Row data pool implementation (for Vec<Vec<String>>)
pub struct RowDataPool {
    pool: SegQueue<Vec<Vec<String>>>,
    max_pool_size: usize,
    current_size: Arc<Mutex<usize>>,
}

impl RowDataPool {
    fn new(max_pool_size: usize) -> Self {
        Self {
            pool: SegQueue::new(),
            max_pool_size,
            current_size: Arc::new(Mutex::new(0)),
        }
    }

    pub fn get(&self) -> PooledRowData {
        match self.pool.pop() {
            Some(mut vec) => {
                vec.clear();
                let mut size = self.current_size.lock();
                *size = size.saturating_sub(1);
                PooledRowData::new(vec, self)
            }
            None => {
                let vec = Vec::new();
                PooledRowData::new(vec, self)
            }
        }
    }

    pub(crate) fn return_vec(&self, mut vec: Vec<Vec<String>>) {
        // Clear inner vectors to avoid holding too much memory
        for inner_vec in &mut vec {
            inner_vec.clear();
        }
        vec.clear();

        let size = {
            let mut current_size = self.current_size.lock();
            *current_size += 1;
            *current_size
        };

        if size <= self.max_pool_size {
            self.pool.push(vec);
        }
    }

    fn pool_size(&self) -> usize {
        *self.current_size.lock()
    }

    fn clear(&self) {
        while self.pool.pop().is_some() {}
        let mut size = self.current_size.lock();
        *size = 0;
    }
}

/// Byte vector pool implementation
pub struct ByteVecPool {
    pool: SegQueue<Vec<u8>>,
    initial_capacity: usize,
    max_pool_size: usize,
    current_size: Arc<Mutex<usize>>,
}

impl ByteVecPool {
    fn new(initial_capacity: usize, max_pool_size: usize) -> Self {
        Self {
            pool: SegQueue::new(),
            initial_capacity,
            max_pool_size,
            current_size: Arc::new(Mutex::new(0)),
        }
    }

    pub fn get(&self) -> PooledByteVec {
        match self.pool.pop() {
            Some(mut vec) => {
                vec.clear();
                let mut size = self.current_size.lock();
                *size = size.saturating_sub(1);
                PooledByteVec::new(vec, self)
            }
            None => {
                let vec = Vec::with_capacity(self.initial_capacity);
                PooledByteVec::new(vec, self)
            }
        }
    }

    pub(crate) fn return_vec(&self, vec: Vec<u8>) {
        let size = {
            let mut current_size = self.current_size.lock();
            *current_size += 1;
            *current_size
        };

        if size <= self.max_pool_size {
            self.pool.push(vec);
        }
    }

    fn pool_size(&self) -> usize {
        *self.current_size.lock()
    }

    fn clear(&self) {
        while self.pool.pop().is_some() {}
        let mut size = self.current_size.lock();
        *size = 0;
    }
}

/// Regex vector pool implementation
pub struct RegexVecPool {
    pool: SegQueue<Vec<(Regex, String)>>,
    max_pool_size: usize,
    current_size: Arc<Mutex<usize>>,
}

impl RegexVecPool {
    fn new(max_pool_size: usize) -> Self {
        Self {
            pool: SegQueue::new(),
            max_pool_size,
            current_size: Arc::new(Mutex::new(0)),
        }
    }

    pub fn get(&self) -> PooledRegexVec {
        match self.pool.pop() {
            Some(mut vec) => {
                vec.clear();
                let mut size = self.current_size.lock();
                *size = size.saturating_sub(1);
                PooledRegexVec::new(vec, self)
            }
            None => {
                let vec = Vec::new();
                PooledRegexVec::new(vec, self)
            }
        }
    }

    pub(crate) fn return_vec(&self, vec: Vec<(Regex, String)>) {
        let size = {
            let mut current_size = self.current_size.lock();
            *current_size += 1;
            *current_size
        };

        if size <= self.max_pool_size {
            self.pool.push(vec);
        }
    }

    fn pool_size(&self) -> usize {
        *self.current_size.lock()
    }

    fn clear(&self) {
        while self.pool.pop().is_some() {}
        let mut size = self.current_size.lock();
        *size = 0;
    }
}


/// Convenience functions for accessing the global pool manager
pub fn get_string_vec() -> PooledStringVec {
    MEMORY_POOL_MANAGER.get_string_vec()
}

pub fn get_row_data() -> PooledRowData {
    MEMORY_POOL_MANAGER.get_row_data()
}

pub fn get_byte_vec() -> PooledByteVec {
    MEMORY_POOL_MANAGER.get_byte_vec()
}

pub fn get_regex_vec() -> PooledRegexVec {
    MEMORY_POOL_MANAGER.get_regex_vec()
}

/// Get pool statistics
pub fn get_pool_stats() -> PoolStats {
    MEMORY_POOL_MANAGER.get_stats()
}

/// Clear all pools (mainly for testing)
pub fn clear_all_pools() {
    MEMORY_POOL_MANAGER.clear_all()
}

/// A pooled Vec<String> that automatically returns to the pool when dropped
#[derive(Debug)]
pub struct PooledStringVec {
    inner: Option<Vec<String>>,
    pool: *const StringVecPool,
}

impl PooledStringVec {
    pub(crate) fn new(vec: Vec<String>, pool: &StringVecPool) -> Self {
        Self {
            inner: Some(vec),
            pool: pool as *const StringVecPool,
        }
    }

    /// Take the inner vector, consuming the pooled wrapper
    pub fn take(mut self) -> Vec<String> {
        self.inner.take().unwrap_or_else(Vec::new)
    }

    /// Get the length of the vector
    pub fn len(&self) -> usize {
        self.inner.as_ref().map(|v| v.len()).unwrap_or(0)
    }

    /// Check if the vector is empty
    pub fn is_empty(&self) -> bool {
        self.inner.as_ref().map(|v| v.is_empty()).unwrap_or(true)
    }

    /// Clear the vector
    pub fn clear(&mut self) {
        if let Some(ref mut vec) = self.inner {
            vec.clear();
        }
    }

    /// Push a string to the vector
    pub fn push(&mut self, value: String) {
        if let Some(ref mut vec) = self.inner {
            vec.push(value);
        }
    }

    /// Extend the vector with an iterator
    pub fn extend<I: IntoIterator<Item = String>>(&mut self, iter: I) {
        if let Some(ref mut vec) = self.inner {
            vec.extend(iter);
        }
    }
}

impl std::ops::Deref for PooledStringVec {
    type Target = Vec<String>;

    fn deref(&self) -> &Self::Target {
        self.inner.as_ref().unwrap()
    }
}

impl std::ops::DerefMut for PooledStringVec {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.inner.as_mut().unwrap()
    }
}

impl Drop for PooledStringVec {
    fn drop(&mut self) {
        if let Some(vec) = self.inner.take() {
            unsafe {
                (*self.pool).return_vec(vec);
            }
        }
    }
}

/// A pooled Vec<Vec<String>> for database query results
#[derive(Debug)]
pub struct PooledRowData {
    inner: Option<Vec<Vec<String>>>,
    pool: *const RowDataPool,
}

impl PooledRowData {
    pub(crate) fn new(vec: Vec<Vec<String>>, pool: &RowDataPool) -> Self {
        Self {
            inner: Some(vec),
            pool: pool as *const RowDataPool,
        }
    }

    /// Take the inner vector, consuming the pooled wrapper
    pub fn take(mut self) -> Vec<Vec<String>> {
        self.inner.take().unwrap_or_else(Vec::new)
    }

    /// Get the length of the vector
    pub fn len(&self) -> usize {
        self.inner.as_ref().map(|v| v.len()).unwrap_or(0)
    }

    /// Check if the vector is empty
    pub fn is_empty(&self) -> bool {
        self.inner.as_ref().map(|v| v.is_empty()).unwrap_or(true)
    }

    /// Clear the vector
    pub fn clear(&mut self) {
        if let Some(ref mut vec) = self.inner {
            vec.clear();
        }
    }

    /// Push a row to the vector
    pub fn push(&mut self, row: Vec<String>) {
        if let Some(ref mut vec) = self.inner {
            vec.push(row);
        }
    }
}

impl std::ops::Deref for PooledRowData {
    type Target = Vec<Vec<String>>;

    fn deref(&self) -> &Self::Target {
        self.inner.as_ref().unwrap()
    }
}

impl std::ops::DerefMut for PooledRowData {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.inner.as_mut().unwrap()
    }
}

impl Drop for PooledRowData {
    fn drop(&mut self) {
        if let Some(vec) = self.inner.take() {
            unsafe {
                (*self.pool).return_vec(vec);
            }
        }
    }
}

/// A pooled Vec<u8> for output buffers
#[derive(Debug)]
pub struct PooledByteVec {
    inner: Option<Vec<u8>>,
    pool: *const ByteVecPool,
}

impl PooledByteVec {
    pub(crate) fn new(vec: Vec<u8>, pool: &ByteVecPool) -> Self {
        Self {
            inner: Some(vec),
            pool: pool as *const ByteVecPool,
        }
    }

    /// Take the inner vector, consuming the pooled wrapper
    pub fn take(mut self) -> Vec<u8> {
        self.inner.take().unwrap_or_else(Vec::new)
    }

    /// Get the length of the vector
    pub fn len(&self) -> usize {
        self.inner.as_ref().map(|v| v.len()).unwrap_or(0)
    }

    /// Check if the vector is empty
    pub fn is_empty(&self) -> bool {
        self.inner.as_ref().map(|v| v.is_empty()).unwrap_or(true)
    }

    /// Clear the vector
    pub fn clear(&mut self) {
        if let Some(ref mut vec) = self.inner {
            vec.clear();
        }
    }

    /// Push a byte to the vector
    pub fn push(&mut self, value: u8) {
        if let Some(ref mut vec) = self.inner {
            vec.push(value);
        }
    }

    /// Extend the vector with an iterator
    pub fn extend<I: IntoIterator<Item = u8>>(&mut self, iter: I) {
        if let Some(ref mut vec) = self.inner {
            vec.extend(iter);
        }
    }
}

impl std::ops::Deref for PooledByteVec {
    type Target = Vec<u8>;

    fn deref(&self) -> &Self::Target {
        self.inner.as_ref().unwrap()
    }
}

impl std::ops::DerefMut for PooledByteVec {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.inner.as_mut().unwrap()
    }
}

impl Drop for PooledByteVec {
    fn drop(&mut self) {
        if let Some(vec) = self.inner.take() {
            unsafe {
                (*self.pool).return_vec(vec);
            }
        }
    }
}

/// A pooled Vec<(Regex, String)> for replace regex operations
#[derive(Debug)]
pub struct PooledRegexVec {
    inner: Option<Vec<(Regex, String)>>,
    pool: *const RegexVecPool,
}

impl PooledRegexVec {
    pub(crate) fn new(vec: Vec<(Regex, String)>, pool: &RegexVecPool) -> Self {
        Self {
            inner: Some(vec),
            pool: pool as *const RegexVecPool,
        }
    }

    /// Take the inner vector, consuming the pooled wrapper
    pub fn take(mut self) -> Vec<(Regex, String)> {
        self.inner.take().unwrap_or_else(Vec::new)
    }

    /// Get the length of the vector
    pub fn len(&self) -> usize {
        self.inner.as_ref().map(|v| v.len()).unwrap_or(0)
    }

    /// Check if the vector is empty
    pub fn is_empty(&self) -> bool {
        self.inner.as_ref().map(|v| v.is_empty()).unwrap_or(true)
    }

    /// Clear the vector
    pub fn clear(&mut self) {
        if let Some(ref mut vec) = self.inner {
            vec.clear();
        }
    }

    /// Push a regex rule to the vector
    pub fn push(&mut self, regex: Regex, replacement: String) {
        if let Some(ref mut vec) = self.inner {
            vec.push((regex, replacement));
        }
    }
}

impl std::ops::Deref for PooledRegexVec {
    type Target = Vec<(Regex, String)>;

    fn deref(&self) -> &Self::Target {
        self.inner.as_ref().unwrap()
    }
}

impl std::ops::DerefMut for PooledRegexVec {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.inner.as_mut().unwrap()
    }
}

impl Drop for PooledRegexVec {
    fn drop(&mut self) {
        if let Some(vec) = self.inner.take() {
            unsafe {
                (*self.pool).return_vec(vec);
            }
        }
    }
}

// Implement useful traits for pooled types
impl std::iter::FromIterator<String> for PooledStringVec {
    fn from_iter<T: IntoIterator<Item = String>>(iter: T) -> Self {
        let mut pooled = get_string_vec();
        pooled.extend(iter);
        pooled
    }
}

impl AsRef<[u8]> for PooledByteVec {
    fn as_ref(&self) -> &[u8] {
        self.inner.as_ref().map(|v| v.as_slice()).unwrap_or(&[])
    }
}

// Iterator support for PooledRegexVec  
impl<'a> IntoIterator for &'a PooledRegexVec {
    type Item = &'a (Regex, String);
    type IntoIter = std::slice::Iter<'a, (Regex, String)>;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.as_ref().unwrap().iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_string_vec_pool_basic() {
        let config = PoolConfig::default();
        let manager = MemoryPoolManager::new(config);

        // Get a pooled vector
        let mut pooled_vec = manager.get_string_vec();
        pooled_vec.push("test".to_string());
        pooled_vec.push("data".to_string());

        assert_eq!(pooled_vec.len(), 2);
        assert_eq!(pooled_vec[0], "test");
        assert_eq!(pooled_vec[1], "data");

        // Drop the pooled vector (should return to pool)
        drop(pooled_vec);

        // Get another vector (should reuse the previous one)
        let pooled_vec2 = manager.get_string_vec();
        assert_eq!(pooled_vec2.len(), 0); // Should be cleared
    }

    #[test]
    fn test_row_data_pool_basic() {
        let config = PoolConfig::default();
        let manager = MemoryPoolManager::new(config);

        let mut row_data = manager.get_row_data();
        row_data.push(vec!["col1".to_string(), "col2".to_string()]);
        row_data.push(vec!["val1".to_string(), "val2".to_string()]);

        assert_eq!(row_data.len(), 2);
        assert_eq!(row_data[0][0], "col1");

        drop(row_data);

        let row_data2 = manager.get_row_data();
        assert_eq!(row_data2.len(), 0);
    }

    #[test]
    fn test_pool_stats() {
        clear_all_pools();
        
        let stats_before = get_pool_stats();
        
        let _vec1 = get_string_vec();
        let _vec2 = get_row_data();
        
        // Vectors are still in use, pool should be empty
        let stats_after = get_pool_stats();
        assert_eq!(stats_after.string_vec_pool_size, stats_before.string_vec_pool_size);
        
        drop(_vec1);
        drop(_vec2);
        
        // Now vectors should be returned to pool
        let stats_final = get_pool_stats();
        assert!(stats_final.string_vec_pool_size >= stats_before.string_vec_pool_size);
    }
}