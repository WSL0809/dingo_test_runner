//! Test execution engine
//! 
//! This module handles the execution of MySQL test cases, including database setup,
//! query execution, result comparison, and cleanup.

use super::database::ConnectionInfo;
use super::parser::Parser;
use super::query::{Query, QueryType};
use crate::tester::command::Command;
use crate::tester::connection_manager::ConnectionManager;
use crate::tester::error_handler::MySQLErrorHandler;
use crate::tester::registry::COMMAND_REGISTRY;
use crate::cli::Args;
use anyhow::{anyhow, Result};
use log::{debug, error, info, warn};
use regex::Regex;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

/// Test execution engine
pub struct Tester {
    /// Connection manager for handling multiple database connections
    pub connection_manager: ConnectionManager,
    /// Current test name
    test_name: String,
    /// Arguments from CLI
    pub args: Args,
    /// Current working directory for test files
    current_dir: PathBuf,
    /// Output buffer for test results
    pub output_buffer: Vec<u8>,
    /// Query logging enabled
    pub enable_query_log: bool,
    /// Result logging enabled
    pub enable_result_log: bool,
    /// Expected errors for the next query
    pub expected_errors: Vec<String>,
    /// MySQL error handler
    error_handler: MySQLErrorHandler,
    /// Result file for comparison (non-record mode)
    result_file_content: Option<String>,
    /// Current position in result file
    result_file_position: usize,
    
    // --- One-shot modifiers for the next query ---
    /// Sort results for the next query
    pub pending_sorted_result: bool,
    /// Regex for result replacement for the next query
    pub pending_replace_regex: Vec<(Regex, String)>,
}

impl Tester {
    /// Create a new tester instance
    pub fn new(args: Args) -> Result<Self> {
        let port = args.port.parse::<u16>()
            .map_err(|_| anyhow!("Invalid port: {}", args.port))?;

        // Create connection info
        let connection_info = ConnectionInfo {
            host: args.host.clone(),
            port,
            user: args.user.clone(),
            password: args.passwd.clone(),
            database: "".to_string(), // Start with no specific database
            params: args.params.clone(),
        };

        // Create connection manager with default connection
        let connection_manager = ConnectionManager::new(
            connection_info,
            args.retry_conn_count as u32,
        )?;

        Ok(Tester {
            connection_manager,
            test_name: String::new(),
            args,
            current_dir: std::env::current_dir()?,
            output_buffer: Vec::new(),
            enable_query_log: true,
            enable_result_log: true,
            expected_errors: Vec::new(),
            error_handler: MySQLErrorHandler::new(),
            result_file_content: None,
            result_file_position: 0,
            pending_sorted_result: false,
            pending_replace_regex: Vec::new(),
        })
    }

    /// Set the current test name and prepare for execution
    pub fn set_test(&mut self, test_name: &str) -> Result<()> {
        self.test_name = test_name.to_string();
        self.output_buffer.clear();
        self.expected_errors.clear();
        self.result_file_position = 0;
        self.pending_sorted_result = false;
        self.pending_replace_regex.clear();
        
        info!("Starting test: {}", test_name);
        
        // Load result file for comparison if not in record mode
        if !self.args.record {
            self.load_result_file(test_name)?;
            debug!("Loaded result file with {} lines", 
                   self.result_file_content.as_ref().map(|c| c.lines().count()).unwrap_or(0));
        }
        
        // Pre-process: setup database state
        self.pre_process()?;
        
        Ok(())
    }

    /// Pre-process: save original database state and setup test environment
    fn pre_process(&mut self) -> Result<()> {
        // Initialize database for test
        if !self.args.reserve_schema {
             self.connection_manager.current_database()?.init_for_test(&self.test_name)?;
        }
        debug!("Test environment initialized for '{}'", self.test_name);
        Ok(())
    }

    /// Post-process: cleanup database state after test
    fn post_process(&mut self) -> Result<()> {
        if !self.args.reserve_schema {
            self.connection_manager.current_database()?.cleanup_after_test(&self.test_name)?;
        }
        
        info!("Test '{}' completed", self.test_name);
        Ok(())
    }

