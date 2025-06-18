//! Database connection management
//!
//! This module handles MySQL database connections, including connection pooling,
//! retry logic, and connection initialization.

use anyhow::{anyhow, Result};
use log::{debug, info, warn};
use mysql::prelude::Queryable;
use mysql::{OptsBuilder, Pool, PooledConn};
use std::time::Duration;

/// Database connection wrapper
pub struct Conn {
    /// Pooled MySQL connection
    conn: Option<PooledConn>,
    /// Connection pool for creating new connections
    pool: Pool,
    /// Database name for this connection
    database: String,
    /// Host information
    host: String,
    /// Port information  
    port: u16,
    /// Username
    user: String,
}

impl Conn {
    /// Create a new database connection
    pub fn new(
        host: &str,
        port: u16,
        user: &str,
        password: &str,
        database: &str,
        params: &str,
    ) -> Result<Self> {
        let mut opts = OptsBuilder::new()
            .ip_or_hostname(Some(host))
            .tcp_port(port)
            .user(Some(user))
            .pass(Some(password))
            .db_name(Some(database));

        // Parse additional parameters
        if !params.is_empty() {
            // Parse params like "key1=value1&key2=value2"
            for param in params.split('&') {
                if let Some((key, value)) = param.split_once('=') {
                    match key {
                        "charset" => opts = opts.prefer_socket(false),
                        "ssl-mode" => {
                            // Handle SSL mode settings
                            debug!("SSL mode parameter: {}", value);
                        }
                        _ => {
                            debug!("Unknown parameter: {}={}", key, value);
                        }
                    }
                }
            }
        }

        // Create connection pool
        let pool = Pool::new(opts)?;

        Ok(Conn {
            conn: None,
            pool,
            database: database.to_string(),
            host: host.to_string(),
            port,
            user: user.to_string(),
        })
    }

    /// Get or create a connection from the pool
    pub fn get_conn(&mut self) -> Result<&mut PooledConn> {
        if self.conn.is_none() {
            self.conn = Some(self.pool.get_conn()?);
            self.init_conn()?;
        }
        self.conn
            .as_mut()
            .ok_or_else(|| anyhow!("Failed to get database connection"))
    }

    /// Initialize connection settings
    fn init_conn(&mut self) -> Result<()> {
        if let Some(ref mut conn) = self.conn {
            // Set database
            if !self.database.is_empty() {
                conn.query_drop(format!("USE `{}`", self.database))?;
                debug!("Switched to database: {}", self.database);
            }

            // Set additional connection settings
            conn.query_drop("SET sql_mode = 'TRADITIONAL'")?;
            conn.query_drop("SET autocommit = 1")?;

            info!(
                "Connection initialized for {}@{}:{}",
                self.user, self.host, self.port
            );
        }
        Ok(())
    }

    /// Execute a query and return the result
    pub fn query<T: mysql::prelude::FromRow>(&mut self, sql: &str) -> Result<Vec<T>> {
        let conn = self.get_conn()?;
        let result = conn.query(sql)?;
        Ok(result)
    }

    /// Execute a query without returning results
    pub fn query_drop(&mut self, sql: &str) -> Result<()> {
        let conn = self.get_conn()?;
        conn.query_drop(sql)?;
        Ok(())
    }

    /// Execute a prepared statement
    pub fn exec_drop<P: Into<mysql::Params>>(&mut self, sql: &str, params: P) -> Result<()> {
        let conn = self.get_conn()?;
        conn.exec_drop(sql, params)?;
        Ok(())
    }

    /// Get connection info as string
    pub fn info(&self) -> String {
        format!(
            "{}@{}:{}/{}",
            self.user, self.host, self.port, self.database
        )
    }

    /// Close the connection
    pub fn close(&mut self) {
        if let Some(conn) = self.conn.take() {
            drop(conn);
            debug!("Connection closed: {}", self.info());
        }
    }
}

impl Drop for Conn {
    fn drop(&mut self) {
        self.close();
    }
}

/// Open database connection with retry logic
pub fn open_db_with_retry(
    host: &str,
    port: u16,
    user: &str,
    password: &str,
    database: &str,
    params: &str,
    max_retries: u32,
) -> Result<Conn> {
    let mut attempt = 0;
    let mut delay = Duration::from_millis(100);

    loop {
        attempt += 1;

        match Conn::new(host, port, user, password, database, params) {
            Ok(mut conn) => {
                // Test the connection
                match conn.get_conn() {
                    Ok(_) => {
                        info!("Successfully connected to database on attempt {}", attempt);
                        return Ok(conn);
                    }
                    Err(e) => {
                        warn!("Connection test failed on attempt {}: {}", attempt, e);
                        if attempt >= max_retries {
                            return Err(anyhow!(
                                "Failed to connect after {} attempts: {}",
                                max_retries,
                                e
                            ));
                        }
                    }
                }
            }
            Err(e) => {
                warn!("Connection creation failed on attempt {}: {}", attempt, e);
                if attempt >= max_retries {
                    return Err(anyhow!(
                        "Failed to create connection after {} attempts: {}",
                        max_retries,
                        e
                    ));
                }
            }
        }

        // Exponential backoff with jitter
        std::thread::sleep(delay);
        delay = std::cmp::min(delay * 2, Duration::from_secs(10));
    }
}

// Note: Integration tests with real database connections should be in the tests/ directory
