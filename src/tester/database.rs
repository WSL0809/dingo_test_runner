//! Database abstraction layer
//! 
//! This module provides a unified interface for different database types,
//! allowing the test runner to work with both MySQL and SQLite.

use anyhow::{anyhow, Result};
use log::{debug, info, warn};
use std::time::Duration;

/// Database connection abstraction
pub enum Database {
    MySQL(MySQLDatabase),
    SQLite(SQLiteDatabase),
}

impl Database {
    /// Create a new database connection based on type
    pub fn new(db_type: &str, connection_info: &ConnectionInfo) -> Result<Self> {
        match db_type.to_lowercase().as_str() {
            "mysql" => {
                let mysql_db = MySQLDatabase::new(connection_info)?;
                Ok(Database::MySQL(mysql_db))
            }
            "sqlite" => {
                let sqlite_db = SQLiteDatabase::new(&connection_info.sqlite_file)?;
                Ok(Database::SQLite(sqlite_db))
            }
            _ => Err(anyhow!("Unsupported database type: {}", db_type)),
        }
    }

    /// Execute a query and return the result as strings
    pub fn query(&mut self, sql: &str) -> Result<Vec<Vec<String>>> {
        match self {
            Database::MySQL(db) => db.query(sql),
            Database::SQLite(db) => db.query(sql),
        }
    }

    /// Execute a query without returning results
    pub fn execute(&mut self, sql: &str) -> Result<()> {
        match self {
            Database::MySQL(db) => db.execute(sql),
            Database::SQLite(db) => db.execute(sql),
        }
    }

    /// Get database connection info
    pub fn info(&self) -> String {
        match self {
            Database::MySQL(db) => db.info(),
            Database::SQLite(db) => db.info(),
        }
    }

    /// Initialize database for testing
    pub fn init_for_test(&mut self, test_name: &str) -> Result<()> {
        match self {
            Database::MySQL(db) => db.init_for_test(test_name),
            Database::SQLite(db) => db.init_for_test(test_name),
        }
    }

    /// Cleanup after test
    pub fn cleanup_after_test(&mut self, test_name: &str) -> Result<()> {
        match self {
            Database::MySQL(db) => db.cleanup_after_test(test_name),
            Database::SQLite(db) => db.cleanup_after_test(test_name),
        }
    }
}

/// Connection information structure
pub struct ConnectionInfo {
    pub host: String,
    pub port: u16,
    pub user: String,
    pub password: String,
    pub database: String,
    pub params: String,
    pub sqlite_file: String,
}

/// MySQL database implementation
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

/// SQLite database implementation
pub struct SQLiteDatabase {
    conn: rusqlite::Connection,
    file_path: String,
}

impl SQLiteDatabase {
    pub fn new(file_path: &str) -> Result<Self> {
        let conn = if file_path == ":memory:" {
            rusqlite::Connection::open_in_memory()?
        } else {
            rusqlite::Connection::open(file_path)?
        };

        info!("SQLite database opened: {}", file_path);

        Ok(SQLiteDatabase {
            conn,
            file_path: file_path.to_string(),
        })
    }

    pub fn query(&mut self, sql: &str) -> Result<Vec<Vec<String>>> {
        // Convert MySQL syntax to SQLite syntax where needed
        let sqlite_sql = self.convert_mysql_to_sqlite(sql);
        
        let mut stmt = self.conn.prepare(&sqlite_sql)?;
        let column_count = stmt.column_count();
        
        let rows = stmt.query_map([], |row| {
            let mut row_data = Vec::new();
            for i in 0..column_count {
                // Try different types to handle SQLite's dynamic typing
                let value = if let Ok(s) = row.get::<_, String>(i) {
                    s
                } else if let Ok(n) = row.get::<_, i64>(i) {
                    n.to_string()
                } else if let Ok(f) = row.get::<_, f64>(i) {
                    f.to_string()
                } else {
                    "NULL".to_string()
                };
                row_data.push(value);
            }
            Ok(row_data)
        })?;

        let mut result = Vec::new();
        for row in rows {
            result.push(row?);
        }

        Ok(result)
    }

