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
            println!(
                "Query {}: type={:?}, content='{}', line={}",
                i, query.query_type, query.query, query.line
            );
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
            println!(
                "Query {}: type={:?}, content='{}', line={}",
                i, query.query_type, query.query, query.line
            );
        }

        // This test always passes, it's just for debugging
        assert!(true);
    }

    #[test]
    fn debug_pest_parser_advanced() {
        let mut parser = create_parser("pest").expect("Failed to create pest parser");

        let content = "--echo Hello World";
        println!("Parsing content: '{}'", content);

        let queries = parser
            .parse(content)
            .expect("Failed to parse with pest");

        println!("Pest parser results:");
        for (i, query) in queries.iter().enumerate() {
            println!(
                "  Query {}: type={:?}, content='{}'",
                i, query.query_type, query.query
            );
        }

        assert!(true);
    }

    #[test]
    fn debug_pest_parsing_complex() {
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
            println!(
                "{}: type={:?}, content='{}'",
                i, query.query_type, query.query
            );
        }

        // Verify we got some queries
        assert!(queries.len() > 0, "Should parse at least one query");
    }
}
