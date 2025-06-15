//! Parser for .test files
//! 
//! This module handles parsing of MySQL test files, including various commands and queries.

use super::query::{Query, QueryType, QueryOptions};
use anyhow::{anyhow, Result};
use phf::phf_map;

/// Static command mapping from command strings to QueryType
static COMMAND_MAP: phf::Map<&'static str, QueryType> = phf_map! {
    "query" => QueryType::Query,
    "exec" => QueryType::Exec,
    "admin" => QueryType::Admin,
    "error" => QueryType::Error,
    "fatal" => QueryType::Fatal,
    "echo" => QueryType::Echo,
    "sleep" => QueryType::Sleep,
    "replace_regex" => QueryType::ReplaceRegex,
    "replace_column" => QueryType::ReplaceColumn,
    "replace_result" => QueryType::Replace,
    "let" => QueryType::Let,
    "eval" => QueryType::Eval,
    "require" => QueryType::Require,
    "source" => QueryType::Source,
    "comment" => QueryType::Comment,
    "connect" => QueryType::Connect,
    "connection" => QueryType::Connection,
    "disconnect" => QueryType::Disconnect,
    "delimiter" => QueryType::Delimiter,
    "disable_query_log" => QueryType::DisableQueryLog,
    "enable_query_log" => QueryType::EnableQueryLog,
    "disable_result_log" => QueryType::DisableResultLog,
    "enable_result_log" => QueryType::EnableResultLog,
    "sorted_result" => QueryType::SortedResult,
    "enable_sort_result" => QueryType::EnableSortResult,
    "disable_sort_result" => QueryType::DisableSortResult,
    "change_user" => QueryType::ChangeUser,
    "eof" => QueryType::EndOfFile,
    "begin_concurrent" => QueryType::BeginConcurrent,
    "end_concurrent" => QueryType::EndConcurrent,
    "concurrent" => QueryType::Concurrent,
    "vertical_results" => QueryType::VerticalResults,
    "horizontal_results" => QueryType::HorizontalResults,
    "send" => QueryType::Send,
    "recv" => QueryType::Recv,
    "wait" => QueryType::Wait,
    "real_sleep" => QueryType::RealSleep,
    "query_async" => QueryType::QueryAsync,
    "block" => QueryType::Block,
    "unblock" => QueryType::Unblock,
    "checkpoint" => QueryType::Checkpoint,
    "restart" => QueryType::Restart,
    "ping" => QueryType::Ping,
    "skip" => QueryType::Skip,
    "exit" => QueryType::Exit,
    // Control flow commands
    "if" => QueryType::If,
    "while" => QueryType::While,
    "end" => QueryType::End,
};

/// Parser for .test files
pub struct Parser {
    delimiter: String,
}

impl Default for Parser {
    fn default() -> Self {
        Self {
            delimiter: ";".to_string(),
        }
    }
}

impl Parser {
    pub fn new() -> Self {
        Self::default()
    }

