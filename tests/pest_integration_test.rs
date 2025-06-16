#[cfg(feature = "pest")]
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
        let comment_queries: Vec<_> = queries.iter().filter(|q| q.query_type == QueryType::Comment).collect();
        assert!(!comment_queries.is_empty());
        
        // Check for echo command
        let echo_queries: Vec<_> = queries.iter().filter(|q| q.query_type == QueryType::Echo).collect();
        assert!(!echo_queries.is_empty());
        assert_eq!(echo_queries[0].query, "Hello World");
        
        // Check for SQL query
        let sql_queries: Vec<_> = queries.iter().filter(|q| q.query_type == QueryType::Query).collect();
        assert!(!sql_queries.is_empty());
        
        // Check for delimiter change
        let delimiter_queries: Vec<_> = queries.iter().filter(|q| q.query_type == QueryType::Delimiter).collect();
        assert!(!delimiter_queries.is_empty());
        assert_eq!(delimiter_queries[0].query, "//");
        
        // Check for if statement
        let if_queries: Vec<_> = queries.iter().filter(|q| q.query_type == QueryType::If).collect();
        assert!(!if_queries.is_empty());
        assert_eq!(if_queries[0].query.trim(), "$var > 0");
    }

    #[test]
    fn test_default_parser_uses_pest_when_enabled() {
        let mut parser = default_parser();
        let content = "--echo test";
        let queries = parser.parse(content).expect("Failed to parse with default parser");
        
        assert_eq!(queries.len(), 1);
        assert_eq!(queries[0].query_type, QueryType::Echo);
        assert_eq!(queries[0].query, "test");
    }

    #[test]
    fn test_pest_vs_handwritten_parser_compatibility() {
        let test_cases = vec![
            "SELECT 1;",
            "--echo hello",
            "# comment",
            "--delimiter //\nSELECT 1//",
            "if ($x > 0) {\n--echo positive\n}",
        ];

        for test_case in test_cases {
            let mut pest_parser = create_parser("pest").expect("Failed to create pest parser");
            let mut handwritten_parser = create_parser("handwritten").expect("Failed to create handwritten parser");
            
            let pest_result = pest_parser.parse(test_case);
            let handwritten_result = handwritten_parser.parse(test_case);
            
            // Both should succeed or both should fail
            match (pest_result, handwritten_result) {
                (Ok(pest_queries), Ok(handwritten_queries)) => {
                    // Compare basic structure (number of queries and types)
                    assert_eq!(pest_queries.len(), handwritten_queries.len(), 
                              "Query count mismatch for: {}", test_case);
                    
                    for (pest_q, handwritten_q) in pest_queries.iter().zip(handwritten_queries.iter()) {
                        assert_eq!(pest_q.query_type, handwritten_q.query_type,
                                  "Query type mismatch for: {}", test_case);
                    }
                }
                (Err(_), Err(_)) => {
                    // Both failed, which is acceptable for some edge cases
                }
                _ => {
                    panic!("Parser results inconsistent for: {}", test_case);
                }
            }
        }
    }
} 