#[cfg(feature = "pest")]
mod debug_pest {
    use dingo_test_runner::tester::parser::{create_parser, QueryParser};

    #[test]
    fn debug_pest_parser() {
        let mut parser = create_parser("pest").expect("Failed to create pest parser");
        
        let content = "--echo Hello World";
        println!("Parsing content: '{}'", content);
        let queries = parser.parse(content).expect("Failed to parse content");
        
        println!("Parsed {} queries:", queries.len());
        for (i, query) in queries.iter().enumerate() {
            println!("Query {}: type={:?}, content='{}', line={}", i, query.query_type, query.query, query.line);
        }
        
        // This test always passes, it's just for debugging
        assert!(true);
    }

    #[test]
    fn debug_pest_parser_simple_sql() {
        let mut parser = create_parser("pest").expect("Failed to create pest parser");
        
        let content = "SELECT 1;";
        println!("Parsing SQL content: '{}'", content);
        let queries = parser.parse(content).expect("Failed to parse content");
        
        println!("Parsed {} queries from SQL:", queries.len());
        for (i, query) in queries.iter().enumerate() {
            println!("Query {}: type={:?}, content='{}', line={}", i, query.query_type, query.query, query.line);
        }
        
        // This test always passes, it's just for debugging
        assert!(true);
    }

    #[test]
    fn debug_pest_parser_handwritten_comparison() {
        let mut pest_parser = create_parser("pest").expect("Failed to create pest parser");
        let mut handwritten_parser = create_parser("handwritten").expect("Failed to create handwritten parser");
        
        let content = "--echo Hello World";
        println!("Comparing parsers for content: '{}'", content);
        
        let pest_queries = pest_parser.parse(content).expect("Failed to parse with pest");
        let handwritten_queries = handwritten_parser.parse(content).expect("Failed to parse with handwritten");
        
        println!("Pest parser results:");
        for (i, query) in pest_queries.iter().enumerate() {
            println!("  Query {}: type={:?}, content='{}'", i, query.query_type, query.query);
        }
        
        println!("Handwritten parser results:");
        for (i, query) in handwritten_queries.iter().enumerate() {
            println!("  Query {}: type={:?}, content='{}'", i, query.query_type, query.query);
        }
        
        assert!(true);
    }

    #[test]
    fn debug_pest_parsing() {
        let content = r#"--echo # concurrent_advanced.test - 高级并发功能测试
--echo # 演示并发块与变量、连接管理的综合应用

--let $table_name = concurrent_advanced_test
CREATE TABLE $table_name (
    id INT PRIMARY KEY,
    name VARCHAR(50),
    value INT
);"#;

        let mut parser = create_parser("pest").unwrap();
        let queries = parser.parse(content).unwrap();
        
        println!("\n=== Pest Parser Results ===");
        for (i, query) in queries.iter().enumerate() {
            println!("{}: type={:?}, content='{}'", i, query.query_type, query.query);
        }
        
        // Also test with handwritten parser for comparison
        let mut handwritten_parser = create_parser("handwritten").unwrap();
        let handwritten_queries = handwritten_parser.parse(content).unwrap();
        
        println!("\n=== Handwritten Parser Results ===");
        for (i, query) in handwritten_queries.iter().enumerate() {
            println!("{}: type={:?}, content='{}'", i, query.query_type, query.query);
        }
        
        // Compare results
        assert_eq!(queries.len(), handwritten_queries.len(), "Parser results should have same length");
        for (i, (pest_q, handwritten_q)) in queries.iter().zip(handwritten_queries.iter()).enumerate() {
            assert_eq!(pest_q.query_type, handwritten_q.query_type, "Query type mismatch at index {}", i);
            
            // Normalize whitespace for content comparison
            let pest_content_normalized = normalize_whitespace(&pest_q.query);
            let handwritten_content_normalized = normalize_whitespace(&handwritten_q.query);
            assert_eq!(pest_content_normalized, handwritten_content_normalized, "Query content mismatch at index {}", i);
        }
    }

    // Helper function to normalize whitespace for comparison
    fn normalize_whitespace(content: &str) -> String {
        content
            .lines()
            .map(|line| line.trim())
            .filter(|line| !line.is_empty())
            .collect::<Vec<_>>()
            .join("\n")
    }
} 