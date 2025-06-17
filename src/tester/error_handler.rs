//! Error handling for MySQL test runner
//! 
//! This module provides error mapping from MySQL error codes to standardized
//! error representations, making it easier to handle expected errors in test cases.

use std::collections::HashMap;

/// MySQL error code mappings
/// Based on the MySQL error codes from the original Go implementation
pub struct MySQLErrorHandler {
    /// Error name to error number mapping
    error_name_map: HashMap<String, u16>,
}

impl MySQLErrorHandler {
    /// Create a new MySQL error handler with standard error mappings
    pub fn new() -> Self {
        let mut error_name_map = HashMap::new();
        
        // Common MySQL error codes
        error_name_map.insert("ER_DUP_KEY".to_string(), 1022);
        error_name_map.insert("ER_DUP_ENTRY".to_string(), 1062);
        error_name_map.insert("ER_NO_SUCH_TABLE".to_string(), 1146);
        error_name_map.insert("ER_TABLE_EXISTS_ERROR".to_string(), 1050);
        error_name_map.insert("ER_BAD_FIELD_ERROR".to_string(), 1054);
        error_name_map.insert("ER_PARSE_ERROR".to_string(), 1064);
        error_name_map.insert("ER_NO_SUCH_INDEX".to_string(), 1082);
        error_name_map.insert("ER_KEY_COLUMN_DOES_NOT_EXITS".to_string(), 1072);
        error_name_map.insert("ER_WRONG_VALUE_COUNT_ON_ROW".to_string(), 1136);
        error_name_map.insert("ER_ACCESS_DENIED_ERROR".to_string(), 1045);
        error_name_map.insert("ER_DBACCESS_DENIED_ERROR".to_string(), 1044);
        error_name_map.insert("ER_TABLEACCESS_DENIED_ERROR".to_string(), 1142);
        error_name_map.insert("ER_COLUMNACCESS_DENIED_ERROR".to_string(), 1143);
        error_name_map.insert("ER_DATA_TOO_LONG".to_string(), 1406);
        error_name_map.insert("ER_WARN_DATA_OUT_OF_RANGE".to_string(), 1264);
        error_name_map.insert("ER_DIVISION_BY_ZERO".to_string(), 1365);
        error_name_map.insert("ER_BAD_NULL_ERROR".to_string(), 1048);
        error_name_map.insert("ER_NON_UNIQ_ERROR".to_string(), 1052);
        error_name_map.insert("ER_WRONG_AUTO_KEY".to_string(), 1075);
        error_name_map.insert("ER_WRONG_VALUE_FOR_VAR".to_string(), 1231);
        error_name_map.insert("ER_UNKNOWN_SYSTEM_VARIABLE".to_string(), 1193);
        error_name_map.insert("ER_CANT_DROP_FIELD_OR_KEY".to_string(), 1091);
        error_name_map.insert("ER_TOO_MANY_KEYS".to_string(), 1069);
        error_name_map.insert("ER_TOO_MANY_KEY_PARTS".to_string(), 1070);
        error_name_map.insert("ER_TOO_LONG_KEY".to_string(), 1071);
        error_name_map.insert("ER_BLOB_USED_AS_KEY".to_string(), 1073);
        error_name_map.insert("ER_TOO_BIG_FIELDLENGTH".to_string(), 1074);
        error_name_map.insert("ER_WRONG_FIELD_SPEC".to_string(), 1063);
        error_name_map.insert("ER_EMPTY_QUERY".to_string(), 1065);
        error_name_map.insert("ER_NONUNIQ_TABLE".to_string(), 1066);
        error_name_map.insert("ER_INVALID_DEFAULT".to_string(), 1067);
        error_name_map.insert("ER_MULTIPLE_PRI_KEY".to_string(), 1068);
        
        Self { error_name_map }
    }

    /// Check if an error matches expected error conditions
    /// Returns true if the error matches any of the expected error codes/names
    pub fn check_expected_error(&self, error: &mysql::Error, expected_errors: &[String]) -> bool {
        if expected_errors.is_empty() {
            return false;
        }

        // Extract MySQL error code from the error
        let error_code = match error {
            mysql::Error::MySqlError(mysql_err) => Some(mysql_err.code),
            _ => None,
        };

        for expected in expected_errors {
            let expected = expected.trim();
            
            // Special case: "0" means accept any error
            if expected == "0" {
                return true;
            }

            // Try to parse as error number
            if let Ok(expected_code) = expected.parse::<u16>() {
                if let Some(code) = error_code {
                    if code == expected_code {
                        return true;
                    }
                }
            }

            // Try to match by error name
            if let Some(&mapped_code) = self.error_name_map.get(expected) {
                if let Some(code) = error_code {
                    if code == mapped_code {
                        return true;
                    }
                }
            }
        }

        false
    }

    /// Get error code from MySQL error
    pub fn get_error_code(&self, error: &mysql::Error) -> Option<u16> {
        match error {
            mysql::Error::MySqlError(mysql_err) => Some(mysql_err.code),
            _ => None,
        }
    }

    /// Format error message for output
    pub fn format_error(&self, error: &mysql::Error) -> String {
        match error {
            mysql::Error::MySqlError(mysql_err) => {
                format!("ERROR {} ({}): {}", mysql_err.code, mysql_err.state, mysql_err.message)
            }
            mysql::Error::DriverError(driver_err) => {
                format!("ERROR (Driver): {}", driver_err)
            }
            _ => format!("ERROR: {}", error),
        }
    }
}

impl Default for MySQLErrorHandler {
    fn default() -> Self {
        Self::new()
    }
}

/// Error checking result
#[derive(Debug, Clone)]
pub enum ErrorCheckResult {
    /// No error expected, no error occurred
    NoError,
    /// Error expected and matched
    ExpectedError(String),
    /// Error expected but didn't match
    UnexpectedError { expected: Vec<String>, actual: String },
    /// No error expected but error occurred
    UnexpectedErrorOccurred(String),
}

impl ErrorCheckResult {
    /// Check if the result represents success
    pub fn is_success(&self) -> bool {
        matches!(self, ErrorCheckResult::NoError | ErrorCheckResult::ExpectedError(_))
    }

    /// Get error message if this represents a failure
    pub fn error_message(&self) -> Option<String> {
        match self {
            ErrorCheckResult::NoError | ErrorCheckResult::ExpectedError(_) => None,
            ErrorCheckResult::UnexpectedError { expected, actual } => {
                Some(format!("Expected error(s) {:?}, but got: {}", expected, actual))
            }
            ErrorCheckResult::UnexpectedErrorOccurred(msg) => {
                Some(format!("Unexpected error: {}", msg))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_handler_creation() {
        let handler = MySQLErrorHandler::new();
        assert!(handler.error_name_map.contains_key("ER_DUP_KEY"));
        assert_eq!(handler.error_name_map.get("ER_DUP_KEY"), Some(&1022));
    }

    #[test]
    fn test_error_check_result() {
        let result = ErrorCheckResult::NoError;
        assert!(result.is_success());
        assert!(result.error_message().is_none());

        let result = ErrorCheckResult::UnexpectedErrorOccurred("test error".to_string());
        assert!(!result.is_success());
        assert!(result.error_message().is_some());
    }
} 