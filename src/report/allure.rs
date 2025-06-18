//! Allure report generation for test results
//!
//! This module generates Allure 2.0 compatible JSON results and attachments.
//! Each test case generates a separate result JSON file and optional attachment files.

use super::{TestSuiteResult, ReportRenderer};
use crate::tester::tester::{TestResult, TestStatus, QueryFailureDetail};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use uuid::Uuid;

/// Allure report renderer that generates JSON result files
pub struct AllureRenderer {
    /// Output directory for Allure results
    pub output_dir: String,
}

impl AllureRenderer {
    /// Create a new Allure renderer
    pub fn new(output_dir: String) -> Self {
        Self { output_dir }
    }
}

impl ReportRenderer for AllureRenderer {
    fn render(&self, suite: &TestSuiteResult) -> Result<String> {
        // Create output directory if it doesn't exist
        fs::create_dir_all(&self.output_dir)?;
        
        // Generate environment.properties
        self.generate_environment_properties(suite)?;
        
        // Generate JSON result file for each test case
        for case in &suite.cases {
            self.generate_test_result_json(case, suite)?;
        }
        
        Ok(format!("Allure results generated in: {}", self.output_dir))
    }

    fn format_name(&self) -> &'static str {
        "allure"
    }
}

impl AllureRenderer {
    /// Generate environment.properties file
    fn generate_environment_properties(&self, suite: &TestSuiteResult) -> Result<()> {
        let env_path = Path::new(&self.output_dir).join("environment.properties");
        let mut content = String::new();
        
        // CLI Args
        let cli_args = suite.environment.cli_args.join(" ");
        content.push_str(&format!("CLI.Args={}\n", cli_args));
        
        // Git commit
        if let Some(ref commit) = suite.environment.git_commit {
            content.push_str(&format!("Git.Commit={}\n", commit));
        }
        
        // OS
        content.push_str(&format!("OS={}\n", suite.environment.os));
        
        // Rust version
        content.push_str(&format!("Rust.Version={}\n", suite.environment.rust_version));
        
        // MySQL version if available
        if let Some(ref mysql_version) = suite.environment.mysql_version {
            content.push_str(&format!("MySQL.Version={}\n", mysql_version));
        }
        
        fs::write(env_path, content)?;
        Ok(())
    }
    
    /// Generate Allure JSON result file for a test case
    fn generate_test_result_json(&self, case: &TestResult, suite: &TestSuiteResult) -> Result<()> {
        let test_uuid = Uuid::new_v4().to_string();
        let result_path = Path::new(&self.output_dir).join(format!("{}-result.json", test_uuid));
        
        // Parse timestamps to get milliseconds since epoch
        let (start_ms, stop_ms, duration_ms) = self.parse_timestamps(case)?;
        
        // Generate attachments (test file content, etc.)
        let attachments = self.generate_attachments(case, &test_uuid)?;
        
        let result = AllureTestResult {
            uuid: test_uuid.clone(),
            name: case.test_name.clone(),
            full_name: format!("mysql-test.{}.{}", 
                              case.classname.replace('/', "."), 
                              case.test_name),
            status: match case.status {
                TestStatus::Passed => "passed".to_string(),
                TestStatus::Failed => "failed".to_string(),
                TestStatus::Skipped => "skipped".to_string(),
            },
            time: AllureTime {
                start: start_ms,
                stop: stop_ms,
                duration: duration_ms,
            },
            description: Some(format!("MySQL test case: {}", case.test_name)),
            description_html: None,
            stage: "finished".to_string(),
            steps: self.generate_test_steps(case),
            attachments,
            parameters: Vec::new(),
            status_details: if case.status == TestStatus::Failed {
                Some(AllureStatusDetails {
                    known: false,
                    muted: false,
                    flaky: false,
                    message: case.errors.first().cloned(),
                    trace: if case.errors.is_empty() { 
                        None 
                    } else { 
                        Some(case.errors.join("\n")) 
                    },
                })
            } else {
                None
            },
            labels: vec![
                AllureLabel { name: "suite".to_string(), value: "mysql-test".to_string() },
                AllureLabel { name: "testClass".to_string(), value: format!("mysql-test.{}", case.classname.replace('/', ".")) },
                AllureLabel { name: "testMethod".to_string(), value: case.test_name.clone() },
                AllureLabel { name: "framework".to_string(), value: "mysql-tester-rs".to_string() },
                AllureLabel { name: "language".to_string(), value: "rust".to_string() },
                AllureLabel { name: "host".to_string(), value: suite.environment.os.clone() },
            ],
            links: Vec::new(),
        };
        
        let json_content = serde_json::to_string_pretty(&result)?;
        fs::write(result_path, json_content)?;
        
        Ok(())
    }
    
