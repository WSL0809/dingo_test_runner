//! Batch operation handlers (batch_insert, batch_execute, end_batch)
//!
//! This module provides handlers for batch database operations.

use crate::tester::tester::Tester;
use anyhow::{anyhow, Result};
use log::{debug, info};

/// Handler for 'batch_insert table_name' command
pub fn execute_batch_insert(tester: &mut Tester, args: &str) -> Result<()> {
    let table_name = args.trim();
    if table_name.is_empty() {
        return Err(anyhow!("batch_insert requires a table name"));
    }

    // Start batch insert mode
    tester.start_batch_insert(table_name.to_string())?;
    
    debug!("Started batch insert for table: {}", table_name);
    Ok(())
}

/// Handler for 'batch_execute' command
pub fn execute_batch_execute(tester: &mut Tester, _args: &str) -> Result<()> {
    // Start batch execute mode
    tester.start_batch_execute()?;
    
    debug!("Started batch execute mode");
    Ok(())
}

/// Handler for 'end_batch' command
pub fn execute_end_batch(tester: &mut Tester, _args: &str) -> Result<()> {
    // End current batch operation and execute
    let result = tester.end_batch()?;
    
    info!("Completed batch operation: {} statements executed", result);
    Ok(())
}