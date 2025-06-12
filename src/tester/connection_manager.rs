//! Connection Manager for MySQL Test Runner
//! 
//! This module manages multiple database connections for test execution,
//! allowing tests to create, switch between, and manage multiple database connections.

use super::database::{Database, ConnectionInfo, create_database_with_retry};
use anyhow::{anyhow, Result};
use log::{debug, info, warn};
use std::collections::HashMap;

/// Connection manager for handling multiple database connections
pub struct ConnectionManager {
    /// Map of connection name to database instance
    connections: HashMap<String, Database>,
    /// Current active connection name
    current_connection: String,
    /// Default connection parameters
    default_connection_info: ConnectionInfo,
    /// Database type (mysql or sqlite)
    database_type: String,
    /// Maximum retry count for connections
    max_retries: u32,
}

const DEFAULT_CONNECTION_NAME: &str = "default";

impl ConnectionManager {
    /// Create a new connection manager with default connection
    pub fn new(
        database_type: &str,
        default_connection_info: ConnectionInfo,
        max_retries: u32,
    ) -> Result<Self> {
        let mut connections = HashMap::new();
        
        // Create default connection
        let default_db = create_database_with_retry(
            database_type,
            &default_connection_info,
            max_retries,
        )?;
        
        connections.insert(DEFAULT_CONNECTION_NAME.to_string(), default_db);
        
        Ok(ConnectionManager {
            connections,
            current_connection: DEFAULT_CONNECTION_NAME.to_string(),
            default_connection_info,
            database_type: database_type.to_string(),
            max_retries,
        })
    }

    /// Get the current active database connection
    pub fn current_database(&mut self) -> Result<&mut Database> {
        self.connections
            .get_mut(&self.current_connection)
            .ok_or_else(|| anyhow!("Current connection '{}' not found", self.current_connection))
    }

    /// Connect to a new database with given parameters
    /// Syntax: connect (conn_name, host, user, password, database, port)
    pub fn connect(&mut self, params: &str) -> Result<()> {
        let parsed_params = self.parse_connect_params(params)?;
        
        // Check if connection already exists
        if self.connections.contains_key(&parsed_params.connection_name) {
            return Err(anyhow!("Connection '{}' already exists", parsed_params.connection_name));
        }

        // Create connection info
        let connection_info = self.build_connection_info(&parsed_params)?;
        
        // Create the database connection
        // Use fewer retries for connection commands to fail fast on obvious errors
        let database = create_database_with_retry(
            &self.database_type,
            &connection_info,
            std::cmp::min(self.max_retries, 3), // Maximum 3 retries for connect commands
        )?;

        // Store the connection and switch to it
        self.connections.insert(parsed_params.connection_name.clone(), database);
        self.current_connection = parsed_params.connection_name;
        
        info!("Created and switched to connection '{}'", self.current_connection);
        Ok(())
    }

    /// Switch to an existing connection
    pub fn switch_connection(&mut self, conn_name: &str) -> Result<()> {
        if !self.connections.contains_key(conn_name) {
            return Err(anyhow!("Connection '{}' does not exist", conn_name));
        }
        
        self.current_connection = conn_name.to_string();
        info!("Switched to connection '{}'", conn_name);
        Ok(())
    }

    /// Disconnect a named connection
    pub fn disconnect(&mut self, conn_name: &str) -> Result<()> {
        // Prevent disconnecting the default connection
        if conn_name == DEFAULT_CONNECTION_NAME {
            return Err(anyhow!("Cannot disconnect the default connection"));
        }

        // Check if connection exists
        if !self.connections.contains_key(conn_name) {
            return Err(anyhow!("Connection '{}' does not exist", conn_name));
        }

        // Remove the connection
        self.connections.remove(conn_name);
        
        // If we were using this connection, switch back to default
        if self.current_connection == conn_name {
            self.current_connection = DEFAULT_CONNECTION_NAME.to_string();
            info!("Disconnected '{}' and switched back to default connection", conn_name);
        } else {
            info!("Disconnected connection '{}'", conn_name);
        }
        
        Ok(())
    }

    /// Get information about current connection
    pub fn current_connection_info(&self) -> String {
        format!("Current connection: {}", self.current_connection)
    }

    /// List all active connections
    pub fn list_connections(&self) -> Vec<String> {
        self.connections.keys().cloned().collect()
    }

    /// Parse connect command parameters
    /// Format: (conn_name, host, user, password, database, port)
    fn parse_connect_params(&self, params: &str) -> Result<ConnectParams> {
        // Remove parentheses and split by comma
        let trimmed = params.trim().trim_start_matches('(').trim_end_matches(')');
        let parts: Vec<&str> = trimmed.split(',').map(|s| s.trim()).collect();
        
        if parts.is_empty() {
            return Err(anyhow!("Connection name is required"));
        }

        Ok(ConnectParams {
            connection_name: parts.get(0).unwrap_or(&"").to_string(),
            host: parts.get(1).unwrap_or(&"").to_string(),
            user: parts.get(2).unwrap_or(&"").to_string(),
            password: parts.get(3).unwrap_or(&"").to_string(),
            database: parts.get(4).unwrap_or(&"").to_string(),
            port: parts.get(5).unwrap_or(&"").to_string(),
        })
    }

