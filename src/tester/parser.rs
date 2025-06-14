//! Parser for .test files
//! 
//! This module handles parsing of MySQL test files, including various commands and queries.

use super::query::{Query, QueryType};
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_query() {
        let mut parser = Parser::new();
        let content = "SELECT 1;";
        let queries = parser.parse(content).expect("Failed to parse simple query");
        
        assert_eq!(queries.len(), 1);
        assert_eq!(queries[0].query_type, QueryType::Query);
        assert_eq!(queries[0].query, "SELECT 1");
    }

    #[test]
    fn test_parse_command() {
        let mut parser = Parser::new();
        let content = "--echo hello world";
        let queries = parser.parse(content).expect("Failed to parse command");
        
        assert_eq!(queries.len(), 1);
        assert_eq!(queries[0].query_type, QueryType::Echo);
        assert_eq!(queries[0].query, "hello world");
    }

    #[test]
    fn test_parse_comment() {
        let mut parser = Parser::new();
        let content = "# This is a comment";
        let queries = parser.parse(content).expect("Failed to parse comment");
        
        assert_eq!(queries.len(), 1);
        assert_eq!(queries[0].query_type, QueryType::Comment);
    }

    #[test]
    fn test_parse_multiline_query() {
        let mut parser = Parser::new();
        let content = "SELECT 1\nFROM dual;";
        let queries = parser.parse(content).expect("Failed to parse multiline query");
        
        assert_eq!(queries.len(), 1);
        assert_eq!(queries[0].query_type, QueryType::Query);
        assert_eq!(queries[0].query, "SELECT 1\nFROM dual");
    }

    #[test]
    fn test_delimiter_change() {
        let mut parser = Parser::new();
        let content = "--delimiter //\nSELECT 1//\n--delimiter ;\nSELECT 2;";
        let queries = parser.parse(content).expect("Failed to parse delimiter change");
        
        assert_eq!(queries.len(), 4);
        assert_eq!(queries[0].query_type, QueryType::Delimiter);
        assert_eq!(queries[0].query, "//");
        assert_eq!(queries[1].query_type, QueryType::Query);
        assert_eq!(queries[1].query, "SELECT 1");
        assert_eq!(queries[2].query_type, QueryType::Delimiter);
        assert_eq!(queries[2].query, ";");
        assert_eq!(queries[3].query_type, QueryType::Query);
        assert_eq!(queries[3].query, "SELECT 2");
    }
}