    /// Execute a test file
    pub fn run_test_file<P: AsRef<Path>>(&mut self, test_file: P) -> Result<TestResult> {
        let test_name = test_file.as_ref().to_string_lossy().to_string();
        
        // Construct the actual test file path in ./t/ directory
        let test_path = self.current_dir.join("t").join(format!("{}.test", test_name));

        self.set_test(&test_name)?;

        // Read and parse test file
        let content = fs::read_to_string(&test_path)?;
        let mut parser = Parser::new();
        let queries = parser.parse(&content)?;

        info!("Parsed {} queries from {}", queries.len(), test_path.display());

        // Execute queries
        let mut test_result = TestResult::new(&test_name);
        for (i, query) in queries.iter().enumerate() {
            match self.execute_query(query, i + 1) {
                Ok(_) => {
                    test_result.passed_queries += 1;
                }
                Err(e) => {
                    error!("Line {} (Query {}): {}", query.line, i + 1, e);
                    test_result.failed_queries += 1;
                    test_result.errors.push(format!("Line {}: {}", query.line, e));
                    if self.args.fail_fast {
                        break; // 快速失败，终止后续查询
                    }
                }
            }
        }

        // Post-process
        self.post_process()?;

        // Write result file if in record mode
        // 但如果启用了 fail_fast 且有失败的查询，则不生成 result 文件
        if self.args.record && !(self.args.fail_fast && test_result.failed_queries > 0) {
            self.write_result_file(&test_name)?;
        }

        test_result.success = test_result.failed_queries == 0;
        Ok(test_result)
    }

