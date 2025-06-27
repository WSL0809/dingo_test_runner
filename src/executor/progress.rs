//! Progress tracking for test execution
//!
//! This module provides progress monitoring and reporting capabilities
//! for both serial and parallel test execution.

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// Progress tracker for test execution
#[derive(Debug, Clone)]
pub struct ProgressTracker {
    /// Total number of tests
    total_tests: usize,
    /// Number of completed tests
    completed: Arc<Mutex<usize>>,
    /// Number of passed tests
    passed: Arc<Mutex<usize>>,
    /// Number of failed tests
    failed: Arc<Mutex<usize>>,
    /// Start time of execution
    start_time: Instant,
}

impl ProgressTracker {
    /// Create a new progress tracker
    pub fn new(total_tests: usize) -> Self {
        Self {
            total_tests,
            completed: Arc::new(Mutex::new(0)),
            passed: Arc::new(Mutex::new(0)),
            failed: Arc::new(Mutex::new(0)),
            start_time: Instant::now(),
        }
    }

    /// Mark a test as completed and passed
    pub fn mark_passed(&self) {
        self.increment_completed();
        if let Ok(mut passed) = self.passed.lock() {
            *passed += 1;
        }
    }

    /// Mark a test as completed and failed
    pub fn mark_failed(&self) {
        self.increment_completed();
        if let Ok(mut failed) = self.failed.lock() {
            *failed += 1;
        }
    }

    /// Increment the completed counter
    fn increment_completed(&self) {
        if let Ok(mut completed) = self.completed.lock() {
            *completed += 1;
        }
    }

    /// Get current progress statistics
    pub fn get_stats(&self) -> ProgressStats {
        let completed = self.completed.lock().unwrap_or_else(|p| p.into_inner());
        let passed = self.passed.lock().unwrap_or_else(|p| p.into_inner());
        let failed = self.failed.lock().unwrap_or_else(|p| p.into_inner());

        ProgressStats {
            total: self.total_tests,
            completed: *completed,
            passed: *passed,
            failed: *failed,
            elapsed: self.start_time.elapsed(),
        }
    }

    /// Get progress percentage (0.0 to 1.0)
    pub fn get_progress(&self) -> f64 {
        if self.total_tests == 0 {
            return 1.0;
        }
        
        let completed = self.completed.lock().unwrap_or_else(|p| p.into_inner());
        *completed as f64 / self.total_tests as f64
    }

    /// Check if all tests are completed
    pub fn is_complete(&self) -> bool {
        let completed = self.completed.lock().unwrap_or_else(|p| p.into_inner());
        *completed >= self.total_tests
    }

    /// Estimate remaining time based on current progress
    pub fn estimate_remaining_time(&self) -> Option<Duration> {
        let stats = self.get_stats();
        if stats.completed == 0 {
            return None;
        }

        let rate = stats.completed as f64 / stats.elapsed.as_secs_f64();
        if rate <= 0.0 {
            return None;
        }

        let remaining_tests = (stats.total - stats.completed) as f64;
        let estimated_seconds = remaining_tests / rate;
        
        Some(Duration::from_secs_f64(estimated_seconds))
    }
}

/// Progress statistics snapshot
#[derive(Debug, Clone)]
pub struct ProgressStats {
    /// Total number of tests
    pub total: usize,
    /// Number of completed tests
    pub completed: usize,
    /// Number of passed tests
    pub passed: usize,
    /// Number of failed tests
    pub failed: usize,
    /// Elapsed time since start
    pub elapsed: Duration,
}

impl ProgressStats {
    /// Get the success rate (0.0 to 1.0)
    pub fn success_rate(&self) -> f64 {
        if self.completed == 0 {
            return 0.0;
        }
        self.passed as f64 / self.completed as f64
    }

    /// Format progress as a string
    pub fn format_progress(&self) -> String {
        let percentage = if self.total > 0 {
            (self.completed as f64 / self.total as f64 * 100.0) as u32
        } else {
            100
        };

        format!(
            "[{:3}%] {}/{} tests ({} passed, {} failed) - {:?}",
            percentage,
            self.completed,
            self.total,
            self.passed,
            self.failed,
            self.elapsed
        )
    }
}