    pub fn execute(&mut self, sql: &str) -> Result<()> {
        // Convert MySQL syntax to SQLite syntax where needed
        let sqlite_sql = self.convert_mysql_to_sqlite(sql);
        
        self.conn.execute(&sqlite_sql, [])?;
        Ok(())
    }

    pub fn info(&self) -> String {
        format!("sqlite://{}", self.file_path)
    }

    pub fn init_for_test(&mut self, test_name: &str) -> Result<()> {
        // For SQLite, we'll create a table prefix for the test
        debug!("SQLite test '{}' initialized", test_name);
        Ok(())
    }

    pub fn cleanup_after_test(&mut self, test_name: &str) -> Result<()> {
        // For SQLite, clean up any test-specific tables
        let tables_query = "SELECT name FROM sqlite_master WHERE type='table' AND name LIKE 'test_%'";
        let tables = self.query(tables_query)?;
        
        for table_row in tables {
            if let Some(table_name) = table_row.get(0) {
                if table_name.starts_with(&format!("test_{}", test_name)) {
                    self.execute(&format!("DROP TABLE IF EXISTS {}", table_name))?;
                }
            }
        }
        
        debug!("SQLite test '{}' cleaned up", test_name);
        Ok(())
    }

    /// Convert basic MySQL syntax to SQLite syntax
    fn convert_mysql_to_sqlite(&self, sql: &str) -> String {
        let mut result = sql.to_string();
        
        // Convert SHOW DATABASES to SQLite equivalent
        if result.trim().to_uppercase() == "SHOW DATABASES" {
            return "SELECT 'main' as Database".to_string();
        }
        
        // Convert SHOW TABLES
        if result.trim().to_uppercase() == "SHOW TABLES" {
            return "SELECT name FROM sqlite_master WHERE type='table'".to_string();
        }
        
        // Convert USE database (SQLite doesn't support this, so we'll ignore it)
        if result.trim().to_uppercase().starts_with("USE ") {
            return "SELECT 1".to_string(); // No-op
        }
        
        // Convert CREATE DATABASE (SQLite doesn't support this, so we'll ignore it)
        if result.trim().to_uppercase().starts_with("CREATE DATABASE") {
            return "SELECT 1".to_string(); // No-op
        }
        
        // Convert DROP DATABASE (SQLite doesn't support this, so we'll ignore it)
        if result.trim().to_uppercase().starts_with("DROP DATABASE") {
            return "SELECT 1".to_string(); // No-op
        }
        
        // Remove backticks (SQLite uses double quotes for identifiers)
        result = result.replace('`', "\"");
        
        // Convert AUTO_INCREMENT to AUTOINCREMENT
        result = result.replace("AUTO_INCREMENT", "AUTOINCREMENT");
        
        result
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

        // Only retry for MySQL connections (SQLite failures are usually permanent)
        if db_type.to_lowercase() == "sqlite" {
            return Err(anyhow!("SQLite connection failed permanently"));
        }

        // Exponential backoff for MySQL
        std::thread::sleep(delay);
        delay = std::cmp::min(delay * 2, Duration::from_secs(10));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sqlite_basic_operations() {
        let mut db = SQLiteDatabase::new(":memory:").unwrap();
        
        // Test basic operations
        db.execute("CREATE TABLE test (id INTEGER, name TEXT)").unwrap();
        db.execute("INSERT INTO test VALUES (1, 'hello')").unwrap();
        
        let results = db.query("SELECT * FROM test").unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0], vec!["1", "hello"]);
    }

    #[test]
    fn test_mysql_to_sqlite_conversion() {
        let db = SQLiteDatabase::new(":memory:").unwrap();
        
        assert_eq!(db.convert_mysql_to_sqlite("SHOW DATABASES"), "SELECT 'main' as Database");
        assert_eq!(db.convert_mysql_to_sqlite("USE test"), "SELECT 1");
        assert_eq!(db.convert_mysql_to_sqlite("CREATE TABLE `test` (`id` INT AUTO_INCREMENT)"), 
                   "CREATE TABLE \"test\" (\"id\" INT AUTOINCREMENT)");
    }
} 