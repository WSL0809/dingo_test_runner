//! Database abstraction layer
//!
//! This module provides a unified interface for different database types.
//! Currently supports MySQL, with extensible design for future database backends.

use anyhow::{anyhow, Result};
use log::{debug, error, info, trace, warn};
use std::time::Duration;
use crate::util::memory_pool::{get_row_data, get_string_vec, PooledRowData};

/// 网络超时配置常量
const TCP_CONNECT_TIMEOUT_SECS: u64 = 3;  // TCP连接建立超时
const CONNECTION_GET_TIMEOUT_SECS: u64 = 2;  // 从连接池获取连接的超时
const DEFAULT_READ_TIMEOUT_SECS: u64 = 30;   // 默认读超时（大数据操作）
const DEFAULT_WRITE_TIMEOUT_SECS: u64 = 30;  // 默认写超时（大数据操作）
const QUICK_QUERY_TIMEOUT_SECS: u64 = 10;    // 快速查询超时

/// 连接池配置常量
const POOL_MIN_CONNECTIONS: usize = 2;
const POOL_MAX_CONNECTIONS: usize = 10;

/// 操作类型枚举，用于动态超时配置
#[derive(Debug, Clone, Copy)]
pub enum OperationType {
    /// 快速查询（如 SELECT count(*), SHOW TABLES）
    QuickQuery,
    /// 数据库DDL操作（CREATE/DROP DATABASE/TABLE）
    DatabaseDDL,
    /// 大数据操作（INSERT/UPDATE 大量数据）
    BulkOperation,
    /// 默认操作
    Default,
}

/// 获取操作类型对应的连接获取超时
fn get_connection_timeout_for_operation(op_type: OperationType) -> Duration {
    match op_type {
        OperationType::QuickQuery => Duration::from_secs(1),
        OperationType::DatabaseDDL => Duration::from_secs(5),
        OperationType::BulkOperation => Duration::from_secs(10),
        OperationType::Default => Duration::from_secs(CONNECTION_GET_TIMEOUT_SECS),
    }
}

