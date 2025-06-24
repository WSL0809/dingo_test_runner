//! Command line interface for mysql-tester
//!
//! This module defines all CLI arguments compatible with the Go version.

use anyhow::{anyhow, Result};
use clap::Parser;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// MySQL Test Runner (Rust) - A MySQL testing framework
#[derive(Parser, Debug, Clone)]
#[command(name = "mysql-tester")]
#[command(about = "A MySQL testing framework written in Rust")]
#[command(version = "0.2.0")]
pub struct Args {
    /// MySQL server host
    #[arg(long, default_value = "172.30.14.172")]
    pub host: String,

    /// MySQL server port
    #[arg(long, default_value = "3307")]
    pub port: String,

    /// Database username
    #[arg(long, default_value = "root")]
    pub user: String,

    /// Database password
    #[arg(long, default_value = "123123")]
    pub passwd: String,

    /// Log level: error, warn, info, debug, trace
    #[arg(long, default_value = "error")]
    pub log_level: String,

    /// Record test output to result files
    #[arg(long)]
    pub record: bool,

    /// Additional database connection parameters
    #[arg(long, default_value = "")]
    pub params: String,

    /// Run all tests in the t/ directory
    #[arg(long)]
    pub all: bool,

    /// Reserve schema after each test (don't cleanup)
    #[arg(long)]
    pub reserve_schema: bool,

    /// Path to write JUnit XML test results
    #[arg(long, default_value = "")]
    pub xunit_file: String,

    /// Report output format: terminal, html, plain, xunit, allure
    #[arg(long, default_value = "terminal")]
    pub report_format: String,

    /// Path to write Allure JSON results (enables Allure format)
    #[arg(long, default_value = "")]
    pub allure_dir: String,

    /// Maximum number of connection retry attempts
    #[arg(long, default_value = "3")]
    pub retry_conn_count: i32,

    /// Return error instead of warning when --error directive doesn't match
    #[arg(long)]
    pub check_err: bool,

    /// Disable collation-related tests
    #[arg(long)]
    pub collation_disable: bool,

    /// Result file extension
    #[arg(long, default_value = "result")]
    pub extension: String,

    // 邮件相关参数
    /// Enable email notification for test results
    #[arg(long = "email-enable")]
    pub email_enable: bool,

    /// SMTP server host for email notification
    #[arg(long = "email-smtp-host", default_value = "")]
    pub email_smtp_host: String,

    /// SMTP server port for email notification
    #[arg(long = "email-smtp-port", default_value = "587")]
    pub email_smtp_port: i32,

    /// Email username for SMTP authentication
    #[arg(long = "email-username", default_value = "")]
    pub email_username: String,

    /// Email password or app password for SMTP authentication
    #[arg(long = "email-password", default_value = "")]
    pub email_password: String,

    /// Sender name for email notification
    #[arg(long = "email-from", default_value = "MySQL Tester")]
    pub email_from: String,

    /// Recipient email addresses (comma separated)
    #[arg(long = "email-to", default_value = "")]
    pub email_to: String,

    /// Enable TLS for SMTP connection
    #[arg(long = "email-enable-tls")]
    pub email_enable_tls: bool,

    /// Test files to run (positional arguments)
    /// Supports: directories, test names (xxx), full file names (xxx.test), or file paths
    pub test_files: Vec<String>,

    /// 出现首个 ERROR 立即终止当前测试并返回失败
    #[arg(long, default_value = "true")]
    pub fail_fast: bool,
}

/// Represents a resolved test input
#[derive(Debug, Clone)]
pub struct ResolvedTest {
    /// The test name (without .test extension)
    pub name: String,
    /// The full path to the test file
    pub path: PathBuf,
}

/// Email configuration for test result notifications
#[derive(Debug, Clone)]
pub struct EmailConfig {
    pub smtp_host: String,
    pub smtp_port: u16,
    pub username: String,
    pub password: String,
    pub from: String,
    pub to: Vec<String>,
    pub enable_tls: bool,
    pub subject: String,
    pub attach_xunit: bool,
}

impl Args {
    /// Parse command line arguments
    pub fn parse_args() -> Self {
        Args::parse()
    }

