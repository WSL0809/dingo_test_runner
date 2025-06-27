pub mod cli;
pub mod executor;
pub mod loader;
pub mod report;
pub mod stub;
pub mod tester;
pub mod util;

use anyhow::Result;
use cli::Args;
use executor::FileExecutor;
use log::{error, info, warn};
use report::{
    create_renderer_with_allure_dir, html, summary, xunit, TestSuiteResult,
};
use stub::email::MailSender;

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
    
    if args.parallel > 1 {
        info!("File-level parallel execution enabled with {} workers", args.parallel);
    } else {
        info!("Serial execution mode (backward compatibility)");
    }

    // Resolve tests to run
    let resolved_tests = if args.all {
        // Load all tests from the `t/` directory
        match loader::load_all_tests() {
            Ok(test_names) => {
                info!("Found {} tests to run.", test_names.len());
                test_names
                    .into_iter()
                    .map(|name| {
                        let path = std::env::current_dir()
                            .unwrap_or_default()
                            .join("t")
                            .join(format!("{}.test", name));
                        cli::ResolvedTest { name, path }
                    })
                    .collect()
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

    // Create file executor and run tests
    let executor = FileExecutor::new(args.clone());
    let suite = match executor.execute(&resolved_tests) {
        Ok(suite) => suite,
        Err(e) => {
            error!("Failed to execute tests: {}", e);
            std::process::exit(1);
        }
    };

    // Generate and output report using the new renderer architecture
    // Determine if Allure output should be used
    let allure_dir = if !args.allure_dir.is_empty() {
        Some(args.allure_dir.as_str())
    } else {
        None
    };

    let report_format = if allure_dir.is_some() {
        "allure"
    } else {
        &args.report_format
    };

    match create_renderer_with_allure_dir(report_format, allure_dir) {
        Ok(renderer) => {
            match renderer.render(&suite) {
                Ok(report_content) => {
                    match report_format {
                        "terminal" | "console" => {
                            // For terminal, use the existing colored output
                            summary::print_summary(&suite);
                        }
                        "html" => {
                            // Write HTML to file or stdout
                            if !args.xunit_file.is_empty() {
                                let html_file = args.xunit_file.replace(".xml", ".html");
                                if let Err(e) = std::fs::write(&html_file, &report_content) {
                                    error!("Failed to write HTML report: {}", e);
                                } else {
                                    info!("HTML report written to: {}", html_file);
                                }
                            } else {
                                println!("{}", report_content);
                            }
                        }
                        "allure" => {
                            // Allure output is written to directory, just print summary
                            info!("{}", report_content);
                            summary::print_summary(&suite);
                        }
                        _ => {
                            // For other formats, print to stdout
                            println!("{}", report_content);
                        }
                    }
                }
                Err(e) => {
                    error!("Failed to render report: {}", e);
                    // Fallback to terminal output
                    summary::print_summary(&suite);
                }
            }
        }
        Err(e) => {
            error!("Failed to create report renderer: {}", e);
            // Fallback to terminal output
            summary::print_summary(&suite);
        }
    }

    // Write JUnit XML report if requested (backwards compatibility)
    if !args.xunit_file.is_empty() && args.report_format != "xunit" && allure_dir.is_none() {
        if let Err(e) = xunit::write_xunit_report(&suite, &args.xunit_file) {
            error!("Failed to write JUnit XML report: {}", e);
        } else {
            info!("JUnit XML report written to: {}", args.xunit_file);
        }
    }

    // Send email report if configured
    if let Some(email_config) = args.get_email_config() {
        info!("Sending test report email...");
        match send_email_report(&suite, &email_config, &args.xunit_file) {
            Ok(_) => {
                info!("Email report sent successfully");
            }
            Err(e) => {
                warn!("Failed to send email report: {}", e);
                // Don't exit with error for email failure
            }
        }
    }

    // Exit with appropriate code
    let exit_code = if suite.all_passed() { 0 } else { 1 };
    std::process::exit(exit_code);
}

/// Send email report with test results
fn send_email_report(
    suite_result: &TestSuiteResult,
    email_config: &cli::EmailConfig,
    xunit_file: &str,
) -> Result<()> {
    // Create mail sender
    let mail_sender = MailSender::new(email_config.clone())?;

    // Generate plain text report
    let plain_text_body = html::generate_plain_text_report(suite_result);

    // Generate HTML report
    let html_body = {
        let html_report = html::HtmlReport::new(suite_result, &suite_result.cases);
        html_report.generate().unwrap_or_else(|e| {
            warn!(
                "Failed to generate HTML report: {}, falling back to plain text",
                e
            );
            plain_text_body.clone()
        })
    };

    // Determine XUnit file path
    let xunit_path = if !xunit_file.is_empty() && email_config.attach_xunit {
        Some(std::path::Path::new(xunit_file))
    } else {
        None
    };

    // Send email
    mail_sender.send_test_report(suite_result, &plain_text_body, &html_body, xunit_path)?;

    Ok(())
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
        .format(|buf, record| {
            use chrono::Local;
            use std::io::Write;
            
            writeln!(
                buf,
                "{} [{}] {}",
                Local::now().format("%Y-%m-%d %H:%M:%S"),
                record.level(),
                record.args()
            )
        })
        .init();

    Ok(())
}
