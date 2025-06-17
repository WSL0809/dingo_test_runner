//! Terminal summary and colored output module

use super::{TestSuiteResult, TestResult};
use crate::tester::tester::TestStatus;
use console::{style, Term};
use std::io::{self, Write};

/// Print individual test case result
pub fn print_case_result(case: &TestResult) {
    match case.status {
        TestStatus::Passed => {
            println!("{} {} {}", 
                style("✓").green(),
                case.test_name,
                style(format!("({} ms)", case.duration_ms)).cyan());
        }
        TestStatus::Failed => {
            println!("{} {} {}", 
                style("✗").red(),
                case.test_name,
                style(format!("({}/{} failed, {} ms)", 
                    case.failed_queries, 
                    case.passed_queries + case.failed_queries,
                    case.duration_ms)).red());
            
            // Print first error if available
            if let Some(first_error) = case.errors.first() {
                println!("  {}", style(first_error).yellow());
            }
        }
        TestStatus::Skipped => {
            println!("{} {} {}", 
                style("⚠").yellow(),
                case.test_name,
                style("(skipped)").yellow());
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
        print!("{} ", style(format!("Skipped: {}", skipped)).bold().yellow());
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