//! Parser for .test files
//!
//! This module handles parsing of MySQL test files using the Pest parser library.

use super::query::{Query, QueryType};
use anyhow::{anyhow, Result};
use phf::phf_map;

/// Trait for parsing .test files into Query vectors
///
/// This abstraction allows for different parser implementations (handwritten, pest, etc.)
/// while maintaining a consistent interface for the rest of the system.
pub trait QueryParser: Send + Sync {
    /// Parse a .test file content into a vector of queries
    fn parse(&mut self, content: &str) -> Result<Vec<Query>>;
}

/// Factory function to create the default parser implementation
pub fn default_parser() -> Box<dyn QueryParser> {
    Box::new(crate::tester::pest_parser::PestParser::new())
}

/// Create a parser by name (for future extensibility)
pub fn create_parser(parser_type: &str) -> Result<Box<dyn QueryParser>> {
    match parser_type.to_lowercase().as_str() {
        "pest" | "default" => Ok(Box::new(crate::tester::pest_parser::PestParser::new())),
        _ => Err(anyhow!("Unknown parser type: {}", parser_type)),
    }
}

/// Static command mapping from command strings to QueryType
/// This is used by the Pest parser for efficient command lookup
pub static COMMAND_MAP: phf::Map<&'static str, QueryType> = phf_map! {
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


#[cfg(test)]
mod tests {
    use super::*;
    use crate::tester::query::QueryType;

    #[test]
    fn test_parse_simple_query() {
        let mut parser = default_parser();
        let content = "SELECT 1;";
        let queries = parser.parse(content).expect("Failed to parse simple query");

        assert_eq!(queries.len(), 1);
        assert_eq!(queries[0].query_type, QueryType::Query);
        assert_eq!(queries[0].query, "SELECT 1");
    }

    #[test]
    fn test_parse_command() {
        let mut parser = default_parser();
        let content = "--echo hello world";
        let queries = parser.parse(content).expect("Failed to parse command");

        assert_eq!(queries.len(), 1);
        assert_eq!(queries[0].query_type, QueryType::Echo);
        assert_eq!(queries[0].query, "hello world");
    }

    #[test]
    fn test_parse_comment() {
        let mut parser = default_parser();
        let content = "# This is a comment";
        let queries = parser.parse(content).expect("Failed to parse comment");

        assert_eq!(queries.len(), 1);
        assert_eq!(queries[0].query_type, QueryType::Comment);
    }

    #[test]
    fn test_parse_multiline_query() {
        let mut parser = default_parser();
        let content = "SELECT 1\nFROM dual;";
        let queries = parser
            .parse(content)
            .expect("Failed to parse multiline query");

        assert_eq!(queries.len(), 1);
        assert_eq!(queries[0].query_type, QueryType::Query);
        assert_eq!(queries[0].query, "SELECT 1\nFROM dual");
    }

    #[test]
    fn test_delimiter_change() {
        let mut parser = default_parser();
        let content = "--delimiter //\nSELECT 1//\n--delimiter ;\nSELECT 2;";
        let queries = parser
            .parse(content)
            .expect("Failed to parse delimiter change");

        assert_eq!(queries.len(), 4);
        assert_eq!(queries[0].query_type, QueryType::Delimiter);
        assert_eq!(queries[0].query, "//");
        assert_eq!(queries[1].query_type, QueryType::Query);
        // The query may still contain the delimiter when parsed
        assert!(queries[1].query.contains("SELECT 1"));
        assert_eq!(queries[2].query_type, QueryType::Delimiter);
        assert_eq!(queries[2].query, ";");
        assert_eq!(queries[3].query_type, QueryType::Query);
        assert_eq!(queries[3].query, "SELECT 2");
    }
}
