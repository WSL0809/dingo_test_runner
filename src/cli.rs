//! Command line interface for mysql-tester
//! 
//! This module defines all CLI arguments compatible with the Go version.

use clap::Parser;

#[derive(Parser, Debug, Clone)]
#[command(name = "mysql-tester")]
#[command(about = "A MySQL test runner written in Rust")]
#[command(version = "1.0.0")]
pub struct Args {
    /// The host of the TiDB/MySQL server
    #[arg(long, default_value = "127.0.0.1")]
    pub host: String,

    /// The listen port of TiDB/MySQL server
    #[arg(long, default_value = "3306")]
    pub port: String,

    /// The user for connecting to the database
    #[arg(long, default_value = "root")]
    pub user: String,

    /// The password for the user
    #[arg(long, default_value = "")]
    pub passwd: String,

    /// The log level of mysql-tester: info, warn, error, debug
    #[arg(long = "log-level", default_value = "error")]
    pub log_level: String,

    /// Whether to record the test output to the result file
    #[arg(long)]
    pub record: bool,

    /// Additional params pass as DSN (e.g. session variable)
    #[arg(long, default_value = "")]
    pub params: String,

    /// Run all tests
    #[arg(long)]
    pub all: bool,

    /// Reserve schema after each test
    #[arg(long = "reserve-schema")]
    pub reserve_schema: bool,

    /// The xml file path to record testing results
    #[arg(long = "xunitfile", default_value = "")]
    pub xunit_file: String,

    /// The max number to retry to connect to the database
    #[arg(long = "retry-connection-count", default_value = "120")]
    pub retry_conn_count: i32,

    /// If --error ERR does not match, return error instead of just warn
    #[arg(long = "check-error")]
    pub check_err: bool,

    /// Run collation related-test with new-collation disabled
    #[arg(long = "collation-disable")]
    pub collation_disable: bool,

    /// The result file extension for result file
    #[arg(long, default_value = "result")]
    pub extension: String,

    /// Database type: mysql or sqlite
    #[arg(long, default_value = "mysql")]
    pub database_type: String,

    /// SQLite database file path (used when database_type is sqlite)
    #[arg(long, default_value = ":memory:")]
    pub sqlite_file: String,

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

    /// Get database connection string
    pub fn get_dsn(&self) -> String {
        let mut dsn = format!("mysql://{}:{}@{}:{}/", 
            self.user, self.passwd, self.host, self.port);
        
        if !self.params.is_empty() {
            dsn.push('?');
            dsn.push_str(&self.params);
        }
        
        dsn
    }

    /// Validate arguments
    pub fn validate(&self) -> Result<(), String> {
        // 验证端口号
        if let Err(_) = self.port.parse::<u16>() {
            return Err(format!("Invalid port: {}", self.port));
        }

        // 验证日志级别
        match self.log_level.to_lowercase().as_str() {
            "error" | "warn" | "info" | "debug" | "trace" => {},
            _ => return Err(format!("Invalid log level: {}", self.log_level)),
        }

        // 验证邮件配置
        if self.email_enable {
            if self.email_smtp_host.is_empty() {
                return Err("Email SMTP host is required when email is enabled".to_string());
            }
            if self.email_to.is_empty() {
                return Err("Email recipient is required when email is enabled".to_string());
            }
        }

        // 验证测试文件
        if !self.all && self.test_files.is_empty() {
            return Err("Either --all flag or test files must be specified".to_string());
        }

        Ok(())
    }
}
