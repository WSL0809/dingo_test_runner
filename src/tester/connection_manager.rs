//! Connection Manager for MySQL Test Runner
//! 
//! This module manages multiple database connections for test execution,
//! allowing tests to create, switch between, and manage multiple database connections.

use super::database::{Database, ConnectionInfo, create_database_with_retry};
use anyhow::{anyhow, Result};
use log::{debug, info};
use mysql::PooledConn;
use std::collections::HashMap;

/// Connection manager for handling multiple database connections
pub struct ConnectionManager {
    /// Map of connection name to database instance
    connections: HashMap<String, Database>,
    /// Current active connection name
    current_connection: String,
    /// Default connection parameters
    default_connection_info: ConnectionInfo,
    /// Maximum retry count for connections
    max_retries: u32,
}

const DEFAULT_CONNECTION_NAME: &str = "default";

impl ConnectionManager {
    /// Create a new connection manager with default connection
    pub fn new(
        default_connection_info: ConnectionInfo,
        max_retries: u32,
    ) -> Result<Self> {
        let mut connections = HashMap::new();
        
        // Create default connection (always MySQL for now)
        let default_db = create_database_with_retry(
            "mysql",
            &default_connection_info,
            max_retries,
        )?;
        
        connections.insert(DEFAULT_CONNECTION_NAME.to_string(), default_db);
        
        Ok(ConnectionManager {
            connections,
            current_connection: DEFAULT_CONNECTION_NAME.to_string(),
            default_connection_info,
            max_retries,
        })
    }

    /// Get a connection from the pool for concurrent execution.
    /// This returns a raw `PooledConn` which can be used in a separate thread.
    /// It uses the pool from the current active connection, which should have the correct database context.
    pub fn get_pooled_connection(&self) -> Result<PooledConn> {
        self.connections
            .get(&self.current_connection)
            .ok_or_else(|| anyhow!("Current connection '{}' not found", self.current_connection))?
            .get_pooled_connection()
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
        let connect_params = self.parse_connect_params(params)?;
        let connection_info = self.build_connection_info(&connect_params)?;
        
        // Create new database connection (always MySQL)
        let database = create_database_with_retry(
            "mysql",
            &connection_info,
            self.max_retries,
        )?;
        
        // Store the connection and switch to it
        self.connections.insert(connect_params.connection_name.clone(), database);
        self.current_connection = connect_params.connection_name;
        
        info!("Connected and switched to connection: {}", self.current_connection);
        Ok(())
    }

    /// Switch to an existing connection
    pub fn switch_connection(&mut self, conn_name: &str) -> Result<()> {
        if !self.connections.contains_key(conn_name) {
            return Err(anyhow!("Connection '{}' does not exist", conn_name));
        }
        
        self.current_connection = conn_name.to_string();
        debug!("Switched to connection: {}", conn_name);
        Ok(())
    }

    /// Disconnect a specific connection
    pub fn disconnect(&mut self, conn_name: &str) -> Result<()> {
        if conn_name == DEFAULT_CONNECTION_NAME {
            return Err(anyhow!("Cannot disconnect the default connection"));
        }
        
        if !self.connections.contains_key(conn_name) {
            return Err(anyhow!("Connection '{}' does not exist", conn_name));
        }
        
        // If we're disconnecting the current connection, switch to default
        if self.current_connection == conn_name {
            self.current_connection = DEFAULT_CONNECTION_NAME.to_string();
            info!("Switched back to default connection after disconnecting '{}'", conn_name);
        }
        
        self.connections.remove(conn_name);
        info!("Disconnected connection: {}", conn_name);
        Ok(())
    }

    /// List all available connections
    pub fn list_connections(&self) -> Vec<String> {
        self.connections.keys().cloned().collect()
    }

    /// Get current connection information
    pub fn current_connection_info(&self) -> String {
        format!("Current connection: {} (available: {:?})", 
                self.current_connection, 
                self.list_connections())
    }

    /// Parse connection parameters from string
    /// Format: (conn_name, host, user, password, database, port)
    fn parse_connect_params(&self, params: &str) -> Result<ConnectParams> {
        let trimmed = params.trim();
        if !trimmed.starts_with('(') || !trimmed.ends_with(')') {
            return Err(anyhow!("Connect parameters must be enclosed in parentheses"));
        }
        
        let inner = &trimmed[1..trimmed.len()-1];
        let parts: Vec<&str> = inner.split(',').map(|s| s.trim()).collect();
        
        if parts.is_empty() {
            return Err(anyhow!("Connection name is required"));
        }
        
        // Fill missing parameters with empty strings
        let mut filled_parts = parts.clone();
        while filled_parts.len() < 6 {
            filled_parts.push("");
        }
        
        Ok(ConnectParams {
            connection_name: filled_parts[0].to_string(),
            host: filled_parts[1].to_string(),
            user: filled_parts[2].to_string(),
            password: filled_parts[3].to_string(),
            database: filled_parts[4].to_string(),
            port: filled_parts[5].to_string(),
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

        Ok(ConnectionInfo {
            host,
            port,
            user,
            password,
            database,
            params: self.default_connection_info.params.clone(),
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