/// 根据SQL语句自动检测操作类型
fn detect_operation_type(sql: &str) -> OperationType {
    let sql_upper = sql.trim().to_uppercase();
    
    if sql_upper.starts_with("SELECT COUNT") 
        || sql_upper.starts_with("SHOW TABLES")
        || sql_upper.starts_with("SHOW DATABASES")
        || sql_upper.starts_with("DESCRIBE")
        || sql_upper.starts_with("EXPLAIN") {
        OperationType::QuickQuery
    } else if sql_upper.starts_with("CREATE DATABASE")
        || sql_upper.starts_with("DROP DATABASE")
        || sql_upper.starts_with("CREATE TABLE")
        || sql_upper.starts_with("DROP TABLE")
        || sql_upper.starts_with("ALTER TABLE") {
        OperationType::DatabaseDDL
    } else if sql_upper.starts_with("INSERT INTO")
        || sql_upper.starts_with("UPDATE")
        || sql_upper.starts_with("DELETE FROM")
        || sql_upper.contains("--SOURCE") {
        OperationType::BulkOperation
    } else {
        OperationType::Default
    }
}

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
            _ => Err(anyhow!(
                "Unsupported database type: {}. Currently only 'mysql' is supported.",
                db_type
            )),
        }
    }

    /// Execute a query and return the result as pooled strings (memory optimized)
    pub fn query(&mut self, sql: &str) -> Result<PooledRowData> {
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
            // TCP连接建立超时（快速检测网络问题）
            .tcp_connect_timeout(Some(Duration::from_secs(TCP_CONNECT_TIMEOUT_SECS)))
            // 设置网络读/写超时，适合大数据操作
            .read_timeout(Some(Duration::from_secs(DEFAULT_READ_TIMEOUT_SECS)))
            .write_timeout(Some(Duration::from_secs(DEFAULT_WRITE_TIMEOUT_SECS)));

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

        // 使用默认连接池配置（MySQL 26.0.0 不支持手动配置）
        let pool = mysql::Pool::new(opts)?;

        Ok(MySQLDatabase {
            pool,
            info: info.clone(),
            conn: None,
        })
    }

    pub fn get_pooled_connection(&self) -> Result<mysql::PooledConn> {
        self.get_pooled_connection_with_timeout(Duration::from_secs(CONNECTION_GET_TIMEOUT_SECS))
    }

    pub fn get_pooled_connection_with_timeout(&self, timeout: Duration) -> Result<mysql::PooledConn> {
        let mut attempt = 0;
        let max_attempts = 3;
        let mut delay = Duration::from_millis(100);
        
        loop {
            attempt += 1;
            
            match self.pool.try_get_conn(timeout) {
                Ok(conn) => return Ok(conn),
                Err(e) => {
                    let should_retry = match &e {
                        mysql::Error::DriverError(mysql::DriverError::Timeout) => {
                            warn!("Connection pool timeout (attempt {}/{}): no connections available within {:?}", 
                                  attempt, max_attempts, timeout);
                            attempt < max_attempts
                        },
                        mysql::Error::IoError(io_err) => {
                            if io_err.kind() == std::io::ErrorKind::WouldBlock 
                                || io_err.kind() == std::io::ErrorKind::TimedOut 
                                || io_err.to_string().contains("Resource temporarily unavailable") {
                                warn!("Resource temporarily unavailable (attempt {}/{}): {}", 
                                      attempt, max_attempts, io_err);
                                attempt < max_attempts
                            } else {
                                false
                            }
                        },
                        _ => false
                    };
                    
                    if !should_retry || attempt >= max_attempts {
                        return Err(match e {
                            mysql::Error::DriverError(mysql::DriverError::Timeout) => {
                                anyhow!("Connection pool exhausted after {} attempts: no connections available within {:?}", 
                                        max_attempts, timeout)
                            },
                            mysql::Error::IoError(io_err) if io_err.to_string().contains("Resource temporarily unavailable") => {
                                anyhow!("Database connection failed after {} attempts: {}. \
                                        This may indicate network issues or MySQL server overload. \
                                        Try reducing concurrent connections or increasing connection pool size.", 
                                        max_attempts, io_err)
                            },
                            _ => e.into()
                        });
                    }
                    
                    trace!("Retrying connection after {:?} delay...", delay);
                    std::thread::sleep(delay);
                    delay = std::cmp::min(delay * 2, Duration::from_secs(2));
                }
            }
        }
    }

    pub fn query(&mut self, sql: &str) -> Result<PooledRowData> {
        let op_type = detect_operation_type(sql);
        self.query_with_operation_type(sql, op_type)
    }

    pub fn query_with_operation_type(&mut self, sql: &str, op_type: OperationType) -> Result<PooledRowData> {
        use mysql::prelude::Queryable;

        trace!("-> exec ({:?}): {}", op_type, sql);
        let connection_timeout = get_connection_timeout_for_operation(op_type);
        
        
        if self.conn.is_none() {
            self.conn = Some(self.get_pooled_connection_with_timeout(connection_timeout)?);
        }


        let result: Result<Vec<mysql::Row>, mysql::Error> = {
            let conn_ref = self.conn.as_mut().unwrap();
            conn_ref.query(sql)
        };

        let rows: Vec<mysql::Row> = match result {
            Ok(rows) => Ok(rows),
            Err(e) => {
                if let mysql::Error::IoError(ref io_err) = e {
                    if io_err.kind() == std::io::ErrorKind::BrokenPipe
                        || io_err.kind() == std::io::ErrorKind::ConnectionAborted
                    {
                        warn!(
                            "MySQL connection broken, attempting to reconnect. Error: {}",
                            io_err
                        );
                        // 丢弃旧连接并重新获取
                        self.conn = None;
                        if let Ok(new_conn) = self.get_pooled_connection_with_timeout(connection_timeout) {
                            let mut new_conn = new_conn;
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

    /// Helper function to process rows into PooledRowData (memory pool optimized)
    fn process_rows(&self, rows: Vec<mysql::Row>) -> Result<PooledRowData> {
        use mysql::Value;

        // Get a pooled row data container from the memory pool
        let mut result_vec = get_row_data();
        
        // Pre-allocate capacity if we know the number of rows
        if !rows.is_empty() {
            result_vec.reserve(rows.len());
        }

        for row in rows {
            // Get a pooled string vector for each row
            let mut row_data = get_string_vec();
            
            // Pre-allocate capacity for the row
            if row.len() > 0 {
                row_data.reserve(row.len());
            }
            
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
            
            // Convert pooled string vec to regular vec before pushing to result
            result_vec.push(row_data.take());
        }
        Ok(result_vec)
    }

    pub fn execute(&mut self, sql: &str) -> Result<()> {
        let op_type = detect_operation_type(sql);
        self.execute_with_operation_type(sql, op_type)
    }

    pub fn execute_with_operation_type(&mut self, sql: &str, op_type: OperationType) -> Result<()> {
        use mysql::prelude::Queryable;

        trace!("-> execute ({:?}): {}", op_type, sql);
        let connection_timeout = get_connection_timeout_for_operation(op_type);
        
        if self.conn.is_none() {
            self.conn = Some(self.get_pooled_connection_with_timeout(connection_timeout)?);
        }

        let exec_result = {
            let conn_ref = self.conn.as_mut().unwrap();
            conn_ref.query_drop(sql)
        };

        if let Err(e) = exec_result {
            if let mysql::Error::IoError(ref io_err) = e {
                if io_err.kind() == std::io::ErrorKind::BrokenPipe
                    || io_err.kind() == std::io::ErrorKind::ConnectionAborted
                {
                    warn!("MySQL connection broken during execute, attempting to reconnect. Error: {}", io_err);
                    self.conn = None;
                    let mut new_conn = self.get_pooled_connection_with_timeout(connection_timeout)?;
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
        format!(
            "mysql://{}@{}:{}/{}",
            self.info.user, self.info.host, self.info.port, self.info.database
        )
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

    /// Extract unique database suffix from connection parameters
    /// This is used for parallel execution to ensure database isolation
    fn get_unique_db_suffix(&self) -> Option<String> {
        if self.info.params.is_empty() {
            return None;
        }
        
        for param in self.info.params.split('&') {
            if let Some((key, value)) = param.split_once('=') {
                if key == "test_db_suffix" {
                    return Some(value.to_string());
                }
            }
        }
        None
    }

    pub fn init_for_test(&mut self, test_name: &str) -> Result<()> {
        let start_time = std::time::Instant::now();
        
        // Check if we have a unique database suffix for parallel execution
        let test_db = if let Some(suffix) = self.get_unique_db_suffix() {
            // For parallel execution, use the unique suffix
            format!("test_{}_{}", Self::sanitize_db_name(test_name), suffix)
        } else {
            // For serial execution, use the original behavior
            format!("test_{}", Self::sanitize_db_name(test_name))
        };
        
        trace!("Starting init_for_test for database '{}'", test_db);
        
        // Step 1: Drop existing database
        trace!("Dropping existing database '{}' if exists", test_db);
        let drop_start = std::time::Instant::now();
        
        let drop_sql = format!("DROP DATABASE IF EXISTS `{}`", test_db);
        self.execute_with_operation_type(&drop_sql, OperationType::DatabaseDDL)?;
        trace!("Database drop completed in {:?} for '{}'", drop_start.elapsed(), test_db);
        
        // Step 2: Create new database
        trace!("Creating new database '{}'", test_db);
        let create_start = std::time::Instant::now();
        
        // 使用 DatabaseDDL 操作类型，获得更长的连接超时
        let create_sql = format!("CREATE DATABASE `{}`", test_db);
        match self.execute_with_operation_type(&create_sql, OperationType::DatabaseDDL) {
            Ok(_) => {
                trace!("Database creation completed in {:?} for '{}'", create_start.elapsed(), test_db);
            },
            Err(e) => {
                error!("Failed to create database '{}': {}", test_db, e);
                // 对于数据库创建失败，提供更详细的错误信息
                return Err(anyhow!(
                    "Failed to create test database '{}': {}. \
                    This may be caused by: \
                    1) MySQL server connection issues \
                    2) Insufficient privileges for database creation \
                    3) Server resource exhaustion. \
                    Please check MySQL server status and connection parameters.", 
                    test_db, e
                ));
            }
        }
        
        // Step 3: Switch to the new database
        trace!("Switching to database '{}'", test_db);
        let switch_start = std::time::Instant::now();
        self.switch_database(&test_db)?;
        trace!("Database switch completed in {:?} for '{}'", switch_start.elapsed(), test_db);
        
        let total_time = start_time.elapsed();
        info!(
            "MySQL test database '{}' created and connection switched in {:?}.",
            test_db, total_time
        );
        trace!("init_for_test completed in {:?} for '{}'", total_time, test_db);
        Ok(())
    }

    pub fn cleanup_after_test(&mut self, test_name: &str) -> Result<()> {
        let start_time = std::time::Instant::now();
        
        // Use the same naming logic as init_for_test
        let test_db = if let Some(suffix) = self.get_unique_db_suffix() {
            // For parallel execution, use the unique suffix
            format!("test_{}_{}", Self::sanitize_db_name(test_name), suffix)
        } else {
            // For serial execution, use the original behavior
            format!("test_{}", Self::sanitize_db_name(test_name))
        };
        
        trace!("Starting cleanup_after_test for database '{}'", test_db);

        // 1. 查询并删除所有表，避免遗留大型表阻塞 DROP DATABASE
        trace!("Querying tables in database '{}'", test_db);
        let query_start = std::time::Instant::now();
        let table_rows = self.query(&format!(
            "SELECT table_name AS tbl_name FROM information_schema.tables WHERE table_schema = '{}'",
            test_db
        ))?;
        trace!("Table query completed in {:?} for '{}', found {} tables", 
               query_start.elapsed(), test_db, table_rows.len());
        
        let drop_tables_start = std::time::Instant::now();
        for (i, row) in table_rows.iter().enumerate() {
            if let Some(tbl_name) = row.get(0) {
                trace!("Dropping table {}/{}: {}.{}", i + 1, table_rows.len(), test_db, tbl_name);
                let table_drop_start = std::time::Instant::now();
                // 使用完全限定名避免 current database 影响
                let drop_sql = format!("DROP TABLE IF EXISTS `{}`.`{}`", test_db, tbl_name);
                // 忽略单表删除错误，继续后续清理
                if let Err(e) = self.execute(&drop_sql) {
                    warn!("failed to drop table {}.{}: {}", test_db, tbl_name, e);
                } else {
                    trace!("Table {}.{} dropped in {:?}", test_db, tbl_name, table_drop_start.elapsed());
                }
            }
        }
        trace!("All tables dropped in {:?} for '{}'", drop_tables_start.elapsed(), test_db);

        // 2. 通过切换数据库连接来释放对当前测试库的占用
        trace!("Switching to 'mysql' database during cleanup for '{}'", test_db);
        let switch_start = std::time::Instant::now();
        if let Err(e) = self.switch_database("mysql") {
            warn!(
                "Failed to switch to 'mysql' db during cleanup, proceeding with DROP: {}",
                e
            );
        } else {
            trace!("Database switch to 'mysql' completed in {:?} during cleanup", switch_start.elapsed());
        }

        // 3. 最终删除数据库
        trace!("Dropping database '{}'", test_db);
        let db_drop_start = std::time::Instant::now();
        if let Err(e) = self.execute(&format!("DROP DATABASE IF EXISTS `{}`", test_db)) {
            warn!(
                "Failed to drop database '{}': {}. This may happen if the connection was lost.",
                test_db, e
            );
        } else {
            trace!("Database '{}' dropped in {:?}", test_db, db_drop_start.elapsed());
            debug!("MySQL test database '{}' dropped", test_db);
        }

        // 注意：不需要切换回原始数据库，因为测试数据库已被删除
        // 连接池会在需要时自动重新连接到合适的数据库

        let total_time = start_time.elapsed();
        trace!("cleanup_after_test completed in {:?} for '{}'", total_time, test_db);
        Ok(())
    }

    /// 快速数据库切换（使用 USE 命令）
    /// 这是最快的数据库切换方式，直接在现有连接上执行 USE 命令
    fn quick_switch_database(&mut self, new_db_name: &str) -> Result<()> {
        let start_time = std::time::Instant::now();
        trace!("Starting quick database switch to '{}'", new_db_name);
        
        // 验证数据库名称（防止 SQL 注入）
        if new_db_name.contains('`') || new_db_name.contains(';') || new_db_name.contains('\0') {
            return Err(anyhow!("Invalid database name: contains illegal characters"));
        }
        
        // 使用现有连接执行 USE 命令
        let use_sql = format!("USE `{}`", new_db_name);
        trace!("Executing quick switch: {}", use_sql);
        
        // 使用 QuickQuery 操作类型，获得更快的连接超时
        self.execute_with_operation_type(&use_sql, OperationType::QuickQuery)?;
        
        // 更新内部状态
        self.info.database = new_db_name.to_string();
        
        let total_time = start_time.elapsed();
        debug!("Quick database switch to '{}' completed in {:?}", new_db_name, total_time);
        trace!("quick_switch_database completed in {:?} for '{}'", total_time, new_db_name);
        
        Ok(())
    }
    
    /// 传统数据库切换（重建连接池）
    /// 这是降级方案，当快速切换失败时使用
    fn recreate_pool_switch_database(&mut self, new_db_name: &str) -> Result<()> {
        let start_time = std::time::Instant::now();
        warn!("Using fallback pool recreation method for database switch to '{}'", new_db_name);
        
        let mut new_info = self.info.clone();
        new_info.database = new_db_name.to_string();

        // 创建一个新的连接池
        trace!("Building MySQL connection options for '{}'", new_db_name);
        let opts_start = std::time::Instant::now();
        let mut opts = mysql::OptsBuilder::new()
            .ip_or_hostname(Some(&new_info.host))
            .tcp_port(new_info.port)
            .user(Some(&new_info.user))
            .pass(Some(&new_info.password))
            .tcp_connect_timeout(Some(Duration::from_secs(TCP_CONNECT_TIMEOUT_SECS)))
            .read_timeout(Some(Duration::from_secs(DEFAULT_READ_TIMEOUT_SECS)))
            .write_timeout(Some(Duration::from_secs(DEFAULT_WRITE_TIMEOUT_SECS)));

        if !new_info.database.is_empty() {
            opts = opts.db_name(Some(&new_info.database));
        }
        trace!("MySQL options built in {:?} for '{}'", opts_start.elapsed(), new_db_name);

        trace!("Creating new MySQL connection pool for '{}'", new_db_name);
        let pool_start = std::time::Instant::now();
        
        // 使用默认连接池配置
        let new_pool = mysql::Pool::new(opts)?;
        trace!("MySQL pool created in {:?} for '{}'", pool_start.elapsed(), new_db_name);

        // 替换旧的连接池和信息
        trace!("Replacing old connection pool and info for '{}'", new_db_name);
        self.pool = new_pool;
        self.info = new_info;
        self.conn = None;

        let total_time = start_time.elapsed();
        warn!("Fallback pool recreation switch to '{}' completed in {:?}", new_db_name, total_time);
        
        Ok(())
    }
    
    /// 切换当前数据库连接到一个新的数据库
    /// 优先尝试快速切换，失败时降级到连接池重建
    pub fn switch_database(&mut self, new_db_name: &str) -> Result<()> {
        let overall_start = std::time::Instant::now();
        info!("Switching database to '{}' using optimized strategy", new_db_name);
        
        // 策略 1: 尝试快速切换（USE 命令）
        match self.quick_switch_database(new_db_name) {
            Ok(_) => {
                let total_time = overall_start.elapsed();
                info!("Database switch to '{}' completed successfully using quick method in {:?}", 
                      new_db_name, total_time);
                Ok(())
            },
            Err(quick_error) => {
                warn!("Quick database switch failed: {}. Attempting fallback method...", quick_error);
                
                // 策略 2: 降级到连接池重建
                match self.recreate_pool_switch_database(new_db_name) {
                    Ok(_) => {
                        let total_time = overall_start.elapsed();
                        warn!("Database switch to '{}' completed using fallback method in {:?}", 
                              new_db_name, total_time);
                        Ok(())
                    },
                    Err(fallback_error) => {
                        let total_time = overall_start.elapsed();
                        error!("All database switch methods failed for '{}' after {:?}", 
                               new_db_name, total_time);
                        Err(anyhow!(
                            "Failed to switch to database '{}': Quick method failed ({}), \
                             Fallback method failed ({}). Total time: {:?}",
                            new_db_name, quick_error, fallback_error, total_time
                        ))
                    }
                }
            }
        }
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
                info!(
                    "Successfully connected to {} database on attempt {}",
                    db_type, attempt
                );
                return Ok(db);
            }
            Err(e) => {
                warn!("Database connection failed on attempt {}: {}", attempt, e);
                if attempt >= max_retries {
                    return Err(anyhow!(
                        "Failed to connect after {} attempts: {}",
                        max_retries,
                        e
                    ));
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
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Unsupported database type"));
    }

    #[test]
    fn test_mysql_database_info_format() {
        let info = create_test_connection_info();
        // Test the info string formatting without creating actual MySQL connection
        let expected_info = "mysql://root@127.0.0.1:3306/test";

        // Test the info formatting logic directly
        let formatted_info = format!(
            "mysql://{}@{}:{}/{}",
            info.user, info.host, info.port, info.database
        );

        assert_eq!(formatted_info, expected_info);
    }
}
