//! Pest-based parser implementation for .test files
//! 
//! This module provides an alternative parser implementation using the Pest parsing library.

#[cfg(feature = "pest")]
use pest::Parser as PestParserTrait;
#[cfg(feature = "pest")]
use pest_derive::Parser;
#[cfg(feature = "pest")]
use anyhow::{anyhow, Result};

#[cfg(feature = "pest")]
use super::query::{Query, QueryType, QueryOptions};
#[cfg(feature = "pest")]
use super::parser::QueryParser;

#[cfg(feature = "pest")]
#[derive(Parser)]
#[grammar = "tester/mysql_test.pest"]
pub struct PestMySQLParser;

#[cfg(feature = "pest")]
pub struct PestParser {
    delimiter: String,
}

#[cfg(feature = "pest")]
impl Default for PestParser {
    fn default() -> Self {
        Self {
            delimiter: ";".to_string(),
        }
    }
}

#[cfg(feature = "pest")]
impl PestParser {
    pub fn new() -> Self {
        Self::default()
    }

    /// Convert pest parse tree to Query objects
    fn convert_to_queries(&mut self, pairs: pest::iterators::Pairs<Rule>) -> Result<Vec<Query>> {
        let mut queries = Vec::new();
        let line_num = 1;

        for pair in pairs {
            match pair.as_rule() {
                Rule::test_file => {
                    // Recursively process the test file contents
                    let inner_queries = self.convert_to_queries(pair.into_inner())?;
                    queries.extend(inner_queries);
                }
                Rule::comment => {
                    let comment_text = self.extract_comment_text(pair)?;
                    queries.push(Query {
                        query_type: QueryType::Comment,
                        query: comment_text,
                        line: line_num,
                        options: QueryOptions::default(),
                    });
                }
                Rule::command => {
                    let (query_type, query_content) = self.parse_command_pair(pair)?;
                    queries.push(Query {
                        query_type,
                        query: query_content,
                        line: line_num,
                        options: QueryOptions::default(),
                    });
                }
                Rule::delimiter_change => {
                    let delimiter_value = self.extract_delimiter_value(pair)?;
                    self.delimiter = delimiter_value.clone();
                    queries.push(Query {
                        query_type: QueryType::Delimiter,
                        query: delimiter_value,
                        line: line_num,
                        options: QueryOptions::default(),
                    });
                }
                Rule::if_stmt => {
                    let condition = self.extract_condition(pair)?;
                    queries.push(Query {
                        query_type: QueryType::If,
                        query: condition,
                        line: line_num,
                        options: QueryOptions::default(),
                    });
                }
                Rule::while_stmt => {
                    let condition = self.extract_condition(pair)?;
                    queries.push(Query {
                        query_type: QueryType::While,
                        query: condition,
                        line: line_num,
                        options: QueryOptions::default(),
                    });
                }
                Rule::end_stmt => {
                    queries.push(Query {
                        query_type: QueryType::End,
                        query: String::new(),
                        line: line_num,
                        options: QueryOptions::default(),
                    });
                }
                Rule::close_brace => {
                    queries.push(Query {
                        query_type: QueryType::CloseBrace,
                        query: String::new(),
                        line: line_num,
                        options: QueryOptions::default(),
                    });
                }
                Rule::sql_statement => {
                    let sql_content = self.extract_sql_statement(pair)?;
                    if !sql_content.trim().is_empty() {
                        // Remove trailing delimiter if present
                        let cleaned_sql = self.remove_delimiter(&sql_content);
                        queries.push(Query {
                            query_type: QueryType::Query,
                            query: cleaned_sql,
                            line: line_num,
                            options: QueryOptions::default(),
                        });
                    }
                }
                Rule::let_stmt => {
                    let let_args = self.extract_let_args(pair)?;
                    queries.push(Query {
                        query_type: QueryType::Let,
                        query: let_args,
                        line: line_num,
                        options: QueryOptions::default(),
                    });
                }
                Rule::empty_line => {
                    // Skip empty lines
                }
                _ => {
                    // Handle other rules or skip
                }
            }
        }

        Ok(queries)
    }

