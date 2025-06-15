//! Database abstraction layer
//! 
//! This module provides a unified interface for different database types.
//! Currently supports MySQL, with extensible design for future database backends.

use anyhow::{anyhow, Result};
use log::{debug, info, warn};
use std::time::Duration;

/// 默认读/写超时时长（秒）
const QUERY_TIMEOUT_SECS: u64 = 30;

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
        })
    }

    pub fn get_pooled_connection(&self) -> Result<mysql::PooledConn> {
        self.pool.get_conn().map_err(Into::into)
    }

    pub fn query(&mut self, sql: &str) -> Result<Vec<Vec<String>>> {
        use mysql::prelude::Queryable;
        
        let mut conn = self.pool.get_conn()?;
        let result: Result<Vec<mysql::Row>, _> = conn.query(sql);

        let rows: Vec<mysql::Row> = match result {
            Ok(rows) => Ok(rows),
            Err(e) => {
                if let mysql::Error::IoError(ref io_err) = e {
                     if io_err.kind() == std::io::ErrorKind::BrokenPipe || io_err.kind() == std::io::ErrorKind::ConnectionAborted {
                        warn!("MySQL connection broken, attempting to reconnect. Error: {}", io_err);
                        if let Ok(mut new_conn) = self.pool.get_conn() {
                            // Re-execute and handle result processing here to avoid complex return types
                            let new_rows: Vec<mysql::Row> = new_conn.query(sql)?;
                            return self.process_rows(new_rows);
                        }
                    }
                }
                Err(e)
            }
        }?;
        
        self.process_rows(rows)
    }

    /// Helper function to process rows into Vec<Vec<String>>
    fn process_rows(&self, rows: Vec<mysql::Row>) -> Result<Vec<Vec<String>>> {
        let mut result_vec = Vec::new();
        for row in rows {
            let mut row_data = Vec::new();
            for i in 0..row.len() {
                let value = row.get::<Option<String>, _>(i)
                    .unwrap_or_else(|| Some("NULL".to_string()))
                    .unwrap_or_else(|| "NULL".to_string());
                row_data.push(value);
            }
            result_vec.push(row_data);
        }
        Ok(result_vec)
    }

    pub fn execute(&mut self, sql: &str) -> Result<()> {
        use mysql::prelude::Queryable;
        
        self.pool.get_conn()?.query_drop(sql)?;
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
        let original_db = self.info.database.clone();
        if let Err(e) = self.switch_database("mysql") {
            warn!("Failed to switch to 'mysql' db during cleanup, proceeding with DROP: {}", e);
        }

        // 3. 最终删除数据库
        if let Err(e) = self.execute(&format!("DROP DATABASE IF EXISTS `{}`", test_db)) {
             warn!("Failed to drop database '{}': {}. This may happen if the connection was lost.", test_db, e);
        } else {
            debug!("MySQL test database '{}' dropped", test_db);
        }

        // (可选) 恢复到原始数据库连接
        if let Err(e) = self.switch_database(&original_db) {
            warn!("Failed to switch back to original db '{}': {}", original_db, e);
        }

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