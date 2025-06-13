//! Command line interface for mysql-tester
//! 
//! This module defines all CLI arguments compatible with the Go version.

use clap::Parser;
use anyhow::{anyhow, Result};

/// MySQL Test Runner (Rust) - A MySQL testing framework
#[derive(Parser, Debug, Clone)]
#[command(name = "mysql-tester")]
#[command(about = "A MySQL testing framework written in Rust")]
#[command(version = "0.2.0")]
pub struct Args {
    /// MySQL server host
    #[arg(long, default_value = "127.0.0.1")]
    pub host: String,

    /// MySQL server port
    #[arg(long, default_value = "3306")]
    pub port: String,

    /// Database username
    #[arg(long, default_value = "root")]
    pub user: String,

    /// Database password
    #[arg(long, default_value = "")]
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

    /// Maximum number of connection retry attempts
    #[arg(long, default_value = "120")]
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
    pub test_files: Vec<String>,
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
            return Err(anyhow!("No test files specified. Use --all to run all tests or specify test files."));
        }

        Ok(())
    }
}
