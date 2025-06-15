use dingo_test_runner::tester::connection_manager::{ConnectionManager, ConnectParams};
use dingo_test_runner::tester::database::ConnectionInfo;

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
fn test_parse_connect_params_full() {
    let manager = ConnectionManager {
        connections: HashMap::new(),
        current_connection: "default".to_string(),
        default_connection_info: create_test_connection_info(),
        max_retries: 1,
    };

    let params = manager.parse_connect_params("(conn1,localhost,user,pass,db,3307)")
        .expect("Failed to parse full connection parameters");
    assert_eq!(params.connection_name, "conn1");
    assert_eq!(params.host, "localhost");
    assert_eq!(params.user, "user");
    assert_eq!(params.password, "pass");
    assert_eq!(params.database, "db");
    assert_eq!(params.port, "3307");
}

#[test]
fn test_parse_connect_params_minimal() {
    let manager = ConnectionManager {
        connections: HashMap::new(),
        current_connection: "default".to_string(),
        default_connection_info: create_test_connection_info(),
        max_retries: 1,
    };

    let params = manager.parse_connect_params("(conn1)")
        .expect("Failed to parse minimal connection parameters");
    assert_eq!(params.connection_name, "conn1");
    assert_eq!(params.host, "");
    assert_eq!(params.user, "");
}

#[test]
fn test_build_connection_info_with_defaults() {
    let manager = ConnectionManager {
        connections: HashMap::new(),
        current_connection: "default".to_string(),
        default_connection_info: create_test_connection_info(),
        max_retries: 1,
    };

    let params = ConnectParams {
        connection_name: "test_conn".to_string(),
        host: "".to_string(),
        user: "".to_string(),
        password: "".to_string(),
        database: "".to_string(),
        port: "".to_string(),
    };

    let info = manager.build_connection_info(&params)
        .expect("Failed to build connection info with defaults");
    assert_eq!(info.host, "127.0.0.1");
    assert_eq!(info.user, "root");
    assert_eq!(info.port, 3306);
}

#[test]
fn test_connection_manager_interface() {
    // This test would require actual database connections
    // In a real implementation, we would mock the database layer
    // For now, we test the logic that doesn't require database access
    
    let connection_info = create_test_connection_info();
    
    // Test the connection name parsing and validation logic
    let manager = ConnectionManager {
        connections: HashMap::new(),
        current_connection: "default".to_string(),
        default_connection_info: connection_info,
        max_retries: 1,
    };

    // Test list connections
    let connections = manager.list_connections();
    assert_eq!(connections.len(), 0); // No connections added yet

    // Test current connection info
    let info = manager.current_connection_info();
    assert!(info.contains("default"));
}
