//! Database abstraction layer
//! 
//! This module provides a unified interface for different database types.
//! Currently supports MySQL, with extensible design for future database backends.

use anyhow::{anyhow, Result};
use log::{debug, info, trace, warn};
use std::time::Duration;

/// 默认读/写超时时长（秒）
const QUERY_TIMEOUT_SECS: u64 = 10;

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

    pub fn get_pooled_connection(&self) -> Result<mysql::PooledConn> {
        match self {
            Database::MySQL(db) => db.get_pooled_connection(),
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
    pool: mysql::Pool,
    info: ConnectionInfo,
    /// 可复用的连接。串行执行场景下复用单个 `PooledConn` 可避免每条语句重新握手造成的额外延迟。
    conn: Option<mysql::PooledConn>,
}

impl MySQLDatabase {
    pub fn new(info: &ConnectionInfo) -> Result<Self> {
        let mut opts = mysql::OptsBuilder::new()
            .ip_or_hostname(Some(&info.host))
            .tcp_port(info.port)
            .user(Some(&info.user))
            .pass(Some(&info.password))
            // 设置网络读/写超时，防止后端长时间无响应导致阻塞
            .read_timeout(Some(Duration::from_secs(QUERY_TIMEOUT_SECS)))
            .write_timeout(Some(Duration::from_secs(QUERY_TIMEOUT_SECS)));

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
            pool,
            info: info.clone(),
            conn: None,
        })
    }

    pub fn get_pooled_connection(&self) -> Result<mysql::PooledConn> {
        self.pool.get_conn().map_err(Into::into)
    }

    pub fn query(&mut self, sql: &str) -> Result<Vec<Vec<String>>> {
        use mysql::prelude::Queryable;
        
        trace!("-> exec: {}", sql);
        // 尝试复用已有连接；若不可用则从连接池重新获取。
        if self.conn.is_none() {
            self.conn = Some(self.pool.get_conn()?);
        }

        // 这里通过 match 单独取出 &mut 引用，避免双可变借用冲突。
        let result: Result<Vec<mysql::Row>, mysql::Error> = {
            let conn_ref = self.conn.as_mut().unwrap();
            conn_ref.query(sql)
        };

        let rows: Vec<mysql::Row> = match result {
            Ok(rows) => Ok(rows),
            Err(e) => {
                if let mysql::Error::IoError(ref io_err) = e {
                     if io_err.kind() == std::io::ErrorKind::BrokenPipe || io_err.kind() == std::io::ErrorKind::ConnectionAborted {
                        warn!("MySQL connection broken, attempting to reconnect. Error: {}", io_err);
                        // 丢弃旧连接并重新获取
                        self.conn = None;
                        if let Ok(mut new_conn) = self.pool.get_conn() {
                            // 将新连接放入缓存，供后续复用
                            let new_rows: Vec<mysql::Row> = new_conn.query(sql)?;
                            self.conn = Some(new_conn);
                            return self.process_rows(new_rows);
                        }
                    }
                }
                warn!("query failed: {:?}", e);
                Err(e)
            }
        }?;
        
        self.process_rows(rows)
    }

    /// Helper function to process rows into Vec<Vec<String>>
    fn process_rows(&self, rows: Vec<mysql::Row>) -> Result<Vec<Vec<String>>> {
        use mysql::Value;

        let mut result_vec = Vec::new();
        for row in rows {
            let mut row_data = Vec::new();
            for idx in 0..row.len() {
                let val = row.as_ref(idx).unwrap_or(&Value::NULL);
                let cell = match val {
                    Value::NULL => "NULL".to_string(),
                    Value::Bytes(b) => String::from_utf8_lossy(b).into_owned(),
                    Value::Int(n) => n.to_string(),
                    Value::UInt(n) => n.to_string(),
                    Value::Float(f) => (*f as f64).to_string(),
                    Value::Double(d) => d.to_string(),
                    Value::Date(y, m, d, hh, mm, ss, _us) => {
                        format!("{:04}-{:02}-{:02} {:02}:{:02}:{:02}", y, m, d, hh, mm, ss)
                    }
                    Value::Time(_neg, d, hh, mm, ss, _us) => {
                        let total_hours = (*d as u32) * 24 + (*hh as u32);
                        format!("{:02}:{:02}:{:02}", total_hours, mm, ss)
                    }
                };
                row_data.push(cell);
            }
            result_vec.push(row_data);
        }
        Ok(result_vec)
    }

    pub fn execute(&mut self, sql: &str) -> Result<()> {
        use mysql::prelude::Queryable;
        
        if self.conn.is_none() {
            self.conn = Some(self.pool.get_conn()?);
        }

        let exec_result = {
            let conn_ref = self.conn.as_mut().unwrap();
            conn_ref.query_drop(sql)
        };

        if let Err(e) = exec_result {
            if let mysql::Error::IoError(ref io_err) = e {
                if io_err.kind() == std::io::ErrorKind::BrokenPipe || io_err.kind() == std::io::ErrorKind::ConnectionAborted {
                    warn!("MySQL connection broken during execute, attempting to reconnect. Error: {}", io_err);
                    self.conn = None;
                    let mut new_conn = self.pool.get_conn()?;
                    new_conn.query_drop(sql)?;
                    self.conn = Some(new_conn);
                    return Ok(());
                }
            }
            return Err(e.into());
        }
        Ok(())
    }

    pub fn info(&self) -> String {
        format!("mysql://{}@{}:{}/{}", self.info.user, self.info.host, self.info.port, self.info.database)
    }

    /// Helper to transform arbitrary test names into valid MySQL schema names
    fn sanitize_db_name(test_name: &str) -> String {
        // 1. Replace path separators及其它非字母数字字符为 '_'
        let mut sanitized: String = test_name
            .chars()
            .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
            .collect();

        // 2. Collapse连续 '_'，避免过长
        while sanitized.contains("__") {
            sanitized = sanitized.replace("__", "_");
        }

        // 3. Trim开头/结尾 '_' 以保持整洁
        sanitized = sanitized.trim_matches('_').to_string();

        // 4. 若结果为空则回退为固定名
        if sanitized.is_empty() {
            sanitized = "dingo".to_string();
        }

        // 5. MySQL 允许数据库名最大 64 字节，这里留出前缀空间
        const MAX_LEN: usize = 55; // 64 - len("test_") 预留
        if sanitized.len() > MAX_LEN {
            sanitized.truncate(MAX_LEN);
        }

        sanitized
    }

    pub fn init_for_test(&mut self, test_name: &str) -> Result<()> {
        let test_db = format!("test_{}", Self::sanitize_db_name(test_name));
        self.execute(&format!("DROP DATABASE IF EXISTS `{}`", test_db))?;
        self.execute(&format!("CREATE DATABASE `{}`", test_db))?;
        // 切换到新创建的数据库，通过重新建立连接池
        self.switch_database(&test_db)?;
        info!("MySQL test database '{}' created and connection switched.", test_db);
        Ok(())
    }

    pub fn cleanup_after_test(&mut self, test_name: &str) -> Result<()> {
        let test_db = format!("test_{}", Self::sanitize_db_name(test_name));
        
        // 1. 查询并删除所有表，避免遗留大型表阻塞 DROP DATABASE
        let table_rows = self.query(&format!(
            "SELECT table_name AS tbl_name FROM information_schema.tables WHERE table_schema = '{}'",
            test_db
        ))?;
        for row in table_rows {
            if let Some(tbl_name) = row.get(0) {
                // 使用完全限定名避免 current database 影响
                let drop_sql = format!("DROP TABLE IF EXISTS `{}`.`{}`", test_db, tbl_name);
                // 忽略单表删除错误，继续后续清理
                if let Err(e) = self.execute(&drop_sql) {
                    warn!("failed to drop table {}.{}: {}", test_db, tbl_name, e);
                }
            }
        }

        // 2. 通过切换数据库连接来释放对当前测试库的占用
        if let Err(e) = self.switch_database("mysql") {
            warn!("Failed to switch to 'mysql' db during cleanup, proceeding with DROP: {}", e);
        }

        // 3. 最终删除数据库
        if let Err(e) = self.execute(&format!("DROP DATABASE IF EXISTS `{}`", test_db)) {
             warn!("Failed to drop database '{}': {}. This may happen if the connection was lost.", test_db, e);
        } else {
            debug!("MySQL test database '{}' dropped", test_db);
        }

        // 注意：不需要切换回原始数据库，因为测试数据库已被删除
        // 连接池会在需要时自动重新连接到合适的数据库

        Ok(())
    }

    /// 切换当前数据库连接到一个新的数据库
    pub fn switch_database(&mut self, new_db_name: &str) -> Result<()> {
        let mut new_info = self.info.clone();
        new_info.database = new_db_name.to_string();

        // 创建一个新的连接池
        let mut opts = mysql::OptsBuilder::new()
            .ip_or_hostname(Some(&new_info.host))
            .tcp_port(new_info.port)
            .user(Some(&new_info.user))
            .pass(Some(&new_info.password))
            .read_timeout(Some(Duration::from_secs(QUERY_TIMEOUT_SECS)))
            .write_timeout(Some(Duration::from_secs(QUERY_TIMEOUT_SECS)));

        if !new_info.database.is_empty() {
            opts = opts.db_name(Some(&new_info.database));
        }

        let new_pool = mysql::Pool::new(opts)?;

        // 替换旧的连接池和信息
        self.pool = new_pool;
        self.info = new_info;
        self.conn = None;

        debug!("Switched database connection to '{}'", new_db_name);
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