//! Handler for the --eval command
//!
//! The eval command provides variable substitution before executing SQL statements or mysqltest commands.
//! It takes a statement string, replaces all mysqltest variables (like $var_name) with their current values,
//! and then executes the resulting string as a new statement.

use crate::tester::command::Command;
use crate::tester::query::{Query, QueryOptions, QueryType};
use crate::tester::tester::Tester;
use anyhow::Result;
use log::debug;

/// Execute function for the command registry
pub fn execute(tester: &mut Tester, cmd: &Command) -> Result<()> {
    debug!("Executing eval command: {}", cmd.args);
    
    // Step 1: Expand variables in the statement
    let expanded_statement = tester.variable_context.expand(&cmd.args)?;
    debug!("After variable expansion: {}", expanded_statement);
    
    // Step 2: Create a Query object and execute it directly as SQL
    // The eval command essentially creates a dynamic SQL query after variable substitution
    let eval_query = Query {
        query_type: QueryType::Query,
        query: expanded_statement,
        line: cmd.line,
        options: QueryOptions::default(),
    };
    
    // Step 3: Execute the expanded SQL directly
    tester.execute_sql_query_public(&eval_query.query, eval_query.line)?;
    
    Ok(())
} 