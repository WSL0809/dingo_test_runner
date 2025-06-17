pub mod cli;
pub mod report;
pub mod stub;
pub mod tester;
pub mod util;
pub mod loader;

use cli::Args;
use tester::tester::Tester;
use report::{TestSuiteResult, summary, xunit};
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
    
    // Create test suite result
    let mut suite = TestSuiteResult::new("mysql-test-runner");
    
    // Create a clone of args for reuse in the loop
    let base_args = args.clone();

    // Resolve tests to run
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

    // Run tests
    for resolved_test in &resolved_tests {
        // Print running indicator
        summary::print_running_test(&resolved_test.name);
        
        // Check if test file exists
        if !resolved_test.path.exists() {
            let mut failed_case = tester::tester::TestResult::new(&resolved_test.name);
            failed_case.add_error(format!("Test file not found: {}", resolved_test.path.display()));
            summary::print_case_result(&failed_case);
            suite.add_case(failed_case);
            continue;
        }
        
        // Create a new tester instance for each test file to ensure isolation
        let mut tester = match Tester::new(base_args.clone()) {
            Ok(t) => t,
            Err(e) => {
                let mut failed_case = tester::tester::TestResult::new(&resolved_test.name);
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
                let mut failed_case = tester::tester::TestResult::new(&resolved_test.name);
                failed_case.add_error(format!("Test execution failed: {}", e));
                summary::print_case_result(&failed_case);
                suite.add_case(failed_case);
            }
        }
        
        // 显式 drop Tester，确保连接及资源释放
        drop(tester);
    }
    
    // Print final summary
    summary::print_summary(&suite);
    
    // Write JUnit XML report if requested
    if !args.xunit_file.is_empty() {
        if let Err(e) = xunit::write_xunit_report(&suite, &args.xunit_file) {
            error!("Failed to write JUnit XML report: {}", e);
        } else {
            info!("JUnit XML report written to: {}", args.xunit_file);
        }
    }
    
    // Exit with appropriate code
    let exit_code = if suite.all_passed() { 0 } else { 1 };
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
