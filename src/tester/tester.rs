//! Test execution engine
//! 
//! This module handles the execution of MySQL test cases, including database setup,
//! query execution, result comparison, and cleanup.

use super::database::ConnectionInfo;
use super::expression::ExpressionEvaluator;
use super::parser::Parser;
use super::query::{Query, QueryType};
use super::variables::VariableContext;
use crate::tester::command::Command;
use crate::tester::connection_manager::ConnectionManager;
use crate::tester::error_handler::MySQLErrorHandler;
use crate::tester::registry::COMMAND_REGISTRY;
use crate::cli::Args;
use anyhow::{anyhow, Result};
use log::{debug, error, info, warn};
use regex::Regex;
use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use rayon::prelude::*;
use mysql::prelude::*;

/// Maximum number of loop iterations to prevent infinite loops
const MAX_LOOP_ITERATIONS: usize = 10_000;

/// Control flow frame for while loops
#[derive(Debug, Clone)]
struct WhileFrame {
    /// Start index of the while loop (index of the while command)
    start_index: usize,
    /// End index of the while loop (index of the matching end command)
    end_index: usize,
    /// The condition expression to evaluate
    condition: String,
    /// Current iteration count
    iteration_count: usize,
}

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
    current_result_line: usize,
    
    // --- One-shot modifiers for the next query ---
    /// Sort results for the next query
    pub pending_sorted_result: bool,
    /// Regex for result replacement for the next query
    pub pending_replace_regex: Vec<(Regex, String)>,
    /// Variable context for storing test variables
    pub variable_context: VariableContext,
    
    // --- Control flow fields ---
    /// Expression evaluator for if/while conditions
    expression_evaluator: ExpressionEvaluator,
    /// Stack of active while loops
    while_stack: Vec<WhileFrame>,
    /// Mapping from start index to end index for control structures
    control_flow_map: HashMap<usize, usize>,
    /// Flag to indicate if we are inside a concurrent block
    in_concurrent_block: bool,
    /// Queries to be executed concurrently
    concurrent_queries: Vec<Query>,
    #[allow(dead_code)]
    test_errors: Vec<String>,
    #[allow(dead_code)]
    pc: usize,
    #[allow(dead_code)]
    loop_stack: Vec<usize>,
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
            current_result_line: 1, // Line numbers are 1-based
            pending_sorted_result: false,
            pending_replace_regex: Vec::new(),
            variable_context: VariableContext::new(),
            expression_evaluator: ExpressionEvaluator::new(),
            while_stack: Vec::new(),
            control_flow_map: HashMap::new(),
            in_concurrent_block: false,
            concurrent_queries: Vec::new(),
            test_errors: Vec::new(),
            pc: 0,
            loop_stack: Vec::new(),
        })
    }

    /// Set the current test name and prepare for execution
    pub fn set_test(&mut self, test_name: &str) -> Result<()> {
        self.test_name = test_name.to_string();
        self.output_buffer.clear();
        self.expected_errors.clear();
        self.current_result_line = 1;
        self.pending_sorted_result = false;
        self.pending_replace_regex.clear();
        
        // Clear control flow state
        self.while_stack.clear();
        self.control_flow_map.clear();
        self.in_concurrent_block = false;
        self.concurrent_queries.clear();
        
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

        // Build control flow mapping
        self.build_control_flow_map(&queries)?;

        // Execute queries with control flow support
        let mut test_result = TestResult::new(&test_name);
        let mut pc = 0; // Program counter
        
        while pc < queries.len() {
            let query = &queries[pc];
            match self.execute_query_with_control_flow(query, pc, &queries) {
                Ok(next_pc) => {
                    test_result.passed_queries += 1;
                    pc = next_pc;
                }
                Err(e) => {
                    error!("Line {} (PC {}): {}", query.line, pc, e);
                    test_result.failed_queries += 1;
                    test_result.errors.push(format!("Line {}: {}", query.line, e));
                    if self.args.fail_fast {
                        break; // 快速失败，终止后续查询
                    }
                    pc += 1; // Continue to next query on error
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

        // 最终验证：确保所有期望输出均已消费，防止遗漏行假通过
        if !self.args.record {
            self.verify_expected_consumed()?;
        }

        Ok(test_result)
    }

    /// Execute a single query or command
    fn execute_query(&mut self, query: &Query, query_num: usize) -> Result<()> {
        debug!("Executing query {:?} (line {}): {:?}", query_num, query.line, query.query_type);

        // 注入绑定在 Query 上的一次性修饰符
        if !query.options.expected_errors.is_empty() {
            self.expected_errors = query.options.expected_errors.clone();
        }
        if !query.options.replace_regex.is_empty() {
            self.pending_replace_regex = query.options.replace_regex.clone();
        }
        if query.options.sorted_result {
            self.pending_sorted_result = true;
        }

        match query.query_type {
            QueryType::Query => {
                self.execute_sql_query(&query.query, query_num)?;
                // Modifiers are one-shot, clear them after the SQL query runs.
                self.expected_errors.clear();
                self.pending_sorted_result = false;
                self.pending_replace_regex.clear();
            }
            QueryType::Exec => {
                // Create a command object to use the new handler system
                let cmd = Command {
                    name: "exec".to_string(),
                    args: query.query.clone(),
                    line: query.line,
                };
                
                // Execute from registry
                if let Some(executor) = COMMAND_REGISTRY.get(cmd.name.as_str()) {
                    executor(self, &cmd)?;
                } else {
                    return Err(anyhow!("'exec' command handler not found in registry"));
                }
                
                // Clear modifiers and expected errors after exec command
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
            QueryType::Let => {
                let cmd = Command {
                    name: "let".to_string(),
                    args: query.query.clone(),
                    line: query.line,
                };
                
                if let Some(executor) = COMMAND_REGISTRY.get(cmd.name.as_str()) {
                    executor(self, &cmd)?;
                } else {
                    return Err(anyhow!("'let' command handler not found in registry"));
                }
                
                // Clear any pending error expectations as they don't apply to let commands
                if !self.expected_errors.is_empty() {
                    warn!("--error directive before --let is ignored");
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
        // Expand variables in the SQL query
        let expanded_sql = self.variable_context.expand(sql)?;
        
        if self.enable_query_log {
            let query_output = format!("{}\n", expanded_sql);
            if self.args.record {
                write!(self.output_buffer, "{}", query_output)?;
            } else {
                self.compare_with_result(&query_output)?;
            }
        }

        let execution_result = self.connection_manager.current_database()?.query(&expanded_sql);

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
        // Expand variables in the echo text
        let expanded_text = self.variable_context.expand(text)?;
        let echo_output = format!("{}\n", expanded_text);
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
        // Expand variables in connection parameters
        let expanded_params = self.variable_context.expand(params)?;
        self.connection_manager.connect(&expanded_params)?;
        info!("Connected to new database connection");
        Ok(())
    }

    /// Handle the --connection command (switch connection)
    fn handle_connection_switch(&mut self, conn_name: &str) -> Result<()> {
        // Expand variables in connection name
        let expanded_conn_name = self.variable_context.expand(conn_name)?;
        self.connection_manager.switch_connection(expanded_conn_name.trim())?;
        info!("Switched to connection: {}", expanded_conn_name.trim());
        Ok(())
    }

    /// Handle the --disconnect command
    fn handle_disconnect(&mut self, conn_name: &str) -> Result<()> {
        // Expand variables in connection name
        let expanded_conn_name = self.variable_context.expand(conn_name)?;
        self.connection_manager.disconnect(expanded_conn_name.trim())?;
        info!("Disconnected connection: {}", expanded_conn_name.trim());
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
    pub fn apply_regex_replacements(&self, buffer: &mut String) {
        use std::borrow::Cow;

        if self.pending_replace_regex.is_empty() {
            return;
        }

        let mut cow: Cow<'_, str> = Cow::Borrowed(buffer.as_str());
        for (regex, replacement) in &self.pending_replace_regex {
            // 调用前先获取 &str 引用，避免同时可变借用
            let replaced = regex.replace_all(&cow, replacement.as_str());
            if let Cow::Owned(s) = replaced {
                cow = Cow::Owned(s);
            }
        }

        if let Cow::Owned(s) = cow {
            *buffer = s;
        }
    }

    /// Get a reference to the expression evaluator
    pub fn expression_evaluator(&self) -> &ExpressionEvaluator {
        &self.expression_evaluator
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
    pub fn compare_with_result(&mut self, output: &str) -> Result<()> {
        if self.args.record {
            self.output_buffer.write_all(output.as_bytes())?;
            return Ok(());
        }

        if let Some(content) = &self.result_file_content {
            let expected_lines: Vec<&str> = content.lines().collect();
            let actual_lines = output.lines();

            for actual_line in actual_lines {
                let cursor = self.current_result_line - 1;

                if cursor >= expected_lines.len() {
                    let err_msg = format!("Output has more lines than expected. Extra line: '{}'", actual_line);
                    return Err(anyhow!(err_msg));
                }

                let expected_line = expected_lines[cursor];
                if actual_line != expected_line {
                    let err_msg = format!(
                        "Output mismatch at line {}:\n    Expected: {}\n    Actual: {}",
                        self.current_result_line, expected_line, actual_line
                    );
                    return Err(anyhow!(err_msg));
                }

                self.current_result_line += 1;
            }
        } else {
            if !output.is_empty() {
                return Err(anyhow!("Result file not found, but output was produced."));
            }
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

    /// 在测试结束时验证是否仍有未消费的期望行
    fn verify_expected_consumed(&self) -> Result<()> {
        if let Some(content) = &self.result_file_content {
            let expected_lines_count = content.lines().count();
            if self.current_result_line - 1 < expected_lines_count {
                let remaining_line = content
                    .lines()
                    .nth(self.current_result_line - 1)
                    .unwrap_or("");
                return Err(anyhow!(format!(
                    "Output missing lines starting at expected line {}:\n    Expected: {}",
                    self.current_result_line,
                    remaining_line
                )));
            }
        }
        Ok(())
    }
}

/// Test execution result
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
            success: true,
            passed_queries: 0,
            failed_queries: 0,
            errors: Vec::new(),
        }
    }
}

impl Tester {
    /// Build control flow mapping for if/while/end structures
    fn build_control_flow_map(&mut self, queries: &[Query]) -> Result<()> {
        let mut stack = Vec::new();

        for (i, query) in queries.iter().enumerate() {
            match query.query_type {
                QueryType::If | QueryType::While => stack.push(i),
                QueryType::End => {
                    if let Some(start_index) = stack.pop() {
                        self.control_flow_map.insert(start_index, i);
                        self.control_flow_map.insert(i, start_index);
                    } else {
                        return Err(anyhow!("Mismatched 'end' at line {}", query.line));
                    }
                }
                _ => {}
            }
        }

        if let Some(&unclosed_idx) = stack.last() {
            let unclosed_query = &queries[unclosed_idx];
            return Err(anyhow!(
                "Unclosed control block starting at line {}",
                unclosed_query.line
            ));
        }

        Ok(())
    }

    /// Execute a query with control flow support
    /// Returns the next program counter value
    fn execute_query_with_control_flow(&mut self, query: &Query, pc: usize, _queries: &[Query]) -> Result<usize> {
        // Handle concurrent blocks first
        if query.query_type == QueryType::BeginConcurrent {
            self.in_concurrent_block = true;
            self.concurrent_queries.clear();
            return Ok(pc + 1);
        }

        if query.query_type == QueryType::EndConcurrent {
            if !self.in_concurrent_block {
                return Err(anyhow!("--end_concurrent without --begin_concurrent at line {}", query.line));
            }
            self.in_concurrent_block = false;
            self.execute_concurrent_queries()?;
            return Ok(pc + 1);
        }

        if self.in_concurrent_block {
            match query.query_type {
                QueryType::Query => {
                    let mut concurrent_query = query.clone();
                    // Expand variables in the SQL query before storing
                    concurrent_query.query = self.variable_context.expand(&concurrent_query.query)?;

                    // 将一次性修饰符绑定到 QueryOptions
                    if !self.expected_errors.is_empty() {
                        concurrent_query.options.expected_errors = self.expected_errors.clone();
                        self.expected_errors.clear();
                    }

                    if !self.pending_replace_regex.is_empty() {
                        concurrent_query.options.replace_regex = self.pending_replace_regex.clone();
                        self.pending_replace_regex.clear();
                    }

                    if self.pending_sorted_result {
                        concurrent_query.options.sorted_result = true;
                        self.pending_sorted_result = false;
                    }

                    self.concurrent_queries.push(concurrent_query);
                }
                QueryType::Error => {
                    // Store error expectations for the next query
                    self.parse_expected_errors(&query.query)?;
                }
                _ => {
                    // Execute other commands immediately in serial.
                    self.execute_query(query, pc)?;
                }
            }
            return Ok(pc + 1);
        }

        // Handle control flow: if, while, end
        let next_pc = match query.query_type {
            QueryType::If => self.handle_if_command(&query.query, pc)?,
            QueryType::While => self.handle_while_command(&query.query, pc)?,
            QueryType::End => self.handle_end_command(pc)?,
            _ => {
                // Not a control flow command, execute it normally
                self.execute_query(query, pc)?;
                pc + 1
            }
        };

        Ok(next_pc)
    }

    /// Handle if command
    fn handle_if_command(&mut self, condition: &str, pc: usize) -> Result<usize> {
        // Evaluate the condition
        if self.expression_evaluator.evaluate_condition(
            condition, 
            &self.variable_context,
            self.connection_manager.current_database()?
        )? {
            // Condition is true, continue to the next statement
            Ok(pc + 1)
        } else {
            // Condition is false, jump to matching end
            let end_index = *self.control_flow_map.get(&pc)
                .ok_or_else(|| anyhow!("Mismatched 'end' for 'if' at line {}", pc))?;
            Ok(end_index + 1)
        }
    }

    /// Handle while command
    fn handle_while_command(&mut self, condition: &str, pc: usize) -> Result<usize> {
        let end_index = *self.control_flow_map.get(&pc)
            .ok_or_else(|| anyhow!("Mismatched 'end' for 'while' at line {}", pc))?;

        if self.expression_evaluator.evaluate_condition(
            condition,
            &self.variable_context,
            self.connection_manager.current_database()?
        )? {
            // Condition is true, push to stack and continue
            self.while_stack.push(WhileFrame {
                start_index: pc,
                end_index,
                condition: condition.to_string(),
                iteration_count: 0,
            });
            
            Ok(pc + 1) // Continue to first statement in loop
        } else {
            // Condition is false, skip the loop
            Ok(end_index + 1)
        }
    }

    /// Handle end command
    fn handle_end_command(&mut self, pc: usize) -> Result<usize> {
        if let Some(frame) = self.while_stack.last_mut() {
            if frame.end_index == pc {
                // This 'end' matches an active while loop
                frame.iteration_count += 1;
                if frame.iteration_count >= MAX_LOOP_ITERATIONS {
                    return Err(anyhow!("Infinite loop detected at line {}", frame.start_index + 1));
                }

                if self.expression_evaluator.evaluate_condition(
                    &frame.condition,
                    &self.variable_context,
                    self.connection_manager.current_database()?
                )? {
                    // Loop condition is still true, jump back to while
                    return Ok(frame.start_index);
                } else {
                    // Loop condition is false, pop from stack and continue
                    self.while_stack.pop();
                    return Ok(pc + 1);
                }
            }
        }
        
        // This 'end' corresponds to an 'if' statement, just continue
        Ok(pc + 1)
    }

    fn execute_concurrent_queries(&mut self) -> Result<()> {
        if self.concurrent_queries.is_empty() {
            return Ok(());
        }

        let indexed_queries: Vec<_> = self.concurrent_queries.iter().cloned().enumerate().collect();
        let results = Arc::new(Mutex::new(Vec::<(usize, Result<String, mysql::Error>, Vec<String>)>::new()));

        indexed_queries.par_iter().for_each(|(index, query)| {
            // 尝试获取连接，若失败则将错误入结果集合并，不直接 panic
            let conn_result = self.connection_manager.get_pooled_connection();

            let query_result: Result<String, mysql::Error> = match conn_result {
                Err(_e) => {
                    // 将连接错误转为 DriverError::CouldNotConnect(None)
                    Err(mysql::Error::DriverError(mysql::DriverError::CouldNotConnect(None)))
                }
                Ok(mut conn) => {
                    // 并发路径下，查询字符串已不包含错误前缀
                    let actual_query = query.query.clone();

                    // 执行查询
                    conn.query_iter(&actual_query).and_then(|result| {
                        let rows: Vec<String> = result.map(|row_result| {
                            let row = row_result?;
                            let row_values: Vec<String> = (0..row.len()).map(|i| {
                                let value = row.get::<Option<String>, _>(i)
                                    .unwrap_or_else(|| Some("NULL".to_string()))
                                    .unwrap_or_else(|| "NULL".to_string());
                                value
                            }).collect();
                            Ok(row_values.join("\t"))
                        }).collect::<Result<Vec<String>, mysql::Error>>()?;
                        let mut output = rows.join("\n");
                        // 应用一次性替换规则（仅对该 Query 生效）
                        for (regex, replacement) in &query.options.replace_regex {
                            output = regex.replace_all(&output, replacement.as_str()).into_owned();
                        }
                        if query.options.sorted_result {
                            // 若要求排序，按照行排序
                            let mut lines: Vec<&str> = output.lines().collect();
                            lines.sort();
                            output = lines.join("\n");
                        }
                        Ok(output)
                    })
                }
            };

            let expected_errors: Vec<String> = query.options.expected_errors.clone();

            // 若 Mutex 被 poison，into_inner 仍可安全取得数据；仅记录告警日志
            match results.lock() {
                Ok(mut guard) => guard.push((*index, query_result, expected_errors)),
                Err(poisoned) => {
                    warn!("Results mutex poisoned, continuing with inner data");
                    let mut guard = poisoned.into_inner();
                    guard.push((*index, query_result, expected_errors));
                }
            }
        });

        let mut final_results = match results.lock() {
            Ok(guard) => guard,
            Err(poisoned) => {
                warn!("Results mutex poisoned during collection; using inner data");
                poisoned.into_inner()
            }
        };
        final_results.sort_by_key(|(index, _, _)| *index);

        let mut output_parts: Vec<String> = Vec::new();
        for (_, result, expected_errors) in final_results.iter() {
            match result {
                Ok(output) => {
                    if !expected_errors.is_empty() {
                        // Expected error but query succeeded
                        let err_msg = format!("Expected error(s) {:?}, but query succeeded", expected_errors);
                        output_parts.push(format!("UNEXPECTED_SUCCESS: {}", err_msg));
                    } else {
                        // Apply one-time regex replacements if any
                        let mut final_output = output.clone();
                        self.apply_regex_replacements(&mut final_output);
                        output_parts.push(final_output);
                    }
                }
                Err(e) => {
                    if !expected_errors.is_empty() && self.error_handler.check_expected_error(e, expected_errors) {
                        let error_str = self.error_handler.format_error(e);
                        output_parts.push(error_str);
                    } else if expected_errors.is_empty() {
                        // Unexpected error
                        let error_str = self.error_handler.format_error(e);
                        output_parts.push(format!("UNEXPECTED_ERROR: {}", error_str));
                    } else {
                        // Expected different error
                        let error_str = self.error_handler.format_error(e);
                        output_parts.push(format!("WRONG_ERROR: Expected {:?}, got {}", expected_errors, error_str));
                    }
                }
            }
        }
        
        // 将并发查询的输出合并，并确保与串行路径保持相同的换行语义（结尾带 \n）
        let mut combined_output = output_parts.join("\n");
        if !combined_output.is_empty() && !combined_output.ends_with('\n') {
            combined_output.push('\n');
        }

        if !combined_output.is_empty() {
            if let Err(e) = self.compare_with_result(&combined_output) {
                return Err(e);
            }
        }

        self.in_concurrent_block = false;
        self.concurrent_queries.clear();
        // 并发块结束后，清理一次性修饰符，避免影响后续串行查询
        self.pending_replace_regex.clear();
        self.pending_sorted_result = false;

        Ok(())
    }
}



