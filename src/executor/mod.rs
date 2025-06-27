//! File-level concurrent execution engine
//!
//! This module provides parallel execution of test files while maintaining
//! full backward compatibility with the existing serial execution model.

pub mod file_executor;
pub mod progress;

pub use file_executor::FileExecutor;
pub use progress::ProgressTracker;