    /// Build connection info from parsed parameters
    fn build_connection_info(&self, params: &ConnectParams) -> Result<ConnectionInfo> {
        // Use defaults for empty parameters
        let host = if params.host.is_empty() {
            self.default_connection_info.host.clone()
        } else {
            params.host.clone()
        };

        let user = if params.user.is_empty() {
            self.default_connection_info.user.clone()
        } else {
            params.user.clone()
        };

        let password = if params.password.is_empty() {
            self.default_connection_info.password.clone()
        } else {
            params.password.clone()
        };

        let database = if params.database.is_empty() {
            self.default_connection_info.database.clone()
        } else {
            params.database.clone()
        };

        let port = if params.port.is_empty() {
            self.default_connection_info.port
        } else {
            params.port.parse::<u16>()
                .map_err(|_| anyhow!("Invalid port: {}", params.port))?
        };

        // For SQLite, use the database parameter as the file path
        let sqlite_file = if self.database_type == "sqlite" {
            if params.database.is_empty() {
                ":memory:".to_string()
            } else {
                params.database.clone()
            }
        } else {
            self.default_connection_info.sqlite_file.clone()
        };

        Ok(ConnectionInfo {
            host,
            port,
            user,
            password,
            database,
            params: self.default_connection_info.params.clone(),
            sqlite_file,
        })
    }
}

/// Parsed connection parameters
#[derive(Debug, Clone)]
struct ConnectParams {
    connection_name: String,
    host: String,
    user: String,
    password: String,
    database: String,
    port: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_connection_info() -> ConnectionInfo {
        ConnectionInfo {
            host: "127.0.0.1".to_string(),
            port: 3306,
            user: "root".to_string(),
            password: "".to_string(),
            database: "test".to_string(),
            params: "".to_string(),
            sqlite_file: ":memory:".to_string(),
        }
    }

    #[test]
    fn test_parse_connect_params_full() {
        let manager = ConnectionManager {
            connections: HashMap::new(),
            current_connection: "default".to_string(),
            default_connection_info: create_test_connection_info(),
            database_type: "mysql".to_string(),
            max_retries: 1,
        };

        let params = manager.parse_connect_params("(conn1,localhost,user,pass,db,3307)").unwrap();
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
            database_type: "mysql".to_string(),
            max_retries: 1,
        };

        let params = manager.parse_connect_params("(conn1)").unwrap();
        assert_eq!(params.connection_name, "conn1");
        assert_eq!(params.host, "");
        assert_eq!(params.user, "");
    }

    #[test]
    fn test_parse_connect_params_partial() {
        let manager = ConnectionManager {
            connections: HashMap::new(),
            current_connection: "default".to_string(),
            default_connection_info: create_test_connection_info(),
            database_type: "mysql".to_string(),
            max_retries: 1,
        };

        let params = manager.parse_connect_params("(conn1,host,,pass)").unwrap();
        assert_eq!(params.connection_name, "conn1");
        assert_eq!(params.host, "host");
        assert_eq!(params.user, "");
        assert_eq!(params.password, "pass");
    }

    #[test]
    fn test_build_connection_info_with_defaults() {
        let manager = ConnectionManager {
            connections: HashMap::new(),
            current_connection: "default".to_string(),
            default_connection_info: create_test_connection_info(),
            database_type: "mysql".to_string(),
            max_retries: 1,
        };

        let params = ConnectParams {
            connection_name: "test".to_string(),
            host: "".to_string(),
            user: "newuser".to_string(),
            password: "".to_string(),
            database: "".to_string(),
            port: "".to_string(),
        };

        let info = manager.build_connection_info(&params).unwrap();
        assert_eq!(info.host, "127.0.0.1"); // from default
        assert_eq!(info.user, "newuser"); // overridden
        assert_eq!(info.password, ""); // from default
        assert_eq!(info.database, "test"); // from default
        assert_eq!(info.port, 3306); // from default
    }

    #[test]
    fn test_sqlite_file_handling() {
        let manager = ConnectionManager {
            connections: HashMap::new(),
            current_connection: "default".to_string(),
            default_connection_info: create_test_connection_info(),
            database_type: "sqlite".to_string(),
            max_retries: 1,
        };

        let params = ConnectParams {
            connection_name: "file_conn".to_string(),
            host: "".to_string(),
            user: "".to_string(),
            password: "".to_string(),
            database: "/tmp/test.db".to_string(),
            port: "".to_string(),
        };

        let info = manager.build_connection_info(&params).unwrap();
        assert_eq!(info.sqlite_file, "/tmp/test.db");
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
            database_type: "sqlite".to_string(),
            max_retries: 1,
        };

        // Test list connections
        let connections = manager.list_connections();
        assert_eq!(connections.len(), 0); // No connections added yet

        // Test current connection info
        let info = manager.current_connection_info();
        assert!(info.contains("default"));
    }
} 