    /// Parse a .test file content into a vector of queries
    pub fn parse(&mut self, content: &str) -> Result<Vec<Query>> {
        let mut queries = Vec::new();
        let lines: Vec<&str> = content.lines().collect();
        let mut line_num = 0;

        while line_num < lines.len() {
            let line = lines[line_num].trim();
            line_num += 1;

            // Skip empty lines
            if line.is_empty() {
                continue;
            }

            // Handle comments (lines starting with #)
            if line.starts_with('#') {
                queries.push(Query {
                    query_type: QueryType::Comment,
                    query: line.to_string(),
                    line: line_num,
                    options: QueryOptions::default(),
                });
                continue;
            }

            // Handle commands (lines starting with --)
            if line.starts_with("--") {
                let (query_type, query_content, lines_consumed) = self.parse_command(line, &lines, line_num - 1)?;
                line_num += lines_consumed;
                
                queries.push(Query {
                    query_type,
                    query: query_content,
                    line: line_num - lines_consumed,
                    options: QueryOptions::default(),
                });
                continue;
            }

            // Handle control flow statements (if, while)
            let trimmed_line = line.trim();
            if self.is_control_flow_statement(trimmed_line) {
                let (query_type, query_content, lines_consumed) = self.parse_control_flow(trimmed_line, &lines, line_num - 1)?;
                line_num += lines_consumed;
                
                queries.push(Query {
                    query_type,
                    query: query_content,
                    line: line_num - lines_consumed,
                    options: QueryOptions::default(),
                });
                continue;
            }

            // Handle closing brace
            if line.trim() == "}" {
                queries.push(Query {
                    query_type: QueryType::CloseBrace,
                    query: String::new(),
                    line: line_num,
                    options: QueryOptions::default(),
                });
                continue;
            }

            // Handle 'end' keyword
            if line.trim() == "end" {
                queries.push(Query {
                    query_type: QueryType::End,
                    query: String::new(),
                    line: line_num,
                    options: QueryOptions::default(),
                });
                continue;
            }

            // Handle let statements without -- prefix
            if Self::is_let_statement(trimmed_line) {
                let let_args = Self::extract_let_args(trimmed_line);
                queries.push(Query {
                    query_type: QueryType::Let,
                    query: let_args,
                    line: line_num,
                    options: QueryOptions::default(),
                });
                continue;
            }

            // Handle regular queries
            let (query_content, lines_consumed) = self.parse_query(&lines, line_num - 1)?;
            line_num += lines_consumed;

            queries.push(Query {
                query_type: QueryType::Query,
                query: query_content,
                line: line_num - lines_consumed,
                options: QueryOptions::default(),
            });
        }

        Ok(queries)
    }

    /// Parse a command line starting with --
    fn parse_command(&mut self, line: &str, _lines: &[&str], _start_line: usize) -> Result<(QueryType, String, usize)> {
        // Remove the -- prefix
        let command_line = line.trim_start_matches("--").trim();
        
        // Split command and arguments
        let parts: Vec<&str> = command_line.split_whitespace().collect();
        if parts.is_empty() {
            return Ok((QueryType::Comment, line.to_string(), 0));
        }

        let command = parts[0].to_lowercase();
        let args = if parts.len() > 1 {
            parts[1..].join(" ")
        } else {
            String::new()
        };

        // Handle delimiter command specially
        if command == "delimiter" {
            if !args.is_empty() {
                self.delimiter = args.clone();
            }
            return Ok((QueryType::Delimiter, args, 0));
        }

        // Look up command in the map
        let query_type = COMMAND_MAP.get(&command).copied().unwrap_or(QueryType::Unknown);
        
        Ok((query_type, args, 0))
    }

    /// Parse a regular query (potentially multi-line until delimiter)
    fn parse_query(&self, lines: &[&str], start_line: usize) -> Result<(String, usize)> {
        let mut query_parts = Vec::new();
        let mut line_idx = start_line;
        
        while line_idx < lines.len() {
            let line = lines[line_idx];
            
            // Check if this line ends with the delimiter
            if line.trim_end().ends_with(&self.delimiter) {
                // Remove the delimiter from the end
                let line_without_delimiter = line.trim_end()
                    .trim_end_matches(&self.delimiter)
                    .trim_end();
                
                if !line_without_delimiter.is_empty() {
                    query_parts.push(line_without_delimiter);
                }
                
                // Calculate lines consumed
                let lines_consumed = line_idx - start_line;
                let full_query = query_parts.join("\n").trim().to_string();
                
                return Ok((full_query, lines_consumed));
            }
            
            // Add the whole line if it doesn't end with delimiter
            query_parts.push(line);
            line_idx += 1;
        }

        // If we reach here, we didn't find a delimiter
        // This could be the last query in the file
        let full_query = query_parts.join("\n").trim().to_string();
        let lines_consumed = line_idx - start_line;
        
        if full_query.is_empty() {
            return Err(anyhow!("Empty query at line {}", start_line + 1));
        }
        
        Ok((full_query, lines_consumed))
    }