    /// Parse start and end timestamps to milliseconds since epoch
    fn parse_timestamps(&self, case: &TestResult) -> Result<(u64, u64, u64)> {
        let duration_ms = case.duration_ms;
        
        // If we have proper timestamps, parse them
        if !case.start_time.is_empty() && !case.end_time.is_empty() {
            let start_time = chrono::DateTime::parse_from_rfc3339(&case.start_time)
                .map_err(|_| anyhow::anyhow!("Invalid start time format"))?;
            let end_time = chrono::DateTime::parse_from_rfc3339(&case.end_time)
                .map_err(|_| anyhow::anyhow!("Invalid end time format"))?;
            
            let start_ms = start_time.timestamp_millis() as u64;
            let stop_ms = end_time.timestamp_millis() as u64;
            
            Ok((start_ms, stop_ms, duration_ms))
        } else {
            // Fallback: use current time and duration
            let now = chrono::Utc::now().timestamp_millis() as u64;
            let start_ms = now - duration_ms;
            Ok((start_ms, now, duration_ms))
        }
    }
    
    /// Generate test execution steps for better visibility
    fn generate_test_steps(&self, case: &TestResult) -> Vec<AllureStep> {
        let mut steps = Vec::new();
        
        // Add a step for test execution summary
        steps.push(AllureStep {
            name: format!("Test Execution Summary ({})", case.test_name),
            status: match case.status {
                TestStatus::Passed => "passed".to_string(),
                TestStatus::Failed => "failed".to_string(),
                TestStatus::Skipped => "skipped".to_string(),
            },
            time: AllureTime {
                start: 0,
                stop: case.duration_ms,
                duration: case.duration_ms,
            },
            status_details: if case.status == TestStatus::Failed {
                Some(AllureStatusDetails {
                    known: false,
                    muted: false,
                    flaky: false,
                    message: Some(format!("Test failed with {} query failure(s)", case.failed_queries)),
                    trace: Some(format!(
                        "Passed: {} queries\nFailed: {} queries\nDuration: {} ms\n\nFirst Error: {}",
                        case.passed_queries,
                        case.failed_queries, 
                        case.duration_ms,
                        case.errors.first().unwrap_or(&"No error details".to_string())
                    )),
                })
            } else {
                Some(AllureStatusDetails {
                    known: false,
                    muted: false,
                    flaky: false,
                    message: Some(format!("Test passed with {} queries executed", case.passed_queries)),
                    trace: None,
                })
            },
            stage: "finished".to_string(),
            steps: Vec::new(),
            attachments: Vec::new(),
            parameters: Vec::new(),
        });
        
        // Add detailed steps for each query failure if available
        for (i, failure) in case.query_failures.iter().enumerate() {
            steps.push(AllureStep {
                name: format!("Query Failure #{} (Line {})", i + 1, failure.line_number),
                status: "failed".to_string(),
                time: AllureTime {
                    start: 0,
                    stop: 0,
                    duration: 0,
                },
                status_details: Some(AllureStatusDetails {
                    known: false,
                    muted: false,
                    flaky: false,
                    message: Some(format!("SQL query failed at line {}", failure.line_number)),
                    trace: Some(format!(
                        "Line Number: {}\n\nSQL Query:\n{}\n\nError Message:\n{}\n{}",
                        failure.line_number,
                        failure.sql,
                        failure.error_message,
                        if !failure.expected.is_empty() || !failure.actual.is_empty() {
                            format!("\nResult Comparison:\nExpected: {}\nActual:   {}", 
                                   failure.expected, failure.actual)
                        } else {
                            String::new()
                        }
                    )),
                }),
                stage: "finished".to_string(),
                steps: Vec::new(),
                attachments: Vec::new(),
                parameters: Vec::new(),
            });
        }
        
        steps
    }
    
