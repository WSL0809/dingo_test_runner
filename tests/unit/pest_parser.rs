#[cfg(feature = "pest")]
mod pest_parser_tests {
    use dingo_test_runner::tester::parser::{create_parser};
    use dingo_test_runner::tester::query::QueryType;

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
        let queries = parser.parse(content).expect("Failed to parse delimiter change");
        
        assert!(queries.len() >= 2);
        
        // Find delimiter command
        let delimiter_query = queries.iter().find(|q| q.query_type == QueryType::Delimiter);
        assert!(delimiter_query.is_some());
        assert_eq!(delimiter_query.unwrap().query, "//");
        
        // Find SQL query
        let sql_query = queries.iter().find(|q| q.query_type == QueryType::Query);
        assert!(sql_query.is_some());
        assert_eq!(sql_query.unwrap().query.trim(), "SELECT 1");
    }
} 