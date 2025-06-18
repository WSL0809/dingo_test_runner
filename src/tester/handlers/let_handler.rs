//! Handler for the `let` command
//!
//! This module implements the handler for variable assignment commands.

use crate::tester::command::Command;
use crate::tester::database::Database;
use crate::tester::expression::ExpressionEvaluator;
use crate::tester::tester::Tester;
use crate::tester::variables::{LetStatement, VariableContext};
use anyhow::Result;
use evalexpr::{eval, Value as EvalValue};
use log::debug;
use std::env;

/// Execute function for the command registry
pub fn execute(tester: &mut Tester, cmd: &Command) -> Result<()> {
    debug!("Executing let command: {}", cmd.args);

    // We need to avoid borrowing conflicts, so we'll create a new evaluator instance
    // This is acceptable since ExpressionEvaluator is lightweight and stateless
    let expression_evaluator = ExpressionEvaluator::new();

    let result = LetHandler::handle_with_context(
        &mut tester.variable_context,
        &expression_evaluator,
        tester.connection_manager.current_database()?,
        &cmd.args,
    );

    match &result {
        Ok(_) => debug!("Let command executed successfully"),
        Err(e) => debug!("Let command failed: {}", e),
    }

    result
}

/// Handler for the `let` command
pub struct LetHandler;

impl LetHandler {
    /// Handle a let command with expression evaluation support
    ///
    /// This method implements "optimistic evaluation":
    /// 1. Parse the let statement and expand variables
    /// 2. Try to resolve SQL backtick expressions
    /// 3. Try to evaluate as mathematical/logical expression
    /// 4. If evaluation fails, fall back to literal string assignment
    pub fn handle_with_context(
        variable_context: &mut VariableContext,
        expression_evaluator: &ExpressionEvaluator,
        database: &mut Database,
        statement: &str,
    ) -> Result<()> {
        // Parse the let statement (this does basic variable expansion)
        let let_stmt = variable_context.parse_let_statement(statement)?;

        // Try optimistic evaluation on the expanded value
        let final_value = Self::evaluate_value_optimistically(
            &let_stmt.value,
            variable_context,
            expression_evaluator,
            database,
        )?;

        // Create a new LetStatement with the evaluated value
        let evaluated_stmt = LetStatement {
            name: let_stmt.name,
            value: final_value,
            is_env: let_stmt.is_env,
        };

        // Execute the assignment
        Self::execute_assignment(variable_context, evaluated_stmt)
    }

    /// Legacy handle method for backward compatibility (without expression evaluation)
    pub fn handle(variable_context: &mut VariableContext, statement: &str) -> Result<()> {
        // Parse the let statement
        let let_stmt = variable_context.parse_let_statement(statement)?;

        // Execute the assignment without expression evaluation
        Self::execute_assignment(variable_context, let_stmt)
    }

    /// Evaluate a value optimistically: try expression evaluation, fall back to literal
    fn evaluate_value_optimistically(
        value: &str,
        _variable_context: &VariableContext,
        expression_evaluator: &ExpressionEvaluator,
        database: &mut Database,
    ) -> Result<String> {
        debug!("Evaluating value optimistically: {}", value);

        // Step 1: The value has already been variable-expanded by parse_let_statement
        // Step 2: Try to resolve SQL backtick expressions
        let sql_resolved = match expression_evaluator.resolve_sql_expressions(value, database) {
            Ok(resolved) => {
                debug!("After SQL resolution: {}", resolved);
                resolved
            }
            Err(e) => {
                debug!("SQL resolution failed: {}, using original value", e);
                value.to_string()
            }
        };

        // Step 3: Try to evaluate as mathematical/logical expression
        match Self::try_evaluate_expression(&sql_resolved) {
            Ok(result) => {
                debug!(
                    "Expression evaluation succeeded: {} -> {}",
                    sql_resolved, result
                );
                Ok(result)
            }
            Err(_) => {
                debug!(
                    "Expression evaluation failed, using literal value: {}",
                    sql_resolved
                );
                Ok(sql_resolved)
            }
        }
    }

    /// Try to evaluate a string as a mathematical/logical expression
    fn try_evaluate_expression(expr: &str) -> Result<String> {
        let trimmed = expr.trim();

        // Handle empty expressions
        if trimmed.is_empty() {
            return Ok(String::new());
        }

        // Try evalexpr evaluation
        match eval(trimmed) {
            Ok(value) => Ok(Self::eval_value_to_string(&value)),
            Err(e) => {
                debug!("evalexpr failed for '{}': {}", trimmed, e);
                Err(anyhow::anyhow!("Expression evaluation failed: {}", e))
            }
        }
    }

    /// Convert evalexpr Value to string for storage
    fn eval_value_to_string(value: &EvalValue) -> String {
        match value {
            EvalValue::Boolean(b) => {
                if *b {
                    "1".to_string()
                } else {
                    "0".to_string()
                }
            }
            EvalValue::Int(i) => i.to_string(),
            EvalValue::Float(f) => f.to_string(),
            EvalValue::String(s) => s.clone(),
            EvalValue::Tuple(t) => format!("{:?}", t), // Fallback for tuples
            EvalValue::Empty => String::new(),
        }
    }

    /// Execute the variable assignment
    fn execute_assignment(
        variable_context: &mut VariableContext,
        let_stmt: LetStatement,
    ) -> Result<()> {
        if let_stmt.is_env {
            // Environment variable - set both in context and process environment
            variable_context.set(&let_stmt.name, &let_stmt.value);
            env::set_var(&let_stmt.name, &let_stmt.value);

            debug!(
                "Set environment variable: {} = {}",
                let_stmt.name, let_stmt.value
            );
        } else {
            // MySQLTest variable - set only in context
            variable_context.set(&let_stmt.name, &let_stmt.value);

            debug!(
                "Set mysqltest variable: ${} = {}",
                let_stmt.name, let_stmt.value
            );
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_handle_mysqltest_variable() {
        let mut ctx = VariableContext::new();

        LetHandler::handle(&mut ctx, "let $test_var = hello world").unwrap();

        assert_eq!(ctx.get("test_var"), Some(&"hello world".to_string()));
    }

    #[test]
    fn test_handle_environment_variable() {
        let mut ctx = VariableContext::new();
        let var_name = "TEST_ENV_VAR_12345"; // Use unique name to avoid conflicts

        // Clean up any existing value
        env::remove_var(var_name);

        LetHandler::handle(&mut ctx, &format!("let {} = test_value", var_name)).unwrap();

        // Check both context and environment
        assert_eq!(ctx.get(var_name), Some(&"test_value".to_string()));
        assert_eq!(env::var(var_name).unwrap(), "test_value");

        // Clean up
        env::remove_var(var_name);
    }

    #[test]
    fn test_handle_variable_expansion() {
        let mut ctx = VariableContext::new();
        ctx.set("base", "world");

        LetHandler::handle(&mut ctx, "let $greeting = Hello $base").unwrap();

        assert_eq!(ctx.get("greeting"), Some(&"Hello world".to_string()));
    }

    #[test]
    fn test_handle_invalid_statement() {
        let mut ctx = VariableContext::new();

        let result = LetHandler::handle(&mut ctx, "let invalid");
        assert!(result.is_err());
    }

    #[test]
    fn test_handle_undefined_variable_in_value() {
        let ctx = VariableContext::new();

        let result = ctx.parse_let_statement("let $test = $undefined");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Undefined variable"));
    }
}