    /// Check if a line is a control flow statement
    fn is_control_flow_statement(&self, line: &str) -> bool {
        let line = line.trim();
        
        // Check for if/while followed by whitespace or opening parenthesis
        if line.starts_with("if") {
            let rest = &line[2..];
            return rest.is_empty() || rest.starts_with(char::is_whitespace) || rest.starts_with('(');
        }
        
        if line.starts_with("while") {
            let rest = &line[5..];
            return rest.is_empty() || rest.starts_with(char::is_whitespace) || rest.starts_with('(');
        }
        
        false
    }

    /// Check if a line is a let statement (without -- prefix)
    /// Supports various formats like:
    /// - "let $var = value"
    /// - "let$var=value" (no spaces)
    /// - "let   $var   =   value" (extra spaces)
    fn is_let_statement(line: &str) -> bool {
        let line = line.trim();
        
        // Must start with "let" (case insensitive)
        if !line.to_lowercase().starts_with("let") {
            return false;
        }
        
        let rest = &line[3..];
        
        // After "let", there must be either:
        // 1. Whitespace followed by variable assignment
        // 2. Direct variable assignment (like "let$var=")
        if rest.is_empty() {
            return false;
        }
        
        // Check if the rest starts with whitespace or directly with $ or variable name
        let rest_trimmed = rest.trim_start();
        if rest_trimmed.is_empty() {
            return false;
        }
        
        // Must contain an assignment (=)
        rest_trimmed.contains('=')
    }

    /// Extract arguments from a let statement
    /// Handles various spacing formats
    fn extract_let_args(line: &str) -> String {
        let line = line.trim();
        
        // Remove "let" prefix (case insensitive)
        let rest = if line.to_lowercase().starts_with("let") {
            &line[3..]
        } else {
            line
        };
        
        rest.trim().to_string()
    }

    /// Parse control flow statements (if/while)
    /// Supports both syntaxes:
    /// - if (condition) { ... }
    /// - if (condition) ... end
    /// - if(condition) { ... }  (no space before parenthesis)
    /// - while (condition) { ... }
    /// - while(condition) { ... }
    fn parse_control_flow(&mut self, line: &str, _lines: &[&str], _start_line: usize) -> Result<(QueryType, String, usize)> {
        let line = line.trim();
        
        // Determine if it's if or while and extract the rest
        let (keyword, rest) = if line.starts_with("if") {
            ("if", line[2..].trim_start())
        } else if line.starts_with("while") {
            ("while", line[5..].trim_start())
        } else {
            return Err(anyhow!("Invalid control flow statement: {}", line));
        };

        let rest = rest.trim();
        
        // Parse condition in parentheses
        if !rest.starts_with('(') {
            return Err(anyhow!("Control flow condition must be in parentheses: {}", line));
        }

        // Find the matching closing parenthesis
        let mut paren_count = 0;
        let mut condition_end = 0;
        for (i, ch) in rest.chars().enumerate() {
            match ch {
                '(' => paren_count += 1,
                ')' => {
                    paren_count -= 1;
                    if paren_count == 0 {
                        condition_end = i;
                        break;
                    }
                }
                _ => {}
            }
        }

        if paren_count != 0 {
            return Err(anyhow!("Unmatched parentheses in control flow condition: {}", line));
        }

        // Extract condition (without the outer parentheses)
        let condition = rest[1..condition_end].trim().to_string();
        
        // Check what follows the condition
        let after_condition = rest[condition_end + 1..].trim();
        
        if after_condition.starts_with('{') {
            // Block syntax: if (condition) { ... }
            // The condition is what we need to store
            let query_type = match keyword {
                "if" => QueryType::If,
                "while" => QueryType::While,
                _ => unreachable!(),
            };
            Ok((query_type, condition, 0))
        } else if after_condition.is_empty() {
            // Traditional syntax: if (condition) ... end
            let query_type = match keyword {
                "if" => QueryType::If,
                "while" => QueryType::While,
                _ => unreachable!(),
            };
            Ok((query_type, condition, 0))
        } else {
            return Err(anyhow!("Invalid syntax after control flow condition: {}", line));
        }
    }
}