    fn parse_command_pair(&self, pair: pest::iterators::Pair<Rule>) -> Result<(QueryType, String)> {
        let mut command_content = String::new();

        for inner_pair in pair.into_inner() {
            match inner_pair.as_rule() {
                Rule::command_content => {
                    command_content = inner_pair.as_str().trim().to_string();
                }
                _ => {}
            }
        }

        // Extract command name and args from content
        let parts: Vec<&str> = command_content.splitn(2, char::is_whitespace).collect();
        let command_name = if !parts.is_empty() {
            parts[0].to_lowercase()
        } else {
            String::new()
        };
        let command_args = if parts.len() > 1 {
            parts[1].trim().to_string()
        } else {
            String::new()
        };

        // Map command name to QueryType (reuse the logic from handwritten parser)
        let query_type = self.map_command_to_query_type(&command_name);
        Ok((query_type, command_args))
    }

    fn map_command_to_query_type(&self, command: &str) -> QueryType {
        match command {
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
            "if" => QueryType::If,
            "while" => QueryType::While,
            "end" => QueryType::End,
            _ => QueryType::Unknown,
        }
    }

    fn extract_delimiter_value(&self, pair: pest::iterators::Pair<Rule>) -> Result<String> {
        for inner_pair in pair.into_inner() {
            if inner_pair.as_rule() == Rule::delimiter_value {
                return Ok(inner_pair.as_str().trim().to_string());
            }
        }
        Err(anyhow!("Failed to extract delimiter value"))
    }

    fn extract_condition(&self, pair: pest::iterators::Pair<Rule>) -> Result<String> {
        for inner_pair in pair.into_inner() {
            if inner_pair.as_rule() == Rule::condition {
                return Ok(inner_pair.as_str().trim().to_string());
            }
        }
        Err(anyhow!("Failed to extract condition"))
    }

    fn extract_comment_text(&self, pair: pest::iterators::Pair<Rule>) -> Result<String> {
        let pair_str = pair.as_str();
        for inner_pair in pair.into_inner() {
            if inner_pair.as_rule() == Rule::comment_text {
                return Ok(format!("#{}", inner_pair.as_str()));
            }
        }
        Ok(pair_str.to_string())
    }

    fn extract_sql_statement(&self, pair: pest::iterators::Pair<Rule>) -> Result<String> {
        let pair_str = pair.as_str();
        for inner_pair in pair.into_inner() {
            if inner_pair.as_rule() == Rule::sql_content {
                return Ok(inner_pair.as_str().to_string());
            }
        }
        Ok(pair_str.trim().to_string())
    }

    fn remove_delimiter(&self, sql: &str) -> String {
        let trimmed = sql.trim();
        if trimmed.ends_with(&self.delimiter) {
            trimmed[..trimmed.len() - self.delimiter.len()].trim().to_string()
        } else {
            trimmed.to_string()
        }
    }

    fn extract_let_args(&self, pair: pest::iterators::Pair<Rule>) -> Result<String> {
        for inner_pair in pair.into_inner() {
            if inner_pair.as_rule() == Rule::let_assignment {
                return Ok(inner_pair.as_str().trim().to_string());
            }
        }
        Err(anyhow!("Failed to extract let arguments"))
    }
}

#[cfg(feature = "pest")]
impl QueryParser for PestParser {
    fn parse(&mut self, content: &str) -> Result<Vec<Query>> {
        let pairs = PestMySQLParser::parse(Rule::test_file, content)
            .map_err(|e| anyhow!("Pest parsing error: {}", e))?;
        
        self.convert_to_queries(pairs)
    }
}

// No re-export needed since we use the factory pattern 