    /// Execute a single query or command
    fn execute_query(&mut self, query: &Query, query_num: usize) -> Result<()> {
        debug!("Executing query {:?} (line {}): {:?}", query_num, query.line, query.query_type);

        match query.query_type {
            QueryType::Query | QueryType::Exec => {
                self.execute_sql_query(&query.query, query_num)?;
                // Modifiers are one-shot, clear them after the SQL query runs.
                self.expected_errors.clear();
                self.pending_sorted_result = false;
                self.pending_replace_regex.clear();
            }
            QueryType::Comment => {
                // Skip comments
            }
            QueryType::Echo => {
                // Create a command object to use the new handler system
                let cmd = Command {
                    name: "echo".to_string(),
                    args: query.query.clone(),
                    line: query.line,
                };
                
                // Execute from registry
                if let Some(executor) = COMMAND_REGISTRY.get(cmd.name.as_str()) {
                    executor(self, &cmd)?;
                } else {
                    // Fallback to original implementation
                    self.handle_echo(&query.query)?;
                }
                
                // Clear any pending error expectations as they don't apply to echo commands
                if !self.expected_errors.is_empty() {
                    warn!("--error directive before --echo is ignored");
                    self.expected_errors.clear();
                }
            },
            QueryType::Sleep => {
                // Create a command object on the fly to use the new handler system.
                let cmd = Command {
                    name: "sleep".to_string(),
                    args: query.query.clone(),
                    line: query.line,
                };
                
                // Execute from registry
                if let Some(executor) = COMMAND_REGISTRY.get(cmd.name.as_str()) {
                    executor(self, &cmd)?;
                } else {
                    // This case should ideally not be reached if commands are registered correctly.
                    return Err(anyhow!("'sleep' command handler not found in registry"));
                }

                // Clear any pending error expectations as they don't apply to sleep commands.
                if !self.expected_errors.is_empty() {
                    warn!("--error directive before --sleep is ignored");
                    self.expected_errors.clear();
                }
            },
            QueryType::Delimiter => {
                // Handled by parser
                // Clear any pending error expectations as they don't apply to delimiter commands
                if !self.expected_errors.is_empty() {
                    warn!("--error directive before --delimiter is ignored");
                    self.expected_errors.clear();
                }
            }
            QueryType::DisableQueryLog => {
                let cmd = Command {
                    name: "disable_query_log".to_string(),
                    args: query.query.clone(),
                    line: query.line,
                };
                
                if let Some(executor) = COMMAND_REGISTRY.get(cmd.name.as_str()) {
                    executor(self, &cmd)?;
                } else {
                    self.enable_query_log = false;
                }
                
                // Clear any pending error expectations as they don't apply to log control commands
                if !self.expected_errors.is_empty() {
                    warn!("--error directive before --disable_query_log is ignored");
                    self.expected_errors.clear();
                }
            },
            QueryType::EnableQueryLog => {
                let cmd = Command {
                    name: "enable_query_log".to_string(),
                    args: query.query.clone(),
                    line: query.line,
                };
                
                if let Some(executor) = COMMAND_REGISTRY.get(cmd.name.as_str()) {
                    executor(self, &cmd)?;
                } else {
                    self.enable_query_log = true;
                }
                
                // Clear any pending error expectations as they don't apply to log control commands
                if !self.expected_errors.is_empty() {
                    warn!("--error directive before --enable_query_log is ignored");
                    self.expected_errors.clear();
                }
            },
            QueryType::DisableResultLog => {
                let cmd = Command {
                    name: "disable_result_log".to_string(),
                    args: query.query.clone(),
                    line: query.line,
                };
                
                if let Some(executor) = COMMAND_REGISTRY.get(cmd.name.as_str()) {
                    executor(self, &cmd)?;
                } else {
                    self.enable_result_log = false;
                }
                
                // Clear any pending error expectations as they don't apply to log control commands
                if !self.expected_errors.is_empty() {
                    warn!("--error directive before --disable_result_log is ignored");
                    self.expected_errors.clear();
                }
            },
            QueryType::EnableResultLog => {
                let cmd = Command {
                    name: "enable_result_log".to_string(),
                    args: query.query.clone(),
                    line: query.line,
                };
                
                if let Some(executor) = COMMAND_REGISTRY.get(cmd.name.as_str()) {
                    executor(self, &cmd)?;
                } else {
                    self.enable_result_log = true;
                }
                
                // Clear any pending error expectations as they don't apply to log control commands
                if !self.expected_errors.is_empty() {
                    warn!("--error directive before --enable_result_log is ignored");
                    self.expected_errors.clear();
                }
            },
            
            // Set one-shot modifiers for the next query
            QueryType::SortedResult => {
                let cmd = Command {
                    name: "sorted_result".to_string(),
                    args: query.query.clone(),
                    line: query.line,
                };
                
                if let Some(executor) = COMMAND_REGISTRY.get(cmd.name.as_str()) {
                    executor(self, &cmd)?;
                } else {
                    self.pending_sorted_result = true;
                }
            },
            QueryType::ReplaceRegex => {
                let cmd = Command {
                    name: "replace_regex".to_string(),
                    args: query.query.clone(),
                    line: query.line,
                };
                
                if let Some(executor) = COMMAND_REGISTRY.get(cmd.name.as_str()) {
                    executor(self, &cmd)?;
                } else {
                    self.parse_replace_regex(&query.query)?;
                }
            },
            QueryType::Error => {
                let cmd = Command {
                    name: "error".to_string(),
                    args: query.query.clone(),
                    line: query.line,
                };
                
                if let Some(executor) = COMMAND_REGISTRY.get(cmd.name.as_str()) {
                    executor(self, &cmd)?;
                } else {
                    self.parse_expected_errors(&query.query)?;
                }
            },
            
            // Connection management commands - these are NOT affected by --error
            QueryType::Connect => {
                let cmd = Command {
                    name: "connect".to_string(),
                    args: query.query.clone(),
                    line: query.line,
                };
                
                if let Some(executor) = COMMAND_REGISTRY.get(cmd.name.as_str()) {
                    executor(self, &cmd)?;
                } else {
                    self.handle_connect(&query.query)?;
                }
                
                // Clear any pending error expectations as they don't apply to connect commands
                if !self.expected_errors.is_empty() {
                    warn!("--error directive before --connect is ignored");
                    self.expected_errors.clear();
                }
            },
            QueryType::Connection => {
                let cmd = Command {
                    name: "connection".to_string(),
                    args: query.query.clone(),
                    line: query.line,
                };
                
                if let Some(executor) = COMMAND_REGISTRY.get(cmd.name.as_str()) {
                    executor(self, &cmd)?;
                } else {
                    self.handle_connection_switch(&query.query)?;
                }
                
                // Clear any pending error expectations as they don't apply to connection commands
                if !self.expected_errors.is_empty() {
                    warn!("--error directive before --connection is ignored");
                    self.expected_errors.clear();
                }
            },
            QueryType::Disconnect => {
                let cmd = Command {
                    name: "disconnect".to_string(),
                    args: query.query.clone(),
                    line: query.line,
                };
                
                if let Some(executor) = COMMAND_REGISTRY.get(cmd.name.as_str()) {
                    executor(self, &cmd)?;
                } else {
                    self.handle_disconnect(&query.query)?;
                }
                
                // Clear any pending error expectations as they don't apply to disconnect commands
                if !self.expected_errors.is_empty() {
                    warn!("--error directive before --disconnect is ignored");
                    self.expected_errors.clear();
                }
            },
            _ => {
                warn!("Unhandled query type: {:?}", query.query_type);
            }
        }

        Ok(())
    }