    /// Validate the parsed arguments
    pub fn validate(&self) -> Result<()> {
        // Validate port
        if self.port.parse::<u16>().is_err() {
            return Err(anyhow!("Invalid port: {}", self.port));
        }

        // Validate log level
        match self.log_level.to_lowercase().as_str() {
            "error" | "warn" | "info" | "debug" | "trace" => {}
            _ => return Err(anyhow!("Invalid log level: {}", self.log_level)),
        }

        // Validate retry count
        if self.retry_conn_count < 1 {
            return Err(anyhow!("Retry connection count must be at least 1"));
        }

        // Validate test files when not using --all
        if !self.all && self.test_files.is_empty() {
            return Err(anyhow!(
                "No test files specified. Use --all to run all tests or specify test files."
            ));
        }

        // Validate email configuration
        self.validate_email_config()?;

        Ok(())
    }

    /// Resolve test inputs to actual test files
    /// Supports:
    /// - Directory paths: runs all .test files in the directory
    /// - Test names (without extension): looks for xxx.test in t/ directory
    /// - Full test file names: xxx.test (looks in t/ directory or uses absolute path)
    /// - Absolute/relative file paths: direct file access
    pub fn resolve_test_inputs(&self) -> Result<Vec<ResolvedTest>> {
        let mut resolved_tests = Vec::new();

        for input in &self.test_files {
            let mut found_tests = self.resolve_single_input(input)?;
            resolved_tests.append(&mut found_tests);
        }

        // Remove duplicates while preserving order
        let mut unique_tests = Vec::new();
        for test in resolved_tests {
            if !unique_tests
                .iter()
                .any(|t: &ResolvedTest| t.path == test.path)
            {
                unique_tests.push(test);
            }
        }

        Ok(unique_tests)
    }

    /// Resolve a single input to one or more test files
    fn resolve_single_input(&self, input: &str) -> Result<Vec<ResolvedTest>> {
        let input_path = Path::new(input);

        // Case 1: Input is a directory
        if input_path.is_dir() {
            return self.resolve_directory(input_path);
        }

        // Case 2: Input is an existing file (absolute or relative path)
        if input_path.is_file() {
            return self.resolve_file(input_path);
        }

        // Case 3: Try to resolve as test name or partial path
        self.resolve_test_name_or_partial_path(input)
    }

    /// Resolve a directory to all .test files within it
    fn resolve_directory(&self, dir_path: &Path) -> Result<Vec<ResolvedTest>> {
        let mut tests = Vec::new();

        for entry in WalkDir::new(dir_path)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|e| !e.file_type().is_dir())
        {
            let path = entry.path();
            if path.extension().map_or(false, |ext| ext == "test") {
                let test_name = self.path_to_test_name(path, Some(dir_path))?;
                tests.push(ResolvedTest {
                    name: test_name,
                    path: path.to_path_buf(),
                });
            }
        }

        if tests.is_empty() {
            return Err(anyhow!(
                "No .test files found in directory: {}",
                dir_path.display()
            ));
        }

