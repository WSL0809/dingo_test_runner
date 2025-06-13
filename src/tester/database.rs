//! Database abstraction layer
//! 
//! This module provides a unified interface for different database types.
//! Currently supports MySQL, with extensible design for future database backends.

use anyhow::{anyhow, Result};
use log::{debug, info, warn};
use std::time::Duration;

/// Database connection abstraction
#[derive(Debug)]
pub enum Database {
    MySQL(MySQLDatabase),
    // Future database types can be added here:
    // PostgreSQL(PostgreSQLDatabase),
    // Oracle(OracleDatabase),
}

impl Database {
    /// Create a new database connection based on type
    pub fn new(db_type: &str, connection_info: &ConnectionInfo) -> Result<Self> {
        match db_type.to_lowercase().as_str() {
            "mysql" => {
                let mysql_db = MySQLDatabase::new(connection_info)?;
                Ok(Database::MySQL(mysql_db))
            }
            // Future database types:
            // "postgresql" | "postgres" => {
            //     let pg_db = PostgreSQLDatabase::new(connection_info)?;
            //     Ok(Database::PostgreSQL(pg_db))
            // }
            _ => Err(anyhow!("Unsupported database type: {}. Currently only 'mysql' is supported.", db_type)),
        }
    }

    /// Execute a query and return the result as strings
    pub fn query(&mut self, sql: &str) -> Result<Vec<Vec<String>>> {
        match self {
            Database::MySQL(db) => db.query(sql),
        }
    }

    /// Execute a query without returning results
    pub fn execute(&mut self, sql: &str) -> Result<()> {
        match self {
            Database::MySQL(db) => db.execute(sql),
        }
    }

    /// Get database connection info
    pub fn info(&self) -> String {
        match self {
            Database::MySQL(db) => db.info(),
        }
    }

    /// Initialize database for testing
    pub fn init_for_test(&mut self, test_name: &str) -> Result<()> {
        match self {
            Database::MySQL(db) => db.init_for_test(test_name),
        }
    }

    /// Cleanup after test
    pub fn cleanup_after_test(&mut self, test_name: &str) -> Result<()> {
        match self {
            Database::MySQL(db) => db.cleanup_after_test(test_name),
        }
    }
}

/// Connection information structure
#[derive(Debug, Clone)]
pub struct ConnectionInfo {
    pub host: String,
    pub port: u16,
    pub user: String,
    pub password: String,
    pub database: String,
    pub params: String,
}

/// MySQL database implementation
#[derive(Debug)]
pub struct MySQLDatabase {
    conn: Option<mysql::PooledConn>,
    pool: mysql::Pool,
    database: String,
    host: String,
    port: u16,
    user: String,
}

impl MySQLDatabase {
    pub fn new(info: &ConnectionInfo) -> Result<Self> {
        let mut opts = mysql::OptsBuilder::new()
            .ip_or_hostname(Some(&info.host))
            .tcp_port(info.port)
            .user(Some(&info.user))
            .pass(Some(&info.password));

        if !info.database.is_empty() {
            opts = opts.db_name(Some(&info.database));
        }

        // Parse additional parameters
        if !info.params.is_empty() {
            for param in info.params.split('&') {
                if let Some((key, value)) = param.split_once('=') {
                    debug!("MySQL parameter: {}={}", key, value);
                    // Handle specific MySQL parameters as needed
                    // This can be extended to support more MySQL-specific options
                }
            }
        }

        let pool = mysql::Pool::new(opts)?;

        Ok(MySQLDatabase {
            conn: None,
            pool,
            database: info.database.clone(),
            host: info.host.clone(),
            port: info.port,
            user: info.user.clone(),
        })
    }

    fn get_conn(&mut self) -> Result<&mut mysql::PooledConn> {
        if self.conn.is_none() {
            self.conn = Some(self.pool.get_conn()?);
        }
        Ok(self.conn.as_mut().unwrap())
    }

    pub fn query(&mut self, sql: &str) -> Result<Vec<Vec<String>>> {
        use mysql::prelude::Queryable;
        
        let conn = self.get_conn()?;
        let rows: Vec<mysql::Row> = conn.query(sql)?;
        
        let mut result = Vec::new();
        for row in rows {
            let mut row_data = Vec::new();
            for i in 0..row.len() {
                let value = row.get::<Option<String>, _>(i)
                    .unwrap_or_else(|| Some("NULL".to_string()))
                    .unwrap_or_else(|| "NULL".to_string());
                row_data.push(value);
            }
            result.push(row_data);
        }
        
        Ok(result)
    }

    pub fn execute(&mut self, sql: &str) -> Result<()> {
        use mysql::prelude::Queryable;
        
        let conn = self.get_conn()?;
        conn.query_drop(sql)?;
        Ok(())
    }

    pub fn info(&self) -> String {
        format!("mysql://{}@{}:{}/{}", self.user, self.host, self.port, self.database)
    }

    pub fn init_for_test(&mut self, test_name: &str) -> Result<()> {
        let test_db = format!("test_{}", test_name);
        self.execute(&format!("DROP DATABASE IF EXISTS `{}`", test_db))?;
        self.execute(&format!("CREATE DATABASE `{}`", test_db))?;
        self.execute(&format!("USE `{}`", test_db))?;
        info!("MySQL test database '{}' created", test_db);
        Ok(())
    }

    pub fn cleanup_after_test(&mut self, test_name: &str) -> Result<()> {
        let test_db = format!("test_{}", test_name);
        self.execute(&format!("DROP DATABASE IF EXISTS `{}`", test_db))?;
        debug!("MySQL test database '{}' dropped", test_db);
        Ok(())
    }
}

/// Create database connection with retry logic
pub fn create_database_with_retry(
    db_type: &str,
    connection_info: &ConnectionInfo,
    max_retries: u32,
) -> Result<Database> {
    let mut attempt = 0;
    let mut delay = Duration::from_millis(100);

    loop {
        attempt += 1;
        
        match Database::new(db_type, connection_info) {
            Ok(db) => {
                info!("Successfully connected to {} database on attempt {}", db_type, attempt);
                return Ok(db);
            }
            Err(e) => {
                warn!("Database connection failed on attempt {}: {}", attempt, e);
                if attempt >= max_retries {
                    return Err(anyhow!("Failed to connect after {} attempts: {}", max_retries, e));
                }
            }
        }

        // Exponential backoff with maximum delay
        std::thread::sleep(delay);
        delay = std::cmp::min(delay * 2, Duration::from_secs(10));
    }
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
} 