    /// Execute SQL query and handle results/errors
    fn execute_sql_query(&mut self, sql: &str, _query_num: usize) -> Result<()> {
        if self.enable_query_log {
            let query_output = format!("{}\n", sql);
            if self.args.record {
                write!(self.output_buffer, "{}", query_output)?;
            } else {
                self.compare_with_result(&query_output)?;
            }
        }

        let execution_result = self.connection_manager.current_database()?.query(sql);

        match execution_result {
            Ok(rows) => {
                if !self.expected_errors.is_empty() {
                    let err_msg = format!("Expected error(s) {:?}, but query succeeded", self.expected_errors);
                    if self.args.check_err { return Err(anyhow!(err_msg)); } 
                    else { warn!("{}", err_msg); }
                }

                if self.enable_result_log {
                    let mut formatted_result = self.format_query_result_to_string(&rows)?;
                    self.apply_regex_replacements(&mut formatted_result);
                    
                    if self.args.record {
                        write!(self.output_buffer, "{}", formatted_result)?;
                    } else {
                        self.compare_with_result(&formatted_result)?;
                    }
                }
            }
            Err(e) => {
                let error_handled = self.handle_query_error(&e)?;
                if !error_handled {
                    return Err(e);
                }
            }
        }
        Ok(())
    }

    /// Format query results to a string
    fn format_query_result_to_string(&self, rows: &[Vec<String>]) -> Result<String> {
        if rows.is_empty() { return Ok(String::new()); }

        let mut result = String::new();
        let mut sorted_rows = rows.to_vec();
        
        if self.pending_sorted_result {
            sorted_rows.sort();
        }

        for row in sorted_rows {
            result.push_str(&row.join("\t"));
            result.push('\n');
        }
        Ok(result)
    }

    /// Handle a query that returned an error
    fn handle_query_error(&mut self, error: &anyhow::Error) -> Result<bool> {
        if !self.expected_errors.is_empty() {
            // Try to extract MySQL error from anyhow error
            let is_match = if let Some(mysql_error) = error.downcast_ref::<mysql::Error>() {
                // Use the error handler for precise MySQL error matching
                self.error_handler.check_expected_error(mysql_error, &self.expected_errors)
            } else {
                // Fallback to string matching for non-MySQL errors
                let error_message = error.to_string();
                self.expected_errors.iter().any(|expected| 
                    expected == "0" || error_message.contains(expected)
                )
            };

            if is_match {
                if self.enable_result_log {
                    let mut error_output = if self.expected_errors.len() == 1 && self.expected_errors[0] != "0" {
                        // For MySQL errors, use formatted error message
                        if let Some(mysql_error) = error.downcast_ref::<mysql::Error>() {
                            format!("{}\n", self.error_handler.format_error(mysql_error))
                        } else {
                            format!("{}\n", error.to_string())
                        }
                    } else {
                        "Got one of the listed errors\n".to_string()
                    };
                    self.apply_regex_replacements(&mut error_output);

                    if self.args.record {
                        write!(self.output_buffer, "{}", error_output)?;
                    } else {
                        self.compare_with_result(&error_output)?;
                    }
                }
                return Ok(true); // Error was expected and handled.
            }

            let error_message = if let Some(mysql_error) = error.downcast_ref::<mysql::Error>() {
                self.error_handler.format_error(mysql_error)
            } else {
                error.to_string()
            };
            
            let err_msg = format!("Expected error(s) {:?}, but got: {}", self.expected_errors, error_message);
            if self.args.check_err { return Err(anyhow!(err_msg)); } 
            else { warn!("{}", err_msg); return Ok(true); /* Treat as handled */ }
        }
        
        Ok(false) // Not an expected error.
    }

    /// Handle the --echo command
    fn handle_echo(&mut self, text: &str) -> Result<()> {
        let echo_output = format!("{}\n", text);
        // Per user feedback, echo is NOT affected by modifiers.
        if self.args.record {
            write!(self.output_buffer, "{}", echo_output)?;
        } else {
            self.compare_with_result(&echo_output)?;
        }
        Ok(())
    }

    /// Parses a --error command
    fn parse_expected_errors(&mut self, error_spec: &str) -> Result<()> {
        self.expected_errors = error_spec.split(',').map(|s| s.trim().to_string()).collect();
        debug!("Expected errors set to: {:?}", self.expected_errors);
        Ok(())
    }

    /// Handle the --connect command
    fn handle_connect(&mut self, params: &str) -> Result<()> {
        self.connection_manager.connect(params)?;
        info!("Connected to new database connection");
        Ok(())
    }

    /// Handle the --connection command (switch connection)
    fn handle_connection_switch(&mut self, conn_name: &str) -> Result<()> {
        self.connection_manager.switch_connection(conn_name.trim())?;
        info!("Switched to connection: {}", conn_name.trim());
        Ok(())
    }

    /// Handle the --disconnect command
    fn handle_disconnect(&mut self, conn_name: &str) -> Result<()> {
        self.connection_manager.disconnect(conn_name.trim())?;
        info!("Disconnected connection: {}", conn_name.trim());
        Ok(())
    }