        // Sort tests by name for consistent ordering
        tests.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(tests)
    }

    /// Resolve a file path to a test
    fn resolve_file(&self, file_path: &Path) -> Result<Vec<ResolvedTest>> {
        if !file_path.extension().map_or(false, |ext| ext == "test") {
            return Err(anyhow!(
                "File must have .test extension: {}",
                file_path.display()
            ));
        }

        let test_name = self.path_to_test_name(file_path, None)?;
        Ok(vec![ResolvedTest {
            name: test_name,
            path: file_path.to_path_buf(),
        }])
    }

    /// Resolve a test name or partial path
    fn resolve_test_name_or_partial_path(&self, input: &str) -> Result<Vec<ResolvedTest>> {
        // Strategy 1: Try input as test name in t/ directory
        let test_dir = Path::new("t");

        // Remove .test extension if present
        let clean_input = if input.ends_with(".test") {
            &input[..input.len() - 5]
        } else {
            input
        };

        // Try exact match in t/ directory
        let test_file_path = test_dir.join(format!("{}.test", clean_input));
        if test_file_path.is_file() {
            return Ok(vec![ResolvedTest {
                name: clean_input.to_string(),
                path: test_file_path,
            }]);
        }

        // Strategy 2: Try as relative path with .test extension
        let with_extension = if input.ends_with(".test") {
            input.to_string()
        } else {
            format!("{}.test", input)
        };

        let relative_path = Path::new(&with_extension);
        if relative_path.is_file() {
            let test_name = self.path_to_test_name(relative_path, None)?;
            return Ok(vec![ResolvedTest {
                name: test_name,
                path: relative_path.to_path_buf(),
            }]);
        }

        // Strategy 3: Search for partial matches in t/ directory
        if test_dir.is_dir() {
            let mut matches = Vec::new();

            for entry in WalkDir::new(test_dir)
                .into_iter()
                .filter_map(Result::ok)
                .filter(|e| !e.file_type().is_dir())
            {
                let path = entry.path();
                if path.extension().map_or(false, |ext| ext == "test") {
                    if let Ok(relative_path) = path.strip_prefix(test_dir) {
                        let path_without_ext = relative_path.with_extension("");
                        let test_name = path_without_ext.to_string_lossy();
                        if test_name.contains(clean_input) {
                            matches.push(ResolvedTest {
                                name: test_name.to_string(),
                                path: path.to_path_buf(),
                            });
                        }
                    }
                }
            }

            if !matches.is_empty() {
                matches.sort_by(|a, b| a.name.cmp(&b.name));
                return Ok(matches);
            }
        }

        Err(anyhow!(
            "Could not resolve test input: '{}'\n\
            Tried:\n\
            - Test name in t/ directory: t/{}.test\n\
            - Relative file path: {}\n\
            - Partial match search in t/ directory\n\
            \n\
            Hint: Use one of these formats:\n\
            - Test name: 'basic' (looks for t/basic.test)\n\
            - Directory: 't/feature' (runs all .test files in directory)\n\
            - Full file name: 'basic.test' (looks for t/basic.test)\n\
            - File path: 'path/to/test.test' (direct file access)",
            input,
            clean_input,
            with_extension
        ))
    }

    /// Convert file path to test name
    fn path_to_test_name(&self, path: &Path, base_dir: Option<&Path>) -> Result<String> {
        let path_without_ext = path.with_extension("");

        if let Some(base) = base_dir {
            if let Ok(relative) = path_without_ext.strip_prefix(base) {
                return Ok(relative.to_string_lossy().to_string());
            }
        }

        // Try to strip common test directory prefixes
        let test_dir = Path::new("t");
        if let Ok(relative) = path_without_ext.strip_prefix(test_dir) {
            return Ok(relative.to_string_lossy().to_string());
        }

        // Use the file stem as test name
        let file_name = path_without_ext.file_name().unwrap_or_default();
        Ok(file_name.to_string_lossy().to_string())
    }

    /// Get email configuration if email is enabled
    pub fn get_email_config(&self) -> Option<EmailConfig> {
        if !self.email_enable || self.email_smtp_host.is_empty() || self.email_to.is_empty() {
            return None;
        }

        let to_addresses: Vec<String> = self
            .email_to
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        if to_addresses.is_empty() {
            return None;
        }

        let subject = if self.email_to.contains("Test Report") {
            format!(
                "MySQL Test Report - {}",
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S")
            )
        } else {
            "MySQL Test Report".to_string()
        };

        Some(EmailConfig {
            smtp_host: self.email_smtp_host.clone(),
            smtp_port: self.email_smtp_port as u16,
            username: self.email_username.clone(),
            password: self.email_password.clone(),
            from: if self.email_from.is_empty() {
                self.email_username.clone()
            } else {
                self.email_from.clone()
            },
            to: to_addresses,
            enable_tls: self.email_enable_tls,
            subject,
            attach_xunit: !self.xunit_file.is_empty(),
        })
    }

    /// Validate email configuration
    pub fn validate_email_config(&self) -> Result<()> {
        if !self.email_enable {
            return Ok(());
        }

        if self.email_smtp_host.is_empty() {
            return Err(anyhow!("SMTP host is required when email is enabled"));
        }

        if self.email_username.is_empty() {
            return Err(anyhow!("Email username is required when email is enabled"));
        }

        if self.email_password.is_empty() {
            return Err(anyhow!("Email password is required when email is enabled"));
        }

        if self.email_to.is_empty() {
            return Err(anyhow!(
                "Email recipients are required when email is enabled"
            ));
        }

        // Validate email addresses format (basic validation)
        for addr in self.email_to.split(',') {
            let addr = addr.trim();
            if !addr.contains('@') || addr.len() < 5 {
                return Err(anyhow!("Invalid email address: {}", addr));
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    /// Helper function to create a test Args instance
    fn create_test_args(test_files: Vec<String>) -> Args {
        Args {
            host: "localhost".to_string(),
            port: "3306".to_string(),
            user: "root".to_string(),
            passwd: "123456".to_string(),
            log_level: "error".to_string(),
            record: false,
            params: "".to_string(),
            all: false,
            reserve_schema: false,
            xunit_file: "".to_string(),
            report_format: "terminal".to_string(),
            retry_conn_count: 3,
            check_err: false,
            collation_disable: false,
            extension: "result".to_string(),
            email_enable: false,
            email_smtp_host: "".to_string(),
            email_smtp_port: 587,
            email_username: "".to_string(),
            email_password: "".to_string(),
            email_from: "MySQL Tester".to_string(),
            email_to: "".to_string(),
            email_enable_tls: false,
            test_files,
            fail_fast: true,
            allure_dir: "".to_string(),
        }
    }

    #[test]
    fn test_resolve_test_name() {
        // Create a temporary directory structure
        let temp_dir = tempdir().unwrap();
        let test_dir = temp_dir.path().join("t");
        fs::create_dir(&test_dir).unwrap();

        // Create test files
        fs::write(test_dir.join("basic.test"), "# Basic test\nSELECT 1;").unwrap();
        fs::write(test_dir.join("advanced.test"), "# Advanced test\nSELECT 2;").unwrap();

        // Change to temp directory
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(&temp_dir).unwrap();

        let args = create_test_args(vec!["basic".to_string()]);
        let resolved = args.resolve_test_inputs().unwrap();

        assert_eq!(resolved.len(), 1);
        assert_eq!(resolved[0].name, "basic");
        assert!(resolved[0].path.ends_with("t/basic.test"));

        // Restore original directory
        std::env::set_current_dir(original_dir).unwrap();
    }

    #[test]
    fn test_resolve_test_with_extension() {
        // Create a temporary directory structure
        let temp_dir = tempdir().unwrap();
        let test_dir = temp_dir.path().join("t");
        fs::create_dir(&test_dir).unwrap();

        // Create test files
        fs::write(test_dir.join("basic.test"), "# Basic test\nSELECT 1;").unwrap();

        // Change to temp directory
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(&temp_dir).unwrap();

        let args = create_test_args(vec!["basic.test".to_string()]);
        let resolved = args.resolve_test_inputs().unwrap();

        assert_eq!(resolved.len(), 1);
        assert_eq!(resolved[0].name, "basic");
        assert!(resolved[0].path.ends_with("t/basic.test"));

        // Restore original directory
        std::env::set_current_dir(original_dir).unwrap();
    }

    #[test]
    fn test_resolve_directory() {
        // Create a temporary directory structure
        let temp_dir = tempdir().unwrap();
        let feature_dir = temp_dir.path().join("feature");
        fs::create_dir_all(&feature_dir).unwrap();

        // Create test files in the feature directory
        fs::write(
            feature_dir.join("advanced.test"),
            "# Advanced test\nSELECT 2;",
        )
        .unwrap();
        fs::write(
            feature_dir.join("complex.test"),
            "# Complex test\nSELECT 3;",
        )
        .unwrap();

        let args = create_test_args(vec![feature_dir.to_string_lossy().to_string()]);
        let resolved = args.resolve_test_inputs().unwrap();

        assert_eq!(resolved.len(), 2); // advanced.test and complex.test

        // Tests should be sorted by name
        assert_eq!(resolved[0].name, "advanced");
        assert_eq!(resolved[1].name, "complex");
    }

    #[test]
    fn test_resolve_multiple_inputs() {
        // Create a temporary directory structure
        let temp_dir = tempdir().unwrap();
        let test_file_1 = temp_dir.path().join("basic.test");
        let test_file_2 = temp_dir.path().join("advanced.test");

        // Create test files directly in temp directory
        fs::write(&test_file_1, "# Basic test").unwrap();
        fs::write(&test_file_2, "# Advanced test").unwrap();

        let args = create_test_args(vec![
            test_file_1.to_string_lossy().to_string(),
            test_file_2.to_string_lossy().to_string(),
        ]);
        let resolved = args.resolve_test_inputs().unwrap();

        assert_eq!(resolved.len(), 2);
        assert_eq!(resolved[0].name, "basic");
        assert_eq!(resolved[1].name, "advanced");
        assert_eq!(resolved[0].path, test_file_1);
        assert_eq!(resolved[1].path, test_file_2);
    }

    #[test]
    fn test_resolve_nonexistent_input() {
        let args = create_test_args(vec!["nonexistent".to_string()]);
        let result = args.resolve_test_inputs();

        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("Could not resolve test input"));
        assert!(error_msg.contains("Hint: Use one of these formats"));
    }
}
