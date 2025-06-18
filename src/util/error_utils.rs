//! Error handling utilities
//!
//! This module provides helper functions and types for better error handling
//! throughout the codebase, reducing the need for unwrap() calls.

use anyhow::{Context, Result};
use std::fs;
use std::io;
use std::path::Path;

/// Extension trait for Option types to provide better error messages
pub trait OptionExt<T> {
    /// Convert Option to Result with a context message
    fn or_context<C>(self, context: C) -> Result<T>
    where
        C: std::fmt::Display + Send + Sync + 'static;
}

impl<T> OptionExt<T> for Option<T> {
    fn or_context<C>(self, context: C) -> Result<T>
    where
        C: std::fmt::Display + Send + Sync + 'static,
    {
        self.ok_or_else(|| anyhow::anyhow!("{}", context))
    }
}

/// Safe file operations with better error messages
pub struct SafeFs;

impl SafeFs {
    /// Create a directory with all parent directories, with context
    pub fn create_dir_all<P: AsRef<Path>>(path: P) -> Result<()> {
        let path = path.as_ref();
        fs::create_dir_all(path)
            .with_context(|| format!("Failed to create directory: {}", path.display()))
    }

    /// Read file to string with context
    pub fn read_to_string<P: AsRef<Path>>(path: P) -> Result<String> {
        let path = path.as_ref();
        fs::read_to_string(path).with_context(|| format!("Failed to read file: {}", path.display()))
    }

    /// Write string to file with context
    pub fn write<P: AsRef<Path>, C: AsRef<[u8]>>(path: P, contents: C) -> Result<()> {
        let path = path.as_ref();
        fs::write(path, contents)
            .with_context(|| format!("Failed to write file: {}", path.display()))
    }

    /// Remove file with context
    pub fn remove_file<P: AsRef<Path>>(path: P) -> Result<()> {
        let path = path.as_ref();
        fs::remove_file(path).with_context(|| format!("Failed to remove file: {}", path.display()))
    }

    /// Remove directory and all contents with context
    pub fn remove_dir_all<P: AsRef<Path>>(path: P) -> Result<()> {
        let path = path.as_ref();
        fs::remove_dir_all(path)
            .with_context(|| format!("Failed to remove directory: {}", path.display()))
    }
}

/// Safe IO operations with better error messages
pub struct SafeIo;

impl SafeIo {
    /// Write to a writer with context
    pub fn write_line<W: io::Write>(writer: &mut W, line: &str) -> Result<()> {
        writeln!(writer, "{}", line).with_context(|| "Failed to write line")
    }
}

/// Convert string to other types with better error messages
pub struct SafeParse;

impl SafeParse {
    /// Parse string to u16 with context
    pub fn parse_u16(s: &str, field_name: &str) -> Result<u16> {
        s.parse::<u16>()
            .with_context(|| format!("Failed to parse {} as u16: '{}'", field_name, s))
    }

    /// Parse string to u32 with context
    pub fn parse_u32(s: &str, field_name: &str) -> Result<u32> {
        s.parse::<u32>()
            .with_context(|| format!("Failed to parse {} as u32: '{}'", field_name, s))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_option_ext() {
        let some_value: Option<i32> = Some(42);
        let result = some_value.or_context("Expected value");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);

        let none_value: Option<i32> = None;
        let result = none_value.or_context("Value was None");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Value was None"));
    }

    #[test]
    fn test_safe_parse() {
        let result = SafeParse::parse_u16("123", "port");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 123);

        let result = SafeParse::parse_u16("abc", "port");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Failed to parse port"));
    }
}
