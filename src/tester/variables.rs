//! Variable system for MySQL test runner
//! 
//! This module provides variable storage, expansion, and management functionality
//! compatible with MySQL's mysqltest variable system.

use anyhow::{anyhow, Result};
use regex::Regex;
use std::collections::HashMap;

/// Maximum recursion depth for variable expansion to prevent infinite loops
const MAX_EXPANSION_DEPTH: usize = 10;

/// Context for storing and managing test variables
#[derive(Debug, Clone)]
pub struct VariableContext {
    /// Variable storage (name -> value)
    variables: HashMap<String, String>,
    /// Regex for matching variable references ($var_name)
    var_regex: Regex,
}

impl Default for VariableContext {
    fn default() -> Self {
        Self::new()
    }
}

impl VariableContext {
    /// Create a new variable context
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
            // Match $variable_name (alphanumeric + underscore)
            var_regex: Regex::new(r"\$([a-zA-Z_][a-zA-Z0-9_]*)").expect("Invalid regex"),
        }
    }

    /// Set a variable value
    pub fn set<S1, S2>(&mut self, name: S1, value: S2)
    where
        S1: Into<String>,
        S2: Into<String>,
    {
        self.variables.insert(name.into(), value.into());
    }

    /// Get a variable value
    pub fn get(&self, name: &str) -> Option<&String> {
        self.variables.get(name)
    }

    /// Check if a variable exists
    pub fn contains(&self, name: &str) -> bool {
        self.variables.contains_key(name)
    }

    /// Remove a variable
    pub fn remove(&mut self, name: &str) -> Option<String> {
        self.variables.remove(name)
    }

    /// Clear all variables
    pub fn clear(&mut self) {
        self.variables.clear();
    }

    /// Get all variable names
    pub fn keys(&self) -> impl Iterator<Item = &String> {
        self.variables.keys()
    }

    /// Expand variables in a text string
    /// 
    /// Replaces all $variable_name occurrences with their values.
    /// Returns an error if any referenced variable is undefined.
    pub fn expand(&self, text: &str) -> Result<String> {
        self.expand_with_depth(text, 0)
    }

    /// Internal expansion with recursion depth tracking
    fn expand_with_depth(&self, text: &str, depth: usize) -> Result<String> {
        if depth > MAX_EXPANSION_DEPTH {
            return Err(anyhow!("Variable expansion exceeded maximum depth ({})", MAX_EXPANSION_DEPTH));
        }

        let mut result = text.to_string();
        let mut has_variables = false;

        // Find all variable references
        for captures in self.var_regex.captures_iter(text) {
            let full_match = captures.get(0).unwrap().as_str(); // $var_name
            let var_name = captures.get(1).unwrap().as_str();   // var_name

            match self.variables.get(var_name) {
                Some(value) => {
                    result = result.replace(full_match, value);
                    has_variables = true;
                }
                None => {
                    return Err(anyhow!("Undefined variable: ${}", var_name));
                }
            }
        }

        // If we made substitutions, recursively expand in case the values contain variables
        if has_variables {
            self.expand_with_depth(&result, depth + 1)
        } else {
            Ok(result)
        }
    }

    /// Parse a let statement and extract variable name and value
    /// 
    /// Supports formats:
    /// - `let $var_name = value`
    /// - `let VAR_NAME = value` (environment variable)
    pub fn parse_let_statement(&self, statement: &str) -> Result<LetStatement> {
        let statement = statement.trim();
        
        // Remove 'let' keyword
        let statement = if statement.to_lowercase().starts_with("let ") {
            statement[4..].trim()
        } else {
            statement
        };

        // Find the = sign
        let eq_pos = statement.find('=')
            .ok_or_else(|| anyhow!("Invalid let statement: missing '=' in '{}'", statement))?;

        let name_part = statement[..eq_pos].trim();
        let value_part = statement[eq_pos + 1..].trim();

        // Check if it's a mysqltest variable ($var) or environment variable (VAR)
        let (var_name, is_env) = if name_part.starts_with('$') {
            (&name_part[1..], false)
        } else {
            (name_part, true)
        };

        // Validate variable name
        if var_name.is_empty() {
            return Err(anyhow!("Empty variable name in let statement"));
        }

        // Validate variable name: must start with letter or underscore, then alphanumeric or underscore
        if var_name.is_empty() || 
           (!var_name.chars().next().unwrap().is_alphabetic() && var_name.chars().next().unwrap() != '_') ||
           !var_name.chars().all(|c| c.is_alphanumeric() || c == '_') {
            return Err(anyhow!("Invalid variable name '{}': must start with letter or underscore, followed by alphanumeric characters or underscores", var_name));
        }

        // Expand variables in the value
        let expanded_value = self.expand(value_part)?;

        Ok(LetStatement {
            name: var_name.to_string(),
            value: expanded_value,
            is_env,
        })
    }
}

