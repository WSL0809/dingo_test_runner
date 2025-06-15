//! Handler for the `let` command
//! 
//! This module implements the handler for variable assignment commands.

use crate::tester::variables::{VariableContext, LetStatement};
use crate::tester::command::Command;
use crate::tester::tester::Tester;
use anyhow::Result;
use std::env;

/// Execute function for the command registry
pub fn execute(tester: &mut Tester, cmd: &Command) -> Result<()> {
    LetHandler::handle(&mut tester.variable_context, &cmd.args)
}

/// Handler for the `let` command
pub struct LetHandler;

impl LetHandler {
    /// Handle a let command
    /// 
    /// Parses the let statement and updates the variable context.
    /// For environment variables (without $), also updates the process environment.
    pub fn handle(
        variable_context: &mut VariableContext,
        statement: &str,
    ) -> Result<()> {
        // Parse the let statement
        let let_stmt = variable_context.parse_let_statement(statement)?;
        
        // Execute the assignment
        Self::execute_assignment(variable_context, let_stmt)
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
            
            log::debug!("Set environment variable: {} = {}", let_stmt.name, let_stmt.value);
        } else {
            // MySQLTest variable - set only in context
            variable_context.set(&let_stmt.name, &let_stmt.value);
            
            log::debug!("Set mysqltest variable: ${} = {}", let_stmt.name, let_stmt.value);
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
        assert!(result.unwrap_err().to_string().contains("Undefined variable"));
    }
} 