//! Expression evaluation module for control flow conditions
//! 
//! This module handles evaluation of expressions used in if and while statements,
//! including variable substitution, SQL backtick expressions, and basic arithmetic/comparison operations.

use super::variables::VariableContext;
use super::database::Database;
use anyhow::Result;
use evalexpr::{eval, Value as EvalValue};
use log::debug;
use regex::Regex;
use std::collections::HashMap;

/// Expression evaluator for control flow conditions
pub struct ExpressionEvaluator {
    /// Regex for finding SQL backtick expressions
    backtick_regex: Regex,
}

impl Default for ExpressionEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

impl ExpressionEvaluator {
    /// Create a new expression evaluator
    pub fn new() -> Self {
        Self {
            backtick_regex: Regex::new(r"`([^`]+)`").expect("Invalid backtick regex"),
        }
    }

    /// Evaluate an expression to a boolean result
    /// 
    /// The expression can contain:
    /// - Variables (will be expanded using variable_context)
    /// - SQL backtick expressions (will be executed against database)
    /// - Arithmetic and comparison operations
    /// - Logical operations (&&, ||, !)
    pub fn evaluate_condition(
        &self,
        expression: &str,
        variable_context: &VariableContext,
        database: &mut Database,
    ) -> Result<bool> {
        debug!("Evaluating condition: {}", expression);

        // Step 1: Expand variables
        let expanded_expr = variable_context.expand(expression)?;
        debug!("After variable expansion: {}", expanded_expr);

        // Step 2: Execute SQL backtick expressions
        let sql_resolved_expr = self.resolve_sql_expressions(&expanded_expr, database)?;
        debug!("After SQL resolution: {}", sql_resolved_expr);

        // Step 3: Evaluate the final expression
        let result = self.evaluate_final_expression(&sql_resolved_expr)?;
        debug!("Expression '{}' evaluated to: {}", expression, result);

        Ok(result)
    }

    /// Resolve SQL backtick expressions in the string
    pub fn resolve_sql_expressions(&self, expression: &str, database: &mut Database) -> Result<String> {
        let mut result = expression.to_string();
        let mut replacements = HashMap::new();

        // Find all backtick expressions
        for captures in self.backtick_regex.captures_iter(expression) {
            // 使用安全的索引获取，避免 panic
            let (full_match, sql_query) = match (captures.get(0), captures.get(1)) {
                (Some(m0), Some(m1)) => (m0.as_str(), m1.as_str()),
                _ => continue, // 如果正则匹配异常，跳过即可
            };

            // Skip if we've already processed this exact expression
            if replacements.contains_key(full_match) {
                continue;
            }

            debug!("Executing SQL expression: {}", sql_query);

            // Execute the SQL query
            match database.query(sql_query) {
                Ok(rows) => {
                    let replacement_value = if rows.is_empty() {
                        // Empty result set
                        "0".to_string()
                    } else if rows.len() == 1 && rows[0].len() == 1 {
                        // Single scalar value
                        rows[0][0].clone()
                    } else if rows.len() == 1 {
                        // Single row, multiple columns - use first column
                        rows[0][0].clone()
                    } else {
                        // Multiple rows - use count
                        rows.len().to_string()
                    };

                    replacements.insert(full_match.to_string(), replacement_value);
                }
                Err(e) => {
                    // SQL error - treat as false/0
                    debug!("SQL expression failed: {}, treating as 0", e);
                    replacements.insert(full_match.to_string(), "0".to_string());
                }
            }
        }

        // Apply all replacements
        for (pattern, replacement) in replacements {
            result = result.replace(&pattern, &replacement);
        }

        Ok(result)
    }

    /// Evaluate the final expression using evalexpr
    fn evaluate_final_expression(&self, expression: &str) -> Result<bool> {
        // Handle empty or whitespace-only expressions
        let trimmed = expression.trim();
        if trimmed.is_empty() {
            return Ok(false);
        }

        // Try to evaluate as a mathematical/logical expression
        match eval(trimmed) {
            Ok(value) => Ok(self.value_to_bool(&value)),
            Err(_) => {
                // If evalexpr fails, try to interpret as a simple value
                self.interpret_as_simple_value(trimmed)
            }
        }
    }

    /// Convert evalexpr Value to boolean
    fn value_to_bool(&self, value: &EvalValue) -> bool {
        match value {
            EvalValue::Boolean(b) => *b,
            EvalValue::Int(i) => *i != 0,
            EvalValue::Float(f) => *f != 0.0,
            EvalValue::String(s) => !s.is_empty(),
            EvalValue::Tuple(_) => true, // Non-empty tuple is truthy
            EvalValue::Empty => false,
        }
    }

    /// Interpret a string as a simple value when evalexpr fails
    fn interpret_as_simple_value(&self, value: &str) -> Result<bool> {
        let trimmed = value.trim();

        // Try to parse as integer
        if let Ok(i) = trimmed.parse::<i64>() {
            return Ok(i != 0);
        }

        // Try to parse as float
        if let Ok(f) = trimmed.parse::<f64>() {
            return Ok(f != 0.0);
        }

        // Try to parse as boolean
        match trimmed.to_lowercase().as_str() {
            "true" | "yes" | "on" | "1" => Ok(true),
            "false" | "no" | "off" | "0" => Ok(false),
            "" => Ok(false), // Empty string is false
            _ => Ok(true),   // Non-empty string is true
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_expressions() {
        let evaluator = ExpressionEvaluator::new();
        let _var_context = VariableContext::new();

        // Test simple numeric expressions without database
        assert!(evaluator.evaluate_final_expression("1").unwrap());
        assert!(!evaluator.evaluate_final_expression("0").unwrap());
        assert!(evaluator.evaluate_final_expression("5 > 3").unwrap());
        assert!(!evaluator.evaluate_final_expression("2 > 5").unwrap());

        // Test string expressions
        assert!(evaluator.evaluate_final_expression("\"hello\"").unwrap());
        assert!(!evaluator.evaluate_final_expression("\"\"").unwrap());
    }

    #[test]
    fn test_variable_expansion() {
        let evaluator = ExpressionEvaluator::new();
        let mut var_context = VariableContext::new();

        // Set up variables
        var_context.set("test_var", "5");
        var_context.set("zero_var", "0");

        // Test variable expansion
        let expanded1 = var_context.expand("$test_var > 3").unwrap();
        assert_eq!(expanded1, "5 > 3");
        assert!(evaluator.evaluate_final_expression(&expanded1).unwrap());

        let expanded2 = var_context.expand("$zero_var").unwrap();
        assert_eq!(expanded2, "0");
        assert!(!evaluator.evaluate_final_expression(&expanded2).unwrap());
    }
} 