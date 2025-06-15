use dingo_test_runner::tester::parser::{Parser, QueryType};

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
