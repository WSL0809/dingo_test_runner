#[cfg(feature = "pest")]
mod debug_pest {
    use dingo_test_runner::tester::parser::{create_parser};

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
} 