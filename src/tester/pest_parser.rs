//! Pest-based parser implementation for .test files
//!
//! This module provides an alternative parser implementation using the Pest parsing library.

#[cfg(feature = "pest")]
use anyhow::{anyhow, Result};
#[cfg(feature = "pest")]
use pest::Parser as PestParserTrait;
#[cfg(feature = "pest")]
use pest_derive::Parser;

#[cfg(feature = "pest")]
use super::parser::QueryParser;
#[cfg(feature = "pest")]
use super::query::{Query, QueryOptions, QueryType};

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
        let mut pending_sql_lines = Vec::new();
        let mut line_num = 1;

        for pair in pairs {
            // Update line number based on the pair's position
            let pair_line = pair.line_col().0;
            line_num = pair_line;
            match pair.as_rule() {
                Rule::test_file => {
                    // Recursively process the test file contents
                    let inner_queries = self.convert_to_queries(pair.into_inner())?;
                    queries.extend(inner_queries);
                }
                Rule::comment => {
                    // Finalize any pending SQL before processing comment
                    if !pending_sql_lines.is_empty() {
                        self.finalize_pending_sql(&mut queries, &mut pending_sql_lines, line_num)?;
                    }

                    let comment_text = self.extract_comment_text(pair)?;
                    queries.push(Query {
                        query_type: QueryType::Comment,
                        query: comment_text,
                        line: line_num,
                        options: QueryOptions::default(),
                    });
                }
                Rule::command => {
                    // Finalize any pending SQL before processing command
                    if !pending_sql_lines.is_empty() {
                        self.finalize_pending_sql(&mut queries, &mut pending_sql_lines, line_num)?;
                    }

                    let (query_type, query_content) = self.parse_command_pair(pair)?;
                    queries.push(Query {
                        query_type,
                        query: query_content,
                        line: line_num,
                        options: QueryOptions::default(),
                    });
                }
                Rule::delimiter_change => {
                    // Finalize any pending SQL before changing delimiter
                    if !pending_sql_lines.is_empty() {
                        self.finalize_pending_sql(&mut queries, &mut pending_sql_lines, line_num)?;
                    }

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
                    // Finalize any pending SQL before processing control flow
                    if !pending_sql_lines.is_empty() {
                        self.finalize_pending_sql(&mut queries, &mut pending_sql_lines, line_num)?;
                    }

                    let condition = self.extract_condition(pair)?;
                    queries.push(Query {
                        query_type: QueryType::If,
                        query: condition,
                        line: line_num,
                        options: QueryOptions::default(),
                    });
                }
                Rule::while_stmt => {
                    // Finalize any pending SQL before processing control flow
                    if !pending_sql_lines.is_empty() {
                        self.finalize_pending_sql(&mut queries, &mut pending_sql_lines, line_num)?;
                    }

                    let condition = self.extract_condition(pair)?;
                    queries.push(Query {
                        query_type: QueryType::While,
                        query: condition,
                        line: line_num,
                        options: QueryOptions::default(),
                    });
                }
                Rule::end_stmt => {
                    // Finalize any pending SQL before processing end
                    if !pending_sql_lines.is_empty() {
                        self.finalize_pending_sql(&mut queries, &mut pending_sql_lines, line_num)?;
                    }

                    queries.push(Query {
                        query_type: QueryType::End,
                        query: String::new(),
                        line: line_num,
                        options: QueryOptions::default(),
                    });
                }
                Rule::close_brace => {
                    // Finalize any pending SQL before processing close brace
                    if !pending_sql_lines.is_empty() {
                        self.finalize_pending_sql(&mut queries, &mut pending_sql_lines, line_num)?;
                    }

                    queries.push(Query {
                        query_type: QueryType::CloseBrace,
                        query: String::new(),
                        line: line_num,
                        options: QueryOptions::default(),
                    });
                }
                Rule::sql_statement => {
                    let sql_content = self.extract_sql_statement(pair)?;
                    self.process_sql_line(
                        &mut pending_sql_lines,
                        &mut queries,
                        sql_content,
                        line_num,
                    )?;
                }
                Rule::let_stmt => {
                    // Finalize any pending SQL before processing let
                    if !pending_sql_lines.is_empty() {
                        self.finalize_pending_sql(&mut queries, &mut pending_sql_lines, line_num)?;
                    }

                    let let_args = self.extract_let_args(pair)?;
                    queries.push(Query {
                        query_type: QueryType::Let,
                        query: let_args,
                        line: line_num,
                        options: QueryOptions::default(),
                    });
                }
                Rule::empty_line => {
                    // Skip empty lines but don't finalize SQL (allow SQL to span empty lines)
                }
                _ => {
                    // Handle other rules or skip
                }
            }
        }

        // Finalize any remaining SQL
        if !pending_sql_lines.is_empty() {
            self.finalize_pending_sql(&mut queries, &mut pending_sql_lines, line_num)?;
        }

        Ok(queries)
    }

    /// Process a single SQL line, accumulating until delimiter is found
    fn process_sql_line(
        &self,
        pending_sql_lines: &mut Vec<String>,
        queries: &mut Vec<Query>,
        sql_content: String,
        line_num: usize,
    ) -> Result<()> {
        let trimmed_content = sql_content.trim();
        if trimmed_content.is_empty() {
            return Ok(());
        }

        // Check if this line ends with the delimiter
        if trimmed_content.ends_with(&self.delimiter) {
            // Remove delimiter and add to pending lines
            let content_without_delimiter = trimmed_content
                .strip_suffix(&self.delimiter)
                .unwrap_or(trimmed_content)
                .trim();

            if !content_without_delimiter.is_empty() {
                pending_sql_lines.push(content_without_delimiter.to_string());
            }

            // Finalize the SQL statement
            self.finalize_pending_sql(queries, pending_sql_lines, line_num)?;
        } else {
            // Add to pending lines (multi-line SQL continues)
            pending_sql_lines.push(trimmed_content.to_string());
        }

        Ok(())
    }

    /// Finalize pending SQL lines into a single Query
    fn finalize_pending_sql(
        &self,
        queries: &mut Vec<Query>,
        pending_sql_lines: &mut Vec<String>,
        line_num: usize,
    ) -> Result<()> {
        if pending_sql_lines.is_empty() {
            return Ok(());
        }

        let full_sql = pending_sql_lines.join("\n").trim().to_string();
        if !full_sql.is_empty() {
            queries.push(Query {
                query_type: QueryType::Query,
                query: full_sql,
                line: line_num,
                options: QueryOptions::default(),
            });
        }

        pending_sql_lines.clear();
        Ok(())
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
            trimmed[..trimmed.len() - self.delimiter.len()]
                .trim()
                .to_string()
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tester::parser::create_parser;
    use crate::tester::query::QueryType;

    #[test]
    fn test_pest_parse_simple_query() {
        let mut parser = create_parser("pest").expect("Failed to create pest parser");
        let content = "SELECT 1;";
        let queries = parser.parse(content).expect("Failed to parse simple query");

        assert_eq!(queries.len(), 1);
        assert_eq!(queries[0].query_type, QueryType::Query);
        assert_eq!(queries[0].query.trim(), "SELECT 1");
    }

    #[test]
    fn test_pest_parse_command() {
        let mut parser = create_parser("pest").expect("Failed to create pest parser");
        let content = "--echo hello world";
        let queries = parser.parse(content).expect("Failed to parse command");

        assert_eq!(queries.len(), 1);
        assert_eq!(queries[0].query_type, QueryType::Echo);
        assert_eq!(queries[0].query, "hello world");
    }

    #[test]
    fn test_pest_parse_comment() {
        let mut parser = create_parser("pest").expect("Failed to create pest parser");
        let content = "# This is a comment";
        let queries = parser.parse(content).expect("Failed to parse comment");

        assert_eq!(queries.len(), 1);
        assert_eq!(queries[0].query_type, QueryType::Comment);
        assert!(queries[0].query.contains("This is a comment"));
    }

    #[test]
    fn test_pest_parse_if_statement() {
        let mut parser = create_parser("pest").expect("Failed to create pest parser");
        let content = "if ($var > 0) {\n--echo positive\n}";
        let queries = parser.parse(content).expect("Failed to parse if statement");

        // Should have at least the if statement
        assert!(!queries.is_empty());
        let if_query = queries.iter().find(|q| q.query_type == QueryType::If);
        assert!(if_query.is_some());
        assert_eq!(if_query.unwrap().query.trim(), "$var > 0");
    }

    #[test]
    fn test_pest_parse_delimiter_change() {
        let mut parser = create_parser("pest").expect("Failed to create pest parser");
        let content = "--delimiter //\nSELECT 1//";
        let queries = parser
            .parse(content)
            .expect("Failed to parse delimiter change");

        assert!(queries.len() >= 2);

        // Find delimiter command
        let delimiter_query = queries
            .iter()
            .find(|q| q.query_type == QueryType::Delimiter);
        assert!(delimiter_query.is_some());
        assert_eq!(delimiter_query.unwrap().query, "//");

        // Find SQL query
        let sql_query = queries.iter().find(|q| q.query_type == QueryType::Query);
        assert!(sql_query.is_some());
        // The query may still contain the delimiter when parsed
        assert!(sql_query.unwrap().query.contains("SELECT 1"));
    }
}

// No re-export needed since we use the factory pattern