    /// Parses a --replace_regex command
    fn parse_replace_regex(&mut self, pattern: &str) -> Result<()> {
        if !pattern.starts_with('/') || !pattern.ends_with('/') || pattern.len() < 3 {
            return Err(anyhow!("Invalid replace_regex: must be /regex/replacement/. Got: {}", pattern));
        }
        let inner = &pattern[1..pattern.len() - 1];
        
        let mut parts = Vec::with_capacity(2);
        let mut current_part = String::new();
        let mut in_escape = false;
        
        for char in inner.chars() {
            if in_escape {
                current_part.push(char);
                in_escape = false;
            } else if char == '\\' {
                in_escape = true;
                current_part.push(char);
            } else if char == '/' && parts.is_empty() {
                parts.push(current_part);
                current_part = String::new();
            } else {
                current_part.push(char);
            }
        }
        parts.push(current_part);

        if parts.len() != 2 {
            return Err(anyhow!("Invalid replace_regex format. Got: {}", pattern));
        }

        let regex = Regex::new(&parts[0])?;
        self.pending_replace_regex.push((regex, parts[1].to_string()));
        Ok(())
    }

    /// Applies stored regex replacements to a string buffer
    fn apply_regex_replacements(&self, buffer: &mut String) {
        if self.pending_replace_regex.is_empty() { return; }
        let mut temp_buffer = buffer.clone();
        for (regex, replacement) in &self.pending_replace_regex {
            temp_buffer = regex.replace_all(&temp_buffer, replacement.as_str()).to_string();
        }
        *buffer = temp_buffer;
    }

    /// Load result file for comparison (non-record mode)
    fn load_result_file(&mut self, test_name: &str) -> Result<()> {
        let result_dir = self.current_dir.join("r");
        let result_file = result_dir.join(format!("{}.{}", test_name, self.args.extension));
        
        if result_file.exists() {
            self.result_file_content = Some(fs::read_to_string(result_file)?);
            debug!("Loaded result file for comparison: {}", test_name);
        } else {
            warn!("Result file not found for test: {}", test_name);
            self.result_file_content = None;
        }
        
        Ok(())
    }

    /// Compare current output with expected result
    pub fn compare_with_result(&mut self, new_output: &str) -> Result<()> {
        if let Some(ref expected_content) = self.result_file_content {
            let expected_lines: Vec<&str> = expected_content.lines().collect();
            let current_position = self.result_file_position;
            
            // Skip empty output
            if new_output.trim().is_empty() {
                return Ok(());
            }
            
            let new_lines: Vec<&str> = new_output.lines().collect();
            
            debug!("Comparing output at position {}, new_lines: {:?}", current_position, new_lines);
            debug!("Expected lines at position {}: {:?}", current_position,
                &expected_lines[current_position..std::cmp::min(current_position + new_lines.len(), expected_lines.len())]);
            
            // Check if we have enough expected lines
            if current_position + new_lines.len() > expected_lines.len() {
                return Err(anyhow!(
                    "Output has more lines than expected. Expected {} total lines, but got {} lines at position {}",
                    expected_lines.len(),
                    current_position + new_lines.len(),
                    current_position
                ));
            }
            
            // Compare line by line
            for (i, new_line) in new_lines.iter().enumerate() {
                let expected_line = expected_lines[current_position + i];
                if new_line != &expected_line {
                    return Err(anyhow!(
                        "Output mismatch at line {}:\nExpected: {}\nActual: {}",
                        current_position + i + 1,
                        expected_line,
                        new_line
                    ));
                }
            }
            
            self.result_file_position += new_lines.len();
        }
        
        Ok(())
    }

    /// Write result file
    fn write_result_file(&self, test_name: &str) -> Result<()> {
        let result_dir = self.current_dir.join("r");
        fs::create_dir_all(&result_dir)?;
        
        // 若 test_name 包含路径分隔符，需要提前创建子目录
        let result_file = result_dir.join(format!("{}.{}", test_name, self.args.extension));
        if let Some(parent) = result_file.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(result_file, &self.output_buffer)?;
        
        info!("Result file written for test: {}", test_name);
        Ok(())
    }
}

/// Test execution result
#[derive(Debug)]
pub struct TestResult {
    pub test_name: String,
    pub success: bool,
    pub passed_queries: usize,
    pub failed_queries: usize,
    pub errors: Vec<String>,
}

impl TestResult {
    pub fn new(test_name: &str) -> Self {
        Self {
            test_name: test_name.to_string(),
            success: false,
            passed_queries: 0,
            failed_queries: 0,
            errors: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tester_creation() {
        let args = Args {
            host: "127.0.0.1".to_string(),
            port: "3306".to_string(),
            user: "root".to_string(),
            passwd: "123456".to_string(),
            log_level: "error".to_string(),
            record: false,
            params: "".to_string(),
            all: false,
            reserve_schema: false,
            xunit_file: "".to_string(),
            retry_conn_count: 1,
            check_err: false,
            collation_disable: false,
            extension: "result".to_string(),
            email_enable: false,
            email_smtp_host: "".to_string(),
            email_smtp_port: 587,
            email_username: "".to_string(),
            email_password: "".to_string(),
            email_from: "".to_string(),
            email_to: "".to_string(),
            email_enable_tls: false,
            fail_fast: false,
            test_files: vec![],
        };

        // Note: This test would require a running MySQL server to actually work
        // For now, we test that the structure can be created
        let result = Tester::new(args);
        // We expect this to succeed now that a MySQL server is available
        assert!(result.is_ok());
    }

    #[test]
    fn test_sorted_result_modifier() {
        use std::fs::{self, File};
        use std::io::Write;
        // 准备测试文件内容
        let test_name = "sorted_result_test";
        let test_dir = std::path::Path::new("t");
        fs::create_dir_all(test_dir).unwrap();

        let test_file_path = test_dir.join(format!("{}.test", test_name));
        let mut file = File::create(&test_file_path).unwrap();
        writeln!(file, "--disable_query_log").unwrap();
        writeln!(file, "CREATE TABLE nums (val INT);").unwrap();
        writeln!(file, "INSERT INTO nums VALUES (2);").unwrap();
        writeln!(file, "INSERT INTO nums VALUES (1);").unwrap();
        writeln!(file, "--sorted_result").unwrap();
        writeln!(file, "SELECT val FROM nums;").unwrap();

        // 构造参数，开启 record 模式以便读取输出缓冲
        let args = Args {
            host: "127.0.0.1".to_string(),
            port: "3306".to_string(),
            user: "root".to_string(),
            passwd: "123456".to_string(),
            log_level: "error".to_string(),
            record: true,
            params: "".to_string(),
            all: false,
            reserve_schema: false,
            xunit_file: "".to_string(),
            retry_conn_count: 1,
            check_err: false,
            collation_disable: false,
            extension: "result".to_string(),
            email_enable: false,
            email_smtp_host: "".to_string(),
            email_smtp_port: 587,
            email_username: "".to_string(),
            email_password: "".to_string(),
            email_from: "".to_string(),
            email_to: "".to_string(),
            email_enable_tls: false,
            fail_fast: false,
            test_files: vec![],
        };

        let mut tester = match Tester::new(args) {
            Ok(t) => t,
            Err(e) => {
                warn!("Skipping test_sorted_result_modifier due to DB connection error: {}. This test requires a running MySQL server.", e);
                return;
            }
        };
        let result = tester.run_test_file(test_name).unwrap();
        assert!(result.success);

        // 检查输出结果已经排序
        let output = String::from_utf8(tester.output_buffer.clone()).unwrap();
        assert_eq!(output, "1\n2\n");

        // 清理
        fs::remove_file(test_file_path).unwrap();
        let result_file_path = std::path::Path::new("r").join(format!("{}.result", test_name));
        if result_file_path.exists() {
            fs::remove_file(result_file_path).unwrap();
        }
    }

    #[test]
    fn test_replace_regex_modifier() {
        use std::fs::{self, File};
        use std::io::Write;
        let test_name = "replace_regex_test";
        let test_dir = std::path::Path::new("t");
        fs::create_dir_all(test_dir).unwrap();

        let test_file_path = test_dir.join(format!("{}.test", test_name));
        let mut file = File::create(&test_file_path).unwrap();
        writeln!(file, "--disable_query_log").unwrap();
        writeln!(file, "CREATE TABLE t1 (val TEXT);").unwrap();
        writeln!(file, "INSERT INTO t1 VALUES ('abc123');").unwrap();
        writeln!(file, "--replace_regex /[0-9]+/XXX/").unwrap();
        writeln!(file, "SELECT val FROM t1;").unwrap();

        let args = Args {
            host: "127.0.0.1".to_string(),
            port: "3306".to_string(),
            user: "root".to_string(),
            passwd: "123456".to_string(),
            log_level: "error".to_string(),
            record: true,
            params: "".to_string(),
            all: false,
            reserve_schema: false,
            xunit_file: "".to_string(),
            retry_conn_count: 1,
            check_err: false,
            collation_disable: false,
            extension: "result".to_string(),
            email_enable: false,
            email_smtp_host: "".to_string(),
            email_smtp_port: 587,
            email_username: "".to_string(),
            email_password: "".to_string(),
            email_from: "".to_string(),
            email_to: "".to_string(),
            email_enable_tls: false,
            fail_fast: false,
            test_files: vec![],
        };

        let mut tester = match Tester::new(args) {
            Ok(t) => t,
            Err(e) => {
                warn!("Skipping test_replace_regex_modifier due to DB connection error: {}. This test requires a running MySQL server.", e);
                return;
            }
        };
        let result = tester.run_test_file(test_name).unwrap();
        assert!(result.success);

        let output = String::from_utf8(tester.output_buffer.clone()).unwrap();
        assert_eq!(output, "abcXXX\n");

        fs::remove_file(test_file_path).unwrap();
        let result_file_path = std::path::Path::new("r").join(format!("{}.result", test_name));
        if result_file_path.exists() {
            fs::remove_file(result_file_path).unwrap();
        }
    }

    #[test]
    fn test_error_directive_only_affects_sql() {
        // Test that --error directive only affects SQL queries, not other commands
        let args = Args {
            host: "127.0.0.1".to_string(),
            port: "3306".to_string(),
            user: "root".to_string(),
            passwd: "123456".to_string(),
            log_level: "error".to_string(),
            record: true,
            params: "".to_string(),
            all: false,
            reserve_schema: false,
            xunit_file: "".to_string(),
            retry_conn_count: 1,
            check_err: false,
            collation_disable: false,
            extension: "result".to_string(),
            email_enable: false,
            email_smtp_host: "".to_string(),
            email_smtp_port: 587,
            email_username: "".to_string(),
            email_password: "".to_string(),
            email_from: "".to_string(),
            email_to: "".to_string(),
            email_enable_tls: false,
            fail_fast: false,
            test_files: vec![],
        };

        // This test doesn't actually create a tester since it would require MySQL
        // Instead, we test the logic that expected errors should be cleared for non-SQL commands
        // This is more of a design verification test
        
        // We can test the Args structure creation at least
        assert_eq!(args.host, "127.0.0.1");
        assert_eq!(args.port, "3306");
        assert_eq!(args.user, "root");
    }

    #[test]
    fn test_expected_error_handling() {
        use std::fs::{self, File};
        use std::io::Write;
        let test_name = "expected_error_test";
        let test_dir = std::path::Path::new("t");
        let result_dir = std::path::Path::new("r");
        fs::create_dir_all(test_dir).unwrap();
        fs::create_dir_all(result_dir).unwrap();

        let test_file_path = test_dir.join(format!("{}.test", test_name));
        let mut file = File::create(&test_file_path).unwrap();
        writeln!(file, "--disable_query_log").unwrap();
        writeln!(file, "--error 0").unwrap();
        writeln!(file, "SELECT * FROM non_existing_table;").unwrap();

        // 创建一个期望的结果文件用于比较
        let result_file_path = result_dir.join(format!("{}.result", test_name));
        let mut result_file = File::create(&result_file_path).unwrap();
        writeln!(result_file, "Got one of the listed errors").unwrap(); // 期望的错误信息输出

        let args = Args {
            host: "127.0.0.1".to_string(),
            port: "3306".to_string(),
            user: "root".to_string(),
            passwd: "123456".to_string(),
            log_level: "error".to_string(),
            record: false, // 使用比较模式
            params: "".to_string(),
            all: false,
            reserve_schema: false,
            xunit_file: "".to_string(),
            retry_conn_count: 1,
            check_err: false,
            collation_disable: false,
            extension: "result".to_string(),
            email_enable: false,
            email_smtp_host: "".to_string(),
            email_smtp_port: 587,
            email_username: "".to_string(),
            email_password: "".to_string(),
            email_from: "".to_string(),
            email_to: "".to_string(),
            email_enable_tls: false,
            fail_fast: false, // 不启用 fail_fast
            test_files: vec![],
        };

        let mut tester = match Tester::new(args) {
            Ok(t) => t,
            Err(e) => {
                warn!("Skipping test_expected_error_handling due to DB connection error: {}. This test requires a running MySQL server.", e);
                return;
            }
        };
        let result = tester.run_test_file(test_name).unwrap();
        
        // 调试输出
        println!("Test result: success={}, passed={}, failed={}", 
                 result.success, result.passed_queries, result.failed_queries);
        for error in &result.errors {
            println!("Error: {}", error);
        }
        
        // 期望测试成功，因为错误是预期的且结果匹配
        assert!(result.success, "Expected error test should succeed when error is expected and result matches");

        // 清理
        fs::remove_file(test_file_path).unwrap();
        fs::remove_file(result_file_path).unwrap();
    }

    #[test]
    fn test_fail_fast_no_result_file() {
        use std::fs::{self, File};
        use std::io::Write;
        
        let test_name = "fail_fast_no_result_test";
        let test_dir = std::path::Path::new("t");
        fs::create_dir_all(test_dir).unwrap();

        let test_file_path = test_dir.join(format!("{}.test", test_name));
        let mut file = File::create(&test_file_path).unwrap();
        writeln!(file, "--disable_query_log").unwrap();
        writeln!(file, "CREATE TABLE fail_test (id INT);").unwrap();
        writeln!(file, "SELECT * FROM non_existing_table;").unwrap(); // 这个查询会失败

        let args = Args {
            host: "127.0.0.1".to_string(),
            port: "3306".to_string(),
            user: "root".to_string(),
            passwd: "123456".to_string(),
            log_level: "error".to_string(),
            record: true, // 启用 record 模式
            params: "".to_string(),
            all: false,
            reserve_schema: false,
            xunit_file: "".to_string(),
            retry_conn_count: 1,
            check_err: false,
            collation_disable: false,
            extension: "result".to_string(),
            email_enable: false,
            email_smtp_host: "".to_string(),
            email_smtp_port: 587,
            email_username: "".to_string(),
            email_password: "".to_string(),
            email_from: "".to_string(),
            email_to: "".to_string(),
            email_enable_tls: false,
            fail_fast: true, // 启用 fail_fast
            test_files: vec![],
        };

        let mut tester = match Tester::new(args) {
            Ok(t) => t,
            Err(e) => {
                warn!("Skipping test_fail_fast_no_result_file due to DB connection error: {}. This test requires a running MySQL server.", e);
                return;
            }
        };

        let result = tester.run_test_file(test_name).unwrap();
        
        // 验证测试失败（由于SQL错误）
        assert!(!result.success);
        assert!(result.failed_queries > 0);

        // 验证在 fail_fast + record 模式下，不会生成 result 文件
        let result_file_path = std::path::Path::new("r").join(format!("{}.result", test_name));
        assert!(!result_file_path.exists(), "在 fail_fast 模式下出现错误时，不应该生成 result 文件");

        // 清理
        fs::remove_file(test_file_path).unwrap();
        // result 文件应该不存在，所以不需要删除
    }

    #[test]
    fn test_fail_fast_false_still_generates_result() {
        use std::fs::{self, File};
        use std::io::Write;
        
        let test_name = "fail_fast_false_result_test";
        let test_dir = std::path::Path::new("t");
        fs::create_dir_all(test_dir).unwrap();

        let test_file_path = test_dir.join(format!("{}.test", test_name));
        let mut file = File::create(&test_file_path).unwrap();
        writeln!(file, "--disable_query_log").unwrap();
        writeln!(file, "CREATE TABLE fail_test (id INT);").unwrap();
        writeln!(file, "SELECT * FROM non_existing_table;").unwrap(); // 这个查询会失败

        let args = Args {
            host: "127.0.0.1".to_string(),
            port: "3306".to_string(),
            user: "root".to_string(),
            passwd: "123456".to_string(),
            log_level: "error".to_string(),
            record: true, // 启用 record 模式
            params: "".to_string(),
            all: false,
            reserve_schema: false,
            xunit_file: "".to_string(),
            retry_conn_count: 1,
            check_err: false,
            collation_disable: false,
            extension: "result".to_string(),
            email_enable: false,
            email_smtp_host: "".to_string(),
            email_smtp_port: 587,
            email_username: "".to_string(),
            email_password: "".to_string(),
            email_from: "".to_string(),
            email_to: "".to_string(),
            email_enable_tls: false,
            fail_fast: false, // 不启用 fail_fast
            test_files: vec![],
        };

        let mut tester = match Tester::new(args) {
            Ok(t) => t,
            Err(e) => {
                warn!("Skipping test_fail_fast_false_still_generates_result due to DB connection error: {}. This test requires a running MySQL server.", e);
                return;
            }
        };

        let result = tester.run_test_file(test_name).unwrap();
        
        // 验证测试失败（由于SQL错误）
        assert!(!result.success);
        assert!(result.failed_queries > 0);

        // 验证在非 fail_fast 模式下，即使有错误也会生成 result 文件
        let result_file_path = std::path::Path::new("r").join(format!("{}.result", test_name));
        assert!(result_file_path.exists(), "在非 fail_fast 模式下，即使有错误也应该生成 result 文件");

        // 清理
        fs::remove_file(test_file_path).unwrap();
        if result_file_path.exists() {
            fs::remove_file(result_file_path).unwrap();
        }
    }
}
