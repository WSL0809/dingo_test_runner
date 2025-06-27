//! Variable operation handlers (inc, dec, add, sub)
//!
//! This module provides handlers for variable arithmetic operations.

use crate::tester::tester::Tester;
use anyhow::{anyhow, Result};
use log::debug;

/// Handler for 'inc $var' command - increment variable by 1
pub fn execute_inc(tester: &mut Tester, args: &str) -> Result<()> {
    let var_name = args.trim();
    if !var_name.starts_with('$') {
        return Err(anyhow!("Variable name must start with $: {}", var_name));
    }

    let var_key = &var_name[1..]; // Remove $ prefix
    
    // Get current value or default to 0
    let current_value = tester.variable_context
        .get(var_key)
        .map(|v| v.parse::<i64>().unwrap_or(0))
        .unwrap_or(0);
    
    let new_value = current_value + 1;
    tester.variable_context.set(var_key, new_value.to_string());
    
    debug!("Incremented {}: {} -> {}", var_name, current_value, new_value);
    Ok(())
}

/// Handler for 'dec $var' command - decrement variable by 1
pub fn execute_dec(tester: &mut Tester, args: &str) -> Result<()> {
    let var_name = args.trim();
    if !var_name.starts_with('$') {
        return Err(anyhow!("Variable name must start with $: {}", var_name));
    }

    let var_key = &var_name[1..]; // Remove $ prefix
    
    // Get current value or default to 0
    let current_value = tester.variable_context
        .get(var_key)
        .map(|v| v.parse::<i64>().unwrap_or(0))
        .unwrap_or(0);
    
    let new_value = current_value - 1;
    tester.variable_context.set(var_key, new_value.to_string());
    
    debug!("Decremented {}: {} -> {}", var_name, current_value, new_value);
    Ok(())
}

/// Handler for 'add $var, N' command - add N to variable
pub fn execute_add(tester: &mut Tester, args: &str) -> Result<()> {
    let parts: Vec<&str> = args.split(',').map(|s| s.trim()).collect();
    if parts.len() != 2 {
        return Err(anyhow!("add command requires format: add $var, value"));
    }

    let var_name = parts[0];
    let value_str = parts[1];

    if !var_name.starts_with('$') {
        return Err(anyhow!("Variable name must start with $: {}", var_name));
    }

    let var_key = &var_name[1..]; // Remove $ prefix
    
    // Parse the value to add (could be a variable or number)
    let add_value = if value_str.starts_with('$') {
        let value_var_key = &value_str[1..];
        tester.variable_context
            .get(value_var_key)
            .map(|v| v.parse::<i64>().unwrap_or(0))
            .unwrap_or(0)
    } else {
        value_str.parse::<i64>().map_err(|_| {
            anyhow!("Invalid number or variable: {}", value_str)
        })?
    };
    
    // Get current value or default to 0
    let current_value = tester.variable_context
        .get(var_key)
        .map(|v| v.parse::<i64>().unwrap_or(0))
        .unwrap_or(0);
    
    let new_value = current_value + add_value;
    tester.variable_context.set(var_key, new_value.to_string());
    
    debug!("Added {} to {}: {} -> {}", add_value, var_name, current_value, new_value);
    Ok(())
}

/// Handler for 'sub $var, N' command - subtract N from variable
pub fn execute_sub(tester: &mut Tester, args: &str) -> Result<()> {
    let parts: Vec<&str> = args.split(',').map(|s| s.trim()).collect();
    if parts.len() != 2 {
        return Err(anyhow!("sub command requires format: sub $var, value"));
    }

    let var_name = parts[0];
    let value_str = parts[1];

    if !var_name.starts_with('$') {
        return Err(anyhow!("Variable name must start with $: {}", var_name));
    }

    let var_key = &var_name[1..]; // Remove $ prefix
    
    // Parse the value to subtract (could be a variable or number)
    let sub_value = if value_str.starts_with('$') {
        let value_var_key = &value_str[1..];
        tester.variable_context
            .get(value_var_key)
            .map(|v| v.parse::<i64>().unwrap_or(0))
            .unwrap_or(0)
    } else {
        value_str.parse::<i64>().map_err(|_| {
            anyhow!("Invalid number or variable: {}", value_str)
        })?
    };
    
    // Get current value or default to 0
    let current_value = tester.variable_context
        .get(var_key)
        .map(|v| v.parse::<i64>().unwrap_or(0))
        .unwrap_or(0);
    
    let new_value = current_value - sub_value;
    tester.variable_context.set(var_key, new_value.to_string());
    
    debug!("Subtracted {} from {}: {} -> {}", sub_value, var_name, current_value, new_value);
    Ok(())
}