mod pest_integration_tests {
    use dingo_test_runner::tester::parser::{create_parser, default_parser};
    use dingo_test_runner::tester::query::QueryType;

    #[test]
    fn test_pest_parser_basic_functionality() {
        let mut parser = create_parser("pest").expect("Failed to create pest parser");

        let content = r#"
# This is a comment
--echo Hello World
SELECT 1;
--delimiter //
SELECT 2//
if ($var > 0) {
    --echo positive
}
"#;

        let queries = parser.parse(content).expect("Failed to parse content");

        // Should have multiple queries
        assert!(queries.len() > 0);

        // Check for comment
        let comment_queries: Vec<_> = queries
            .iter()
            .filter(|q| q.query_type == QueryType::Comment)
            .collect();
        assert!(!comment_queries.is_empty());

        // Check for echo command
        let echo_queries: Vec<_> = queries
            .iter()
            .filter(|q| q.query_type == QueryType::Echo)
            .collect();
        assert!(!echo_queries.is_empty());
        assert_eq!(echo_queries[0].query, "Hello World");

        // Check for SQL query
        let sql_queries: Vec<_> = queries
            .iter()
            .filter(|q| q.query_type == QueryType::Query)
            .collect();
        assert!(!sql_queries.is_empty());

        // Check for delimiter change
        let delimiter_queries: Vec<_> = queries
            .iter()
            .filter(|q| q.query_type == QueryType::Delimiter)
            .collect();
        assert!(!delimiter_queries.is_empty());
        assert_eq!(delimiter_queries[0].query, "//");

        // Check for if statement
        let if_queries: Vec<_> = queries
            .iter()
            .filter(|q| q.query_type == QueryType::If)
            .collect();
        assert!(!if_queries.is_empty());
        assert_eq!(if_queries[0].query.trim(), "$var > 0");
    }

    #[test]
    fn test_default_parser_uses_pest() {
        let mut parser = default_parser();
        let content = "--echo test";
        let queries = parser
            .parse(content)
            .expect("Failed to parse with default parser");

        assert_eq!(queries.len(), 1);
        assert_eq!(queries[0].query_type, QueryType::Echo);
        assert_eq!(queries[0].query, "test");
    }

    #[test]
    fn test_pest_parser_comprehensive() {
        let test_cases = vec![
            ("SELECT 1;", QueryType::Query, "SELECT 1"),
            ("--echo hello", QueryType::Echo, "hello"),
            ("# comment", QueryType::Comment, "#comment"),
            ("let $x = 5", QueryType::Let, "$x = 5"),
            ("--sleep 1", QueryType::Sleep, "1"),
        ];

        for (test_content, expected_type, expected_query) in test_cases {
            let mut parser = create_parser("pest").expect("Failed to create pest parser");
            let queries = parser.parse(test_content).expect("Failed to parse content");
            
            assert_eq!(queries.len(), 1, "Expected exactly one query for: {}", test_content);
            assert_eq!(queries[0].query_type, expected_type, "Query type mismatch for: {}", test_content);
            assert_eq!(queries[0].query.trim(), expected_query, "Query content mismatch for: {}", test_content);
        }
    }
}
