//! File-level concurrent executor for test files
//!
//! This module enables parallel execution of multiple test files while ensuring
//! proper data isolation and resource management.

use crate::cli::{Args, ResolvedTest};
use crate::report::{summary, TestSuiteResult};
use crate::tester::tester::{Tester, TestResult};
use anyhow::{anyhow, Result};
use log::{debug, info, warn};
use rayon::prelude::*;
use std::sync::{Arc, Mutex};
use std::time::Instant;

/// File-level concurrent executor
pub struct FileExecutor {
    /// Base arguments for test execution
    base_args: Args,
    /// Number of parallel workers
    parallel_workers: usize,
}

impl FileExecutor {
    /// Create a new file executor with the given arguments
    pub fn new(args: Args) -> Self {
        let parallel_workers = if args.parallel > 0 {
            args.parallel
        } else {
            1 // Default to serial execution for backward compatibility
        };

        Self {
            base_args: args,
            parallel_workers,
        }
    }

    /// Execute tests serially (backward compatibility mode)
    pub fn execute_serial(&self, resolved_tests: &[ResolvedTest]) -> Result<TestSuiteResult> {
        info!(
            "Executing {} tests in serial mode",
            resolved_tests.len()
        );

        let mut suite = TestSuiteResult::new("mysql-test-runner");
        let base_args = self.base_args.clone();

        for resolved_test in resolved_tests {
            // Print running indicator
            summary::print_running_test(&resolved_test.name);

            // Check if test file exists
            if !resolved_test.path.exists() {
                let mut failed_case = TestResult::new(&resolved_test.name);
                failed_case.add_error(format!(
                    "Test file not found: {}",
                    resolved_test.path.display()
                ));
                summary::print_case_result(&failed_case);
                suite.add_case(failed_case);
                continue;
            }

            // Create a new tester instance for each test file to ensure isolation
            let mut tester = match Tester::new(base_args.clone()) {
                Ok(t) => t,
                Err(e) => {
                    let mut failed_case = TestResult::new(&resolved_test.name);
                    failed_case.add_error(format!("Failed to create tester: {}", e));
                    summary::print_case_result(&failed_case);
                    suite.add_case(failed_case);
                    continue;
                }
            };

            // Run the test
            match tester.run_test_file(&resolved_test.name) {
                Ok(result) => {
                    summary::print_case_result(&result);
                    suite.add_case(result);
                }
                Err(e) => {
                    let mut failed_case = TestResult::new(&resolved_test.name);
                    failed_case.add_error(format!("Test execution failed: {}", e));
                    summary::print_case_result(&failed_case);
                    suite.add_case(failed_case);
                }
            }

            // Explicitly drop tester to ensure connection and resource cleanup
            drop(tester);
        }

        Ok(suite)
    }

    /// Execute tests in parallel
    pub fn execute_parallel(&self, resolved_tests: &[ResolvedTest]) -> Result<TestSuiteResult> {
        info!(
            "Executing {} tests in parallel mode with {} workers",
            resolved_tests.len(),
            self.parallel_workers
        );

        // Configure rayon thread pool
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(self.parallel_workers)
            .build()
            .map_err(|e| anyhow!("Failed to create thread pool: {}", e))?;

        // Use Arc<Mutex<Vec<TestResult>>> to collect results safely
        let results = Arc::new(Mutex::new(Vec::<(usize, TestResult)>::new()));
        let base_args = self.base_args.clone();

        pool.install(|| {
            resolved_tests
                .par_iter()
                .enumerate()
                .for_each(|(index, resolved_test)| {
                    let test_result = self.execute_single_test(resolved_test, &base_args);
                    
                    match results.lock() {
                        Ok(mut guard) => guard.push((index, test_result)),
                        Err(poisoned) => {
                            warn!("Results mutex poisoned, continuing with inner data");
                            let mut guard = poisoned.into_inner();
                            guard.push((index, test_result));
                        }
                    }
                });
        });

        // Extract results and sort by original order
        let mut final_results = match results.lock() {
            Ok(guard) => guard.clone(),
            Err(poisoned) => {
                warn!("Results mutex poisoned during collection; using inner data");
                poisoned.into_inner().clone()
            }
        };
        final_results.sort_by_key(|(index, _)| *index);

        // Build the test suite result
        let mut suite = TestSuiteResult::new("mysql-test-runner");
        for (_, result) in final_results {
            suite.add_case(result);
        }

        Ok(suite)
    }

    /// Execute a single test file (used by parallel execution)
    fn execute_single_test(&self, resolved_test: &ResolvedTest, base_args: &Args) -> TestResult {
        debug!("Starting test: {}", resolved_test.name);
        let start_time = Instant::now();

        // Check if test file exists
        if !resolved_test.path.exists() {
            let mut failed_case = TestResult::new(&resolved_test.name);
            failed_case.add_error(format!(
                "Test file not found: {}",
                resolved_test.path.display()
            ));
            failed_case.set_duration(start_time.elapsed().as_millis() as u64);
            return failed_case;
        }

        // Create a new tester instance with isolated database schema
        let mut isolated_args = base_args.clone();
        
        // Generate unique database name for isolation
        let thread_id = rayon::current_thread_index().unwrap_or(0);
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis();
        
        // Sanitize test name for database name
        let sanitized_name = resolved_test.name
            .chars()
            .map(|c| if c.is_alphanumeric() || c == '_' { c } else { '_' })
            .collect::<String>();
        
        let unique_db_suffix = format!("{}_{}_{}_{}", 
            sanitized_name, thread_id, timestamp, std::process::id());
        
        // Store the unique suffix in params for tester to use
        if isolated_args.params.is_empty() {
            isolated_args.params = format!("test_db_suffix={}", unique_db_suffix);
        } else {
            isolated_args.params = format!("{}&test_db_suffix={}", isolated_args.params, unique_db_suffix);
        }

        let mut tester = match Tester::new(isolated_args) {
            Ok(t) => t,
            Err(e) => {
                let mut failed_case = TestResult::new(&resolved_test.name);
                failed_case.add_error(format!("Failed to create tester: {}", e));
                failed_case.set_duration(start_time.elapsed().as_millis() as u64);
                return failed_case;
            }
        };

        // Run the test
        let result = match tester.run_test_file(&resolved_test.name) {
            Ok(result) => result,
            Err(e) => {
                let mut failed_case = TestResult::new(&resolved_test.name);
                failed_case.add_error(format!("Test execution failed: {}", e));
                failed_case.set_duration(start_time.elapsed().as_millis() as u64);
                failed_case
            }
        };

        // Explicitly drop tester to ensure cleanup
        drop(tester);

        debug!("Completed test: {} in {:?}", resolved_test.name, start_time.elapsed());
        result
    }

    /// Execute tests (automatically chooses serial or parallel based on configuration)
    pub fn execute(&self, resolved_tests: &[ResolvedTest]) -> Result<TestSuiteResult> {
        if self.parallel_workers <= 1 {
            self.execute_serial(resolved_tests)
        } else {
            self.execute_parallel(resolved_tests)
        }
    }
}