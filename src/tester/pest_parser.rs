//! Pest-based parser implementation for .test files
//!
//! This module provides an alternative parser implementation using the Pest parsing library.

use anyhow::{anyhow, Result};
use pest::Parser as PestParserTrait;
use pest_derive::Parser;

use super::parser::{QueryParser, COMMAND_MAP};
use super::query::{Query, QueryOptions, QueryType};
use crate::util::memory_pool::get_string_vec;

#[derive(Parser)]
#[grammar = "tester/mysql_test.pest"]
pub struct PestMySQLParser;

pub struct PestParser {
    delimiter: String,
}

impl Default for PestParser {
    fn default() -> Self {
        Self {
            delimiter: ";".to_string(),
        }
    }
}

impl PestParser {
    pub fn new() -> Self {
        Self::default()
    }

    /// Convert pest parse tree to Query objects (memory pool optimized)
    fn convert_to_queries(&mut self, pairs: pest::iterators::Pairs<Rule>) -> Result<Vec<Query>> {
        let mut queries = Vec::new();
        let mut pending_sql_lines = get_string_vec();  // Use memory pool for SQL lines
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
                Rule::line => {
                    // Recursively process the line contents
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
                Rule::sql_statement => {
                    let sql_content = self.extract_sql_statement(pair)?;
                    log::debug!("Parsing SQL statement at line {}: '{}'", line_num, sql_content);
                    
                    // Check if this is actually our new syntax that wasn't caught by Pest
                    if let Some(query) = self.try_parse_new_syntax(&sql_content, line_num)? {
                        // Finalize any pending SQL first
                        if !pending_sql_lines.is_empty() {
                            self.finalize_pending_sql(&mut queries, &mut pending_sql_lines, line_num)?;
                        }
                        log::debug!("Processed as new syntax query at line {}", line_num);
                        queries.push(query);
                    } else {
                        // Process as regular SQL
                        log::debug!("Processing as regular SQL at line {}: '{}'", line_num, sql_content);
                        self.process_sql_line(
                            &mut pending_sql_lines,
                            &mut queries,
                            sql_content,
                            line_num,
                        )?;
                    }
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
                Rule::inc_stmt => {
                    // Finalize any pending SQL before processing inc statement
                    if !pending_sql_lines.is_empty() {
                        self.finalize_pending_sql(&mut queries, &mut pending_sql_lines, line_num)?;
                    }

                    // Extract variable name from "inc $varname"
                    let text = pair.as_str().trim();
                    let var_name = text.split_whitespace().nth(1).unwrap_or("").to_string();
                    queries.push(Query {
                        query_type: QueryType::Inc,
                        query: var_name,
                        line: line_num,
                        options: QueryOptions::default(),
                    });
                }
                Rule::inc_operation => {
                    // Finalize any pending SQL before processing inc operation
                    if !pending_sql_lines.is_empty() {
                        self.finalize_pending_sql(&mut queries, &mut pending_sql_lines, line_num)?;
                    }

                    // Extract variable name from "inc $varname"
                    let text = pair.as_str();
                    let var_name = text.split_whitespace().nth(1).unwrap_or("").to_string();
                    queries.push(Query {
                        query_type: QueryType::Inc,
                        query: var_name,
                        line: line_num,
                        options: QueryOptions::default(),
                    });
                }
                Rule::var_stmt => {
                    // Finalize any pending SQL before processing variable statement
                    if !pending_sql_lines.is_empty() {
                        self.finalize_pending_sql(&mut queries, &mut pending_sql_lines, line_num)?;
                    }

                    // Process the inner var_operation
                    for inner in pair.into_inner() {
                        if let Rule::var_operation = inner.as_rule() {
                            let (query_type, query_content) = self.parse_var_operation(inner)?;
                            queries.push(Query {
                                query_type,
                                query: query_content,
                                line: line_num,
                                options: QueryOptions::default(),
                            });
                        }
                    }
                }
                Rule::var_operation => {
                    // Finalize any pending SQL before processing variable operation
                    if !pending_sql_lines.is_empty() {
                        self.finalize_pending_sql(&mut queries, &mut pending_sql_lines, line_num)?;
                    }

                    let (query_type, query_content) = self.parse_var_operation(pair)?;
                    queries.push(Query {
                        query_type,
                        query: query_content,
                        line: line_num,
                        options: QueryOptions::default(),
                    });
                }
                Rule::batch_operation => {
                    // Finalize any pending SQL before processing batch operation
                    if !pending_sql_lines.is_empty() {
                        self.finalize_pending_sql(&mut queries, &mut pending_sql_lines, line_num)?;
                    }

                    let (query_type, query_content) = self.parse_batch_operation(pair)?;
                    queries.push(Query {
                        query_type,
                        query: query_content,
                        line: line_num,
                        options: QueryOptions::default(),
                    });
                }
                Rule::transaction_operation => {
                    // Finalize any pending SQL before processing transaction operation
                    if !pending_sql_lines.is_empty() {
                        self.finalize_pending_sql(&mut queries, &mut pending_sql_lines, line_num)?;
                    }

                    let (query_type, query_content) = self.parse_transaction_operation(pair)?;
                    queries.push(Query {
                        query_type,
                        query: query_content,
                        line: line_num,
                        options: QueryOptions::default(),
                    });
                }
                Rule::simple_command => {
                    // Finalize any pending SQL before processing simple command
                    if !pending_sql_lines.is_empty() {
                        self.finalize_pending_sql(&mut queries, &mut pending_sql_lines, line_num)?;
                    }

                    let (query_type, query_content) = self.parse_simple_command(pair)?;
                    queries.push(Query {
                        query_type,
                        query: query_content,
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

            // Finalize the SQL statement with smart splitting
            self.finalize_pending_sql_with_splitting(queries, pending_sql_lines, line_num)?;
        } else {
            // Add to pending lines (multi-line SQL continues)
            pending_sql_lines.push(trimmed_content.to_string());
        }

        Ok(())
    }

    /// Finalize pending SQL lines with smart splitting for multiple statements
    fn finalize_pending_sql_with_splitting(
        &self,
        queries: &mut Vec<Query>,
        pending_sql_lines: &mut Vec<String>,
        line_num: usize,
    ) -> Result<()> {
        if pending_sql_lines.is_empty() {
            return Ok(());
        }

        let full_sql = pending_sql_lines.join("\n").trim().to_string();
        if full_sql.is_empty() {
            pending_sql_lines.clear();
            return Ok(());
        }

        // Check if this contains multiple complete SQL statements
        // Look for pattern: semicolon + newline + capital letter (start of new SQL)
        let lines: Vec<&str> = full_sql.lines().collect();
        let mut statements = Vec::new();
        let mut current_statement = String::new();
        
        for (i, line) in lines.iter().enumerate() {
            let trimmed_line = line.trim();
            if trimmed_line.is_empty() {
                continue;
            }
            
            current_statement.push_str(line);
            current_statement.push('\n');
            
            // Check if this line ends with semicolon and next line starts new statement
            if trimmed_line.ends_with(';') {
                if i + 1 < lines.len() {
                    // Look for next non-empty line
                    let mut next_line_idx = i + 1;
                    while next_line_idx < lines.len() {
                        let next_line = lines[next_line_idx].trim();
                        if !next_line.is_empty() {
                            // Check if next line starts with SQL keywords (case insensitive)
                            let first_word = next_line.split_whitespace().next().unwrap_or("").to_lowercase();
                            let sql_keywords = ["select", "insert", "update", "delete", "create", "drop", "alter", "show", "describe", "explain"];
                            if sql_keywords.contains(&first_word.as_str()) {
                                // This ends a statement and next line starts a new one
                                statements.push(current_statement.trim().to_string());
                                current_statement.clear();
                                break;
                            } else {
                                // Not a SQL statement start, continue current statement
                                break;
                            }
                        }
                        next_line_idx += 1;
                    }
                    if next_line_idx >= lines.len() {
                        // End of input, finalize current statement
                        statements.push(current_statement.trim().to_string());
                        current_statement.clear();
                    }
                } else {
                    // End of input, finalize current statement
                    statements.push(current_statement.trim().to_string());
                    current_statement.clear();
                }
            }
        }
        
        // Add any remaining statement
        if !current_statement.trim().is_empty() {
            statements.push(current_statement.trim().to_string());
        }
        
        if statements.len() > 1 {
            // Multiple statements detected, split them
            log::debug!("Splitting {} statements from multi-line SQL at line {}", statements.len(), line_num);
            for (i, statement) in statements.iter().enumerate() {
                let trimmed_statement = statement.trim();
                if !trimmed_statement.is_empty() {
                    queries.push(Query {
                        query_type: QueryType::Query,
                        query: trimmed_statement.to_string(),
                        line: line_num + i, // Adjust line numbers
                        options: QueryOptions::default(),
                    });
                }
            }
        } else {
            // Single statement, process normally
            log::debug!("Finalizing single multi-line SQL at line {}: '{}'", line_num, full_sql);
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

    /// Finalize pending SQL lines into a single Query (original method)
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
            log::debug!("Finalizing multi-line SQL at line {}: '{}'", line_num, full_sql);
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

    /// Try to parse new syntax that wasn't caught by Pest grammar
    fn try_parse_new_syntax(&self, content: &str, line_num: usize) -> Result<Option<Query>> {
        let content = content.trim();
        
        // Variable operations
        if content.starts_with("inc ") {
            let parts: Vec<&str> = content.split_whitespace().collect();
            if parts.len() == 2 && parts[1].starts_with('$') {
                return Ok(Some(Query {
                    query_type: QueryType::Inc,
                    query: parts[1].to_string(),
                    line: line_num,
                    options: QueryOptions::default(),
                }));
            }
        } else if content.starts_with("dec ") {
            let parts: Vec<&str> = content.split_whitespace().collect();
            if parts.len() == 2 && parts[1].starts_with('$') {
                return Ok(Some(Query {
                    query_type: QueryType::Dec,
                    query: parts[1].to_string(),
                    line: line_num,
                    options: QueryOptions::default(),
                }));
            }
        } else if content.starts_with("add ") {
            let parts: Vec<&str> = content.split(',').collect();
            if parts.len() == 2 {
                let var_part = parts[0].trim().split_whitespace().nth(1).unwrap_or("");
                let value_part = parts[1].trim();
                if var_part.starts_with('$') {
                    let args = format!("{}, {}", var_part, value_part);
                    return Ok(Some(Query {
                        query_type: QueryType::Add,
                        query: args,
                        line: line_num,
                        options: QueryOptions::default(),
                    }));
                }
            }
        } else if content.starts_with("sub ") {
            let parts: Vec<&str> = content.split(',').collect();
            if parts.len() == 2 {
                let var_part = parts[0].trim().split_whitespace().nth(1).unwrap_or("");
                let value_part = parts[1].trim();
                if var_part.starts_with('$') {
                    let args = format!("{}, {}", var_part, value_part);
                    return Ok(Some(Query {
                        query_type: QueryType::Sub,
                        query: args,
                        line: line_num,
                        options: QueryOptions::default(),
                    }));
                }
            }
        }
        // Batch operations
        else if content.starts_with("batch_insert ") {
            let table = content.strip_prefix("batch_insert ").unwrap_or("").trim();
            return Ok(Some(Query {
                query_type: QueryType::BatchInsert,
                query: table.to_string(),
                line: line_num,
                options: QueryOptions::default(),
            }));
        } else if content == "batch_execute" {
            return Ok(Some(Query {
                query_type: QueryType::BatchExecute,
                query: String::new(),
                line: line_num,
                options: QueryOptions::default(),
            }));
        } else if content == "end_batch" {
            return Ok(Some(Query {
                query_type: QueryType::EndBatch,
                query: String::new(),
                line: line_num,
                options: QueryOptions::default(),
            }));
        }
        // Transaction operations
        else if content == "begin_transaction" {
            return Ok(Some(Query {
                query_type: QueryType::BeginTransaction,
                query: String::new(),
                line: line_num,
                options: QueryOptions::default(),
            }));
        } else if content == "commit_transaction" {
            return Ok(Some(Query {
                query_type: QueryType::CommitTransaction,
                query: String::new(),
                line: line_num,
                options: QueryOptions::default(),
            }));
        } else if content == "rollback_transaction" {
            return Ok(Some(Query {
                query_type: QueryType::RollbackTransaction,
                query: String::new(),
                line: line_num,
                options: QueryOptions::default(),
            }));
        }
        // Simple commands without -- prefix
        else if content.starts_with("echo ") {
            let echo_content = content.strip_prefix("echo ").unwrap_or("");
            return Ok(Some(Query {
                query_type: QueryType::Echo,
                query: echo_content.to_string(),
                line: line_num,
                options: QueryOptions::default(),
            }));
        } else if content.starts_with("sleep ") {
            let sleep_content = content.strip_prefix("sleep ").unwrap_or("");
            return Ok(Some(Query {
                query_type: QueryType::Sleep,
                query: sleep_content.to_string(),
                line: line_num,
                options: QueryOptions::default(),
            }));
        } else if content.starts_with("error ") {
            let error_content = content.strip_prefix("error ").unwrap_or("");
            return Ok(Some(Query {
                query_type: QueryType::Error,
                query: error_content.to_string(),
                line: line_num,
                options: QueryOptions::default(),
            }));
        } else if content == "sorted_result" {
            return Ok(Some(Query {
                query_type: QueryType::SortedResult,
                query: String::new(),
                line: line_num,
                options: QueryOptions::default(),
            }));
        } else if content.starts_with("source ") {
            let source_content = content.strip_prefix("source ").unwrap_or("");
            return Ok(Some(Query {
                query_type: QueryType::Source,
                query: source_content.to_string(),
                line: line_num,
                options: QueryOptions::default(),
            }));
        }
        
        // Not our new syntax
        Ok(None)
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
        let (command_name, command_args) = {
            // Find index of first whitespace or '('
            let mut split_idx = command_content.len();
            for (idx, ch) in command_content.char_indices() {
                if ch.is_whitespace() || ch == '(' {
                    split_idx = idx;
                    break;
                }
            }
            let cmd = command_content[..split_idx].to_lowercase();
            let args = command_content[split_idx..].trim_start().to_string();
            (cmd, args)
        };

        // Map command name to QueryType (reuse the logic from handwritten parser)
        let query_type = self.map_command_to_query_type(&command_name);
        Ok((query_type, command_args))
    }

    fn map_command_to_query_type(&self, command: &str) -> QueryType {
        COMMAND_MAP.get(command).copied().unwrap_or(QueryType::Unknown)
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

    #[allow(dead_code)]
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

    // === New parsing methods for enhanced syntax ===

    fn parse_var_operation(&self, pair: pest::iterators::Pair<Rule>) -> Result<(QueryType, String)> {
        let full_text = pair.as_str().trim();
        
        if full_text.starts_with("inc ") {
            Ok((QueryType::Inc, full_text[4..].trim().to_string()))
        } else if full_text.starts_with("dec ") {
            Ok((QueryType::Dec, full_text[4..].trim().to_string()))
        } else if full_text.starts_with("add ") {
            Ok((QueryType::Add, full_text[4..].trim().to_string()))
        } else if full_text.starts_with("sub ") {
            Ok((QueryType::Sub, full_text[4..].trim().to_string()))
        } else {
            Err(anyhow!("Unknown variable operation: {}", full_text))
        }
    }

    fn parse_batch_operation(&self, pair: pest::iterators::Pair<Rule>) -> Result<(QueryType, String)> {
        let full_text = pair.as_str().trim();
        
        if full_text.starts_with("batch_insert ") {
            Ok((QueryType::BatchInsert, full_text[13..].trim().to_string()))
        } else if full_text.starts_with("batch_execute") {
            Ok((QueryType::BatchExecute, String::new()))
        } else if full_text.starts_with("end_batch") {
            Ok((QueryType::EndBatch, String::new()))
        } else {
            Err(anyhow!("Unknown batch operation: {}", full_text))
        }
    }

    fn parse_transaction_operation(&self, pair: pest::iterators::Pair<Rule>) -> Result<(QueryType, String)> {
        let full_text = pair.as_str().trim();
        
        if full_text.starts_with("begin_transaction") {
            Ok((QueryType::BeginTransaction, String::new()))
        } else if full_text.starts_with("commit_transaction") {
            Ok((QueryType::CommitTransaction, String::new()))
        } else if full_text.starts_with("rollback_transaction") {
            Ok((QueryType::RollbackTransaction, String::new()))
        } else {
            Err(anyhow!("Unknown transaction operation: {}", full_text))
        }
    }

    fn parse_simple_command(&self, pair: pest::iterators::Pair<Rule>) -> Result<(QueryType, String)> {
        let full_text = pair.as_str().trim();
        
        if full_text.starts_with("echo ") {
            Ok((QueryType::Echo, full_text[5..].to_string()))
        } else if full_text.starts_with("sleep ") {
            Ok((QueryType::Sleep, full_text[6..].trim().to_string()))
        } else if full_text.starts_with("error ") {
            Ok((QueryType::Error, full_text[6..].to_string()))
        } else if full_text == "sorted_result" {
            Ok((QueryType::SortedResult, String::new()))
        } else if full_text.starts_with("source ") {
            Ok((QueryType::Source, full_text[7..].to_string()))
        } else {
            Err(anyhow!("Unknown simple command: {}", full_text))
        }
    }
}

impl QueryParser for PestParser {
    fn parse(&mut self, content: &str) -> Result<Vec<Query>> {
        let pairs = PestMySQLParser::parse(Rule::test_file, content)
            .map_err(|e| anyhow!("Pest parsing error: {}", e))?;

        self.convert_to_queries(pairs)
    }
}

#[cfg(test)]
mod tests {
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