    /// Generate attachments for test case (test file content, failure details, etc.)
    fn generate_attachments(&self, case: &TestResult, test_uuid: &str) -> Result<Vec<AllureAttachment>> {
        let mut attachments = Vec::new();
        
        // Always attach test file content (without line numbers for readability)
        if let Ok(test_file_content) = self.read_test_file_content(&case.test_name) {
            let test_file_attachment = format!("{}-test-file.test", test_uuid);
            let test_file_path = Path::new(&self.output_dir).join(&test_file_attachment);
            fs::write(&test_file_path, &test_file_content)?;
            
            attachments.push(AllureAttachment {
                name: "Test File Content".to_string(),
                source: test_file_attachment,
                attachment_type: "text/plain".to_string(),
            });
        }
        
        // Add expected result file with line numbers if available
        if let Ok(result_file_content) = self.read_result_file_content(&case.test_name) {
            // Add line numbers to result file content for easier reference
            let numbered_result_content = self.add_line_numbers_to_content(&result_file_content);
            let result_file_attachment = format!("{}-expected-result-with-lines.result", test_uuid);
            let result_file_path = Path::new(&self.output_dir).join(&result_file_attachment);
            fs::write(&result_file_path, &numbered_result_content)?;
            
            attachments.push(AllureAttachment {
                name: "Expected Result File (with line numbers)".to_string(),
                source: result_file_attachment,
                attachment_type: "text/plain".to_string(),
            });
        }
        
        // Add failure details for failed tests
        if case.status == TestStatus::Failed && !case.query_failures.is_empty() {
            let failure_details = self.generate_failure_details_content(case);
            let failure_attachment = format!("{}-failure-details.txt", test_uuid);
            let failure_path = Path::new(&self.output_dir).join(&failure_attachment);
            fs::write(&failure_path, failure_details)?;
            
            attachments.push(AllureAttachment {
                name: "Query Failure Details".to_string(),
                source: failure_attachment,
                attachment_type: "text/plain".to_string(),
            });
        }
        
        // Add stdout/stderr if available
        if !case.stdout.is_empty() {
            let stdout_attachment = format!("{}-stdout.txt", test_uuid);
            let stdout_path = Path::new(&self.output_dir).join(&stdout_attachment);
            fs::write(&stdout_path, &case.stdout)?;
            
            attachments.push(AllureAttachment {
                name: "Standard Output".to_string(),
                source: stdout_attachment,
                attachment_type: "text/plain".to_string(),
            });
        }
        
        if !case.stderr.is_empty() {
            let stderr_attachment = format!("{}-stderr.txt", test_uuid);
            let stderr_path = Path::new(&self.output_dir).join(&stderr_attachment);
            fs::write(&stderr_path, &case.stderr)?;
            
            attachments.push(AllureAttachment {
                name: "Standard Error".to_string(),
                source: stderr_attachment,
                attachment_type: "text/plain".to_string(),
            });
        }
        
        Ok(attachments)
    }
    
    /// Add line numbers to file content
    fn add_line_numbers_to_content(&self, content: &str) -> String {
        content
            .lines()
            .enumerate()
            .map(|(i, line)| format!("{:3}: {}", i + 1, line))
            .collect::<Vec<_>>()
            .join("\n")
    }
    
    /// Read test file content for attachment
    fn read_test_file_content(&self, test_name: &str) -> Result<String> {
        // Try different possible paths for the test file
        let possible_paths = vec![
            format!("t/{}.test", test_name),
            format!("t/{}", test_name),
            format!("{}.test", test_name),
            test_name.to_string(),
        ];
        
        for path in possible_paths {
            if let Ok(content) = fs::read_to_string(&path) {
                return Ok(content);
            }
        }
        
        Err(anyhow::anyhow!("Could not find test file for: {}", test_name))
    }
    
    /// Read result file content for attachment
    fn read_result_file_content(&self, test_name: &str) -> Result<String> {
        // Try different possible paths for the result file
        let possible_paths = vec![
            format!("r/{}.result", test_name),
            format!("r/{}", test_name.replace(".test", ".result")),
            format!("{}.result", test_name.replace(".test", "")),
            format!("result/{}.result", test_name),
        ];
        
        for path in possible_paths {
            if let Ok(content) = fs::read_to_string(&path) {
                return Ok(content);
            }
        }
        
        Err(anyhow::anyhow!("Could not find result file for: {}", test_name))
    }
    
    /// Generate detailed failure content for attachment
    fn generate_failure_details_content(&self, case: &TestResult) -> String {
        let mut content = String::new();
        
        content.push_str(&format!("Test Failure Analysis for: {}\n", case.test_name));
        content.push_str(&format!("{}\n\n", "=".repeat(80)));
        
        // General test information
        content.push_str("📊 Test Statistics:\n");
        content.push_str(&format!("  • Duration: {} ms\n", case.duration_ms));
        content.push_str(&format!("  • Passed Queries: {}\n", case.passed_queries));
        content.push_str(&format!("  • Failed Queries: {}\n", case.failed_queries));
        content.push_str(&format!("  • Total Errors: {}\n\n", case.errors.len()));
        
        // Error messages
        if !case.errors.is_empty() {
            content.push_str("❌ Error Summary:\n");
            for (i, error) in case.errors.iter().enumerate() {
                content.push_str(&format!("  {}. {}\n", i + 1, error));
            }
            content.push_str("\n");
        }
        
        // Detailed query failures
        if !case.query_failures.is_empty() {
            content.push_str("🔍 Detailed Query Failure Analysis:\n");
            content.push_str(&format!("{}\n", "-".repeat(60)));
            
            for (i, failure) in case.query_failures.iter().enumerate() {
                content.push_str(&format!("\n🚫 Failure #{} Details:\n", i + 1));
                content.push_str(&format!("   📍 Location: Line {} in test file\n", failure.line_number));
                content.push_str(&format!("   📝 SQL Query:\n      {}\n", failure.sql));
                content.push_str(&format!("   ⚠️  Error Message:\n      {}\n", failure.error_message));
                
                if !failure.expected.is_empty() || !failure.actual.is_empty() {
                    content.push_str("\n   📋 Result Comparison:\n");
                    content.push_str(&format!("      Expected: {}\n", 
                        if failure.expected.is_empty() { "(empty)" } else { &failure.expected }));
                    content.push_str(&format!("      Actual:   {}\n", 
                        if failure.actual.is_empty() { "(empty)" } else { &failure.actual }));
                }
                
                if i < case.query_failures.len() - 1 {
                    content.push_str(&format!("\n{}\n", "-".repeat(40)));
                }
            }
            
            content.push_str("\n");
        }
        
        // Instructions for debugging
        content.push_str("🔧 Debugging Tips:\n");
        content.push_str("  1. Check the 'Test File Content' attachment to see the exact test content\n");
        content.push_str("  2. Review the 'Expected Result File (with line numbers)' attachment to find the problematic line\n");
        content.push_str("  3. The error shows result file line numbers - use them to locate issues in the expected result\n");
        content.push_str("  4. Check if the database schema or data setup matches the test expectations\n\n");
        
        content.push_str(&format!("Generated at: {}\n", chrono::Utc::now().to_rfc3339()));
        
        content
    }
}

/// Allure test result JSON structure
#[derive(Serialize, Deserialize)]
struct AllureTestResult {
    uuid: String,
    name: String,
    #[serde(rename = "fullName")]
    full_name: String,
    status: String,
    time: AllureTime,
    description: Option<String>,
    #[serde(rename = "descriptionHtml")]
    description_html: Option<String>,
    stage: String,
    steps: Vec<AllureStep>,
    attachments: Vec<AllureAttachment>,
    parameters: Vec<AllureParameter>,
    #[serde(rename = "statusDetails")]
    status_details: Option<AllureStatusDetails>,
    labels: Vec<AllureLabel>,
    links: Vec<AllureLink>,
}

/// Allure time information
#[derive(Serialize, Deserialize)]
struct AllureTime {
    start: u64,
    stop: u64,
    duration: u64,
}

/// Allure test step
#[derive(Serialize, Deserialize)]
struct AllureStep {
    name: String,
    status: String,
    time: AllureTime,
    #[serde(rename = "statusDetails")]
    status_details: Option<AllureStatusDetails>,
    stage: String,
    steps: Vec<AllureStep>,
    attachments: Vec<AllureAttachment>,
    parameters: Vec<AllureParameter>,
}

/// Allure status details for failures
#[derive(Serialize, Deserialize)]
struct AllureStatusDetails {
    known: bool,
    muted: bool,
    flaky: bool,
    message: Option<String>,
    trace: Option<String>,
}

/// Allure label for categorization
#[derive(Serialize, Deserialize)]
struct AllureLabel {
    name: String,
    value: String,
}

/// Allure attachment (for files)
#[derive(Serialize, Deserialize)]
struct AllureAttachment {
    name: String,
    source: String,
    #[serde(rename = "type")]
    attachment_type: String,
}

/// Allure parameter
#[derive(Serialize, Deserialize)]
struct AllureParameter {
    name: String,
    value: String,
}

/// Allure link
#[derive(Serialize, Deserialize)]
struct AllureLink {
    name: String,
    url: String,
    #[serde(rename = "type")]
    link_type: String,
} 