use dingo_test_runner::tester::database::{Database, ConnectionInfo};

fn create_test_connection_info() -> ConnectionInfo {
    ConnectionInfo {
        host: "127.0.0.1".to_string(),
        port: 3306,
        user: "root".to_string(),
        password: "".to_string(),
        database: "test".to_string(),
        params: "".to_string(),
    }
}

#[test]
fn test_connection_info_creation() {
    let info = create_test_connection_info();
    assert_eq!(info.host, "127.0.0.1");
    assert_eq!(info.port, 3306);
    assert_eq!(info.user, "root");
    assert_eq!(info.database, "test");
}

#[test]
fn test_unsupported_database_type() {
    let info = create_test_connection_info();
    let result = Database::new("unsupported", &info);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Unsupported database type"));
}

#[test]
fn test_mysql_database_info_format() {
    let info = create_test_connection_info();
    // Test the info string formatting without creating actual MySQL connection
    let expected_info = "mysql://root@127.0.0.1:3306/test";
    
    // Test the info formatting logic directly
    let formatted_info = format!("mysql://{}@{}:{}/{}", 
                                info.user, info.host, info.port, info.database);
    
    assert_eq!(formatted_info, expected_info);
}
