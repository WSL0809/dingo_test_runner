pub mod cli;
pub mod report;
pub mod stub;
pub mod tester;
pub mod util;
pub mod loader;

use cli::Args;
use tester::tester::Tester;
use anyhow::Result;
use log::{error, info};

fn main() -> Result<()> {
    // Parse command line arguments
    let args = Args::parse_args();
    
    // Validate arguments
    if let Err(e) = args.validate() {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
    
    // Initialize logging
    init_logging(&args.log_level)?;
    
    info!("MySQL Test Runner (Rust) v0.2.0");
    info!("Connecting to {}@{}:{}", args.user, args.host, args.port);
    
    // Create a clone of args for reuse in the loop
    let base_args = args.clone();

    // Run tests
    let mut total_tests = 0;
    let mut passed_tests = 0;
    let mut failed_tests = 0;
    
    let resolved_tests = if args.all {
        // Load all tests from the `t/` directory
        match loader::load_all_tests() {
            Ok(test_names) => {
                info!("Found {} tests to run.", test_names.len());
                test_names.into_iter().map(|name| {
                    let path = std::env::current_dir().unwrap_or_default().join("t").join(format!("{}.test", name));
                    cli::ResolvedTest { name, path }
                }).collect()
            }
            Err(e) => {
                error!("Failed to load tests: {}", e);
                std::process::exit(1);
            }
        }
    } else {
        // Resolve the specific test inputs provided
        match args.resolve_test_inputs() {
            Ok(tests) => {
                info!("Resolved {} test(s) to run.", tests.len());
                // Show what was resolved
                for test in &tests {
                    info!("  {} -> {}", test.name, test.path.display());
                }
                tests
            }
            Err(e) => {
                error!("Failed to resolve test inputs: {}", e);
                std::process::exit(1);
            }
        }
    };

    if resolved_tests.is_empty() {
        info!("No test files specified or found. Exiting.");
        return Ok(());
    }

    for resolved_test in &resolved_tests {
        total_tests += 1;
        info!("Running test: {} ({})", resolved_test.name, resolved_test.path.display());
        
        // Check if test file exists
        if !resolved_test.path.exists() {
            error!("✗ Test file not found: {}", resolved_test.path.display());
            failed_tests += 1;
            continue;
        }
        
        // Create a new tester instance for each test file to ensure isolation
        let mut tester = match Tester::new(base_args.clone()) {
            Ok(t) => t,
            Err(e) => {
                error!("Failed to create tester for {}: {}", resolved_test.name, e);
                failed_tests += 1;
                continue; // Skip to the next test
            }
        };

        match tester.run_test_file(&resolved_test.name) {
            Ok(result) => {
                if result.success {
                    passed_tests += 1;
                    info!("✓ Test '{}' passed ({} queries)", result.test_name, result.passed_queries);
                } else {
                    failed_tests += 1;
                    error!("✗ Test '{}' failed ({}/{} queries failed)", 
                        result.test_name, result.failed_queries, 
                        result.passed_queries + result.failed_queries);
                    
                    for error in &result.errors {
                        error!("  {}", error);
                    }
                }
            }
            Err(e) => {
                failed_tests += 1;
                error!("✗ Test file '{}' failed to execute: {}", resolved_test.name, e);
            }
        }
        // 显式 drop Tester，确保连接及资源释放；若 hang 可考虑加超时机制
        drop(tester);
    }
    
    // Print summary
    info!("Test execution completed:");
    info!("  Total tests: {}", total_tests);
    info!("  Passed: {}", passed_tests);
    info!("  Failed: {}", failed_tests);
    
    // 显式退出，避免底层线程或连接阻止程序结束
    let exit_code = if failed_tests > 0 { 1 } else { 0 };
    std::process::exit(exit_code);
}

fn init_logging(log_level: &str) -> Result<()> {
    let level = match log_level.to_lowercase().as_str() {
        "error" => log::LevelFilter::Error,
        "warn" => log::LevelFilter::Warn,
        "info" => log::LevelFilter::Info,
        "debug" => log::LevelFilter::Debug,
        "trace" => log::LevelFilter::Trace,
        _ => log::LevelFilter::Error,
    };
    
    env_logger::Builder::new()
        .filter_level(level)
        .format_timestamp_secs()
        .init();
    
    Ok(())
}
