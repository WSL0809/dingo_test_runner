//! Terminal summary and colored output module

use super::{TestResult, TestSuiteResult};
use crate::tester::tester::TestStatus;
use console::{style, Term};
use std::io::{self, Write};

/// Print individual test case result
pub fn print_case_result(case: &TestResult) {
    match case.status {
        TestStatus::Passed => {
            println!(
                "{} {} {}",
                style("✓").green(),
                case.test_name,
                style(format!("({} ms)", case.duration_ms)).cyan()
            );
        }
        TestStatus::Failed => {
            println!(
                "{} {} {}",
                style("✗").red(),
                case.test_name,
                style(format!(
                    "({}/{} failed, {} ms)",
                    case.failed_queries,
                    case.passed_queries + case.failed_queries,
                    case.duration_ms
                ))
                .red()
            );

            // Print first error if available
            if let Some(first_error) = case.errors.first() {
                println!("  {}", style(first_error).yellow());
            }
        }
        TestStatus::Skipped => {
            println!(
                "{} {} {}",
                style("⚠").yellow(),
                case.test_name,
                style("(skipped)").yellow()
            );
        }
    }
}

/// Print running test indicator
pub fn print_running_test(test_name: &str) {
    print!("{} running {} ... ", style("▶").blue(), test_name);
    io::stdout().flush().unwrap_or(());
}

/// Print final summary
pub fn print_summary(suite: &TestSuiteResult) {
    let _term = Term::stdout();

    // Print separator
    println!("{}", "─".repeat(60));

    // Print main summary line
    let total = suite.total_tests();
    let passed = suite.passed_tests();
    let failed = suite.failed_tests();
    let skipped = suite.skipped_tests();
    let duration_sec = suite.total_duration_ms as f64 / 1000.0;

    print!("{} ", style(format!("Total: {}", total)).bold().cyan());

    if passed > 0 {
        print!("{} ", style(format!("Passed: {}", passed)).bold().green());
    }

    if failed > 0 {
        print!("{} ", style(format!("Failed: {}", failed)).bold().red());
    }

    if skipped > 0 {
        print!(
            "{} ",
            style(format!("Skipped: {}", skipped)).bold().yellow()
        );
    }

    println!("{}", style(format!("⏱ {:.1} s", duration_sec)).cyan());

    // Print pass rate
    if total > 0 {
        let pass_rate = suite.pass_rate();
        let rate_style = if pass_rate >= 90.0 {
            style(format!("{:.1}%", pass_rate)).green()
        } else if pass_rate >= 70.0 {
            style(format!("{:.1}%", pass_rate)).yellow()
        } else {
            style(format!("{:.1}%", pass_rate)).red()
        };
        println!("{} {}", style("Pass rate:").bold(), rate_style);
    }

    // Print separator
    println!("{}", "─".repeat(60));

    // Print failed tests summary if any
    if failed > 0 {
        println!("{}", style("Failed tests:").bold().red());
        for case in &suite.cases {
            if case.status == TestStatus::Failed {
                println!("  • {}", case.test_name);
                for error in &case.errors {
                    println!("    {}", style(error).red());
                }
            }
        }
        println!();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_result(name: &str, status: TestStatus, duration_ms: u64) -> TestResult {
        let success = status == TestStatus::Passed;
        let passed_queries = if status == TestStatus::Passed { 1 } else { 0 };
        let failed_queries = if status == TestStatus::Failed { 1 } else { 0 };
        let errors = if status == TestStatus::Failed {
            vec!["Test error".to_string()]
        } else {
            vec![]
        };
        
        TestResult {
            test_name: name.to_string(),
            success,
            status,
            duration_ms,
            start_time: "2023-01-01T00:00:00Z".to_string(),
            end_time: "2023-01-01T00:00:01Z".to_string(),
            passed_queries,
            failed_queries,
            errors,
            stdout: "".to_string(),
            stderr: "".to_string(),
            classname: format!("test.{}", name),
            query_failures: vec![],
        }
    }

    #[test]
    fn test_print_case_result_passed() {
        let case = create_test_result("test_pass", TestStatus::Passed, 100);
        
        // This test verifies the function doesn't panic
        // In a real test environment, you might want to capture stdout
        print_case_result(&case);
    }

    #[test]
    fn test_print_case_result_failed() {
        let case = create_test_result("test_fail", TestStatus::Failed, 200);
        
        // This test verifies the function doesn't panic
        print_case_result(&case);
    }

    #[test]
    fn test_print_case_result_skipped() {
        let case = create_test_result("test_skip", TestStatus::Skipped, 0);
        
        // This test verifies the function doesn't panic
        print_case_result(&case);
    }

    #[test]
    fn test_print_running_test() {
        // This test verifies the function doesn't panic
        print_running_test("test_running");
    }

    #[test]
    fn test_print_summary_empty_suite() {
        let suite = TestSuiteResult::new("empty_suite");
        
        // This test verifies the function doesn't panic with empty suite
        print_summary(&suite);
    }

    #[test]
    fn test_print_summary_mixed_results() {
        let mut suite = TestSuiteResult::new("mixed_suite");
        
        suite.add_case(create_test_result("test1", TestStatus::Passed, 100));
        suite.add_case(create_test_result("test2", TestStatus::Failed, 200));
        suite.add_case(create_test_result("test3", TestStatus::Skipped, 0));
        
        // This test verifies the function doesn't panic with mixed results
        print_summary(&suite);
        
        // Verify basic statistics
        assert_eq!(suite.total_tests(), 3);
        assert_eq!(suite.passed_tests(), 1);
        assert_eq!(suite.failed_tests(), 1);
        assert_eq!(suite.skipped_tests(), 1);
    }

    #[test]
    fn test_print_summary_all_passed() {
        let mut suite = TestSuiteResult::new("all_passed_suite");
        
        suite.add_case(create_test_result("test1", TestStatus::Passed, 100));
        suite.add_case(create_test_result("test2", TestStatus::Passed, 150));
        
        print_summary(&suite);
        
        assert_eq!(suite.total_tests(), 2);
        assert_eq!(suite.passed_tests(), 2);
        assert_eq!(suite.failed_tests(), 0);
        assert!(suite.pass_rate() == 100.0);
    }

    #[test]
    fn test_print_summary_all_failed() {
        let mut suite = TestSuiteResult::new("all_failed_suite");
        
        suite.add_case(create_test_result("test1", TestStatus::Failed, 100));
        suite.add_case(create_test_result("test2", TestStatus::Failed, 150));
        
        print_summary(&suite);
        
        assert_eq!(suite.total_tests(), 2);
        assert_eq!(suite.passed_tests(), 0);
        assert_eq!(suite.failed_tests(), 2);
        assert!(suite.pass_rate() == 0.0);
    }
}