/// Represents a parsed let statement
#[derive(Debug, Clone, PartialEq)]
pub struct LetStatement {
    pub name: String,
    pub value: String,
    pub is_env: bool, // true for environment variables, false for mysqltest variables
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_variable_operations() {
        let mut ctx = VariableContext::new();
        
        // Test set and get
        ctx.set("test_var", "test_value");
        assert_eq!(ctx.get("test_var"), Some(&"test_value".to_string()));
        
        // Test contains
        assert!(ctx.contains("test_var"));
        assert!(!ctx.contains("nonexistent"));
        
        // Test remove
        assert_eq!(ctx.remove("test_var"), Some("test_value".to_string()));
        assert!(!ctx.contains("test_var"));
    }

    #[test]
    fn test_simple_expansion() {
        let mut ctx = VariableContext::new();
        ctx.set("name", "world");
        
        let result = ctx.expand("Hello $name!").unwrap();
        assert_eq!(result, "Hello world!");
    }

    #[test]
    fn test_multiple_variables() {
        let mut ctx = VariableContext::new();
        ctx.set("first", "Hello");
        ctx.set("second", "world");
        
        let result = ctx.expand("$first $second!").unwrap();
        assert_eq!(result, "Hello world!");
    }

    #[test]
    fn test_recursive_expansion() {
        let mut ctx = VariableContext::new();
        ctx.set("base", "world");
        ctx.set("greeting", "Hello $base");
        
        let result = ctx.expand("$greeting!").unwrap();
        assert_eq!(result, "Hello world!");
    }

    #[test]
    fn test_undefined_variable_error() {
        let ctx = VariableContext::new();
        
        let result = ctx.expand("Hello $undefined!");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Undefined variable: $undefined"));
    }

    #[test]
    fn test_infinite_recursion_protection() {
        let mut ctx = VariableContext::new();
        ctx.set("a", "$b");
        ctx.set("b", "$a");
        
        let result = ctx.expand("$a");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("exceeded maximum depth"));
    }

    #[test]
    fn test_parse_let_statement_mysqltest_var() {
        let ctx = VariableContext::new();
        
        let stmt = ctx.parse_let_statement("let $test_var = hello world").unwrap();
        assert_eq!(stmt.name, "test_var");
        assert_eq!(stmt.value, "hello world");
        assert!(!stmt.is_env);
    }

    #[test]
    fn test_parse_let_statement_env_var() {
        let ctx = VariableContext::new();
        
        let stmt = ctx.parse_let_statement("let TEST_VAR = hello world").unwrap();
        assert_eq!(stmt.name, "TEST_VAR");
        assert_eq!(stmt.value, "hello world");
        assert!(stmt.is_env);
    }

    #[test]
    fn test_parse_let_statement_with_expansion() {
        let mut ctx = VariableContext::new();
        ctx.set("base", "world");
        
        let stmt = ctx.parse_let_statement("let $greeting = Hello $base").unwrap();
        assert_eq!(stmt.name, "greeting");
        assert_eq!(stmt.value, "Hello world");
        assert!(!stmt.is_env);
    }

    #[test]
    fn test_parse_let_statement_invalid() {
        let ctx = VariableContext::new();
        
        // Missing =
        assert!(ctx.parse_let_statement("let $var").is_err());
        
        // Empty variable name
        assert!(ctx.parse_let_statement("let $ = value").is_err());
        
        // Invalid variable name
        assert!(ctx.parse_let_statement("let $var-name = value").is_err());
    }

    #[test]
    fn test_no_variables_in_text() {
        let ctx = VariableContext::new();
        
        let result = ctx.expand("This text has no variables").unwrap();
        assert_eq!(result, "This text has no variables");
    }

    #[test]
    fn test_variable_name_validation() {
        let ctx = VariableContext::new();
        
        // Valid names
        assert!(ctx.parse_let_statement("let $valid_name = value").is_ok());
        assert!(ctx.parse_let_statement("let $valid123 = value").is_ok());
        assert!(ctx.parse_let_statement("let $_underscore = value").is_ok());
        
        // Invalid names
        assert!(ctx.parse_let_statement("let $123invalid = value").is_err());
        assert!(ctx.parse_let_statement("let $invalid-name = value").is_err());
        assert!(ctx.parse_let_statement("let $invalid.name = value").is_err());
    }
} 