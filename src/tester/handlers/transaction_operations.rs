//! Transaction operation handlers (begin_transaction, commit_transaction, rollback_transaction)
//!
//! This module provides handlers for simplified transaction management.

use crate::tester::tester::Tester;
use anyhow::{anyhow, Result};
use log::{debug, info};

/// Handler for 'begin_transaction' command
pub fn execute_begin_transaction(tester: &mut Tester, _args: &str) -> Result<()> {
    // Get current database connection and begin transaction
    let db = tester.connection_manager.current_database()?;
    
    // Execute BEGIN statement
    db.execute("BEGIN")?;
    
    // Mark transaction as active
    tester.set_transaction_active(true);
    
    info!("Transaction started");
    debug!("BEGIN transaction executed successfully");
    Ok(())
}

/// Handler for 'commit_transaction' command
pub fn execute_commit_transaction(tester: &mut Tester, _args: &str) -> Result<()> {
    if !tester.is_transaction_active() {
        return Err(anyhow!("No active transaction to commit"));
    }
    
    // Get current database connection and commit transaction
    let db = tester.connection_manager.current_database()?;
    
    // Execute COMMIT statement
    db.execute("COMMIT")?;
    
    // Mark transaction as inactive
    tester.set_transaction_active(false);
    
    info!("Transaction committed");
    debug!("COMMIT transaction executed successfully");
    Ok(())
}

/// Handler for 'rollback_transaction' command
pub fn execute_rollback_transaction(tester: &mut Tester, _args: &str) -> Result<()> {
    if !tester.is_transaction_active() {
        return Err(anyhow!("No active transaction to rollback"));
    }
    
    // Get current database connection and rollback transaction
    let db = tester.connection_manager.current_database()?;
    
    // Execute ROLLBACK statement
    db.execute("ROLLBACK")?;
    
    // Mark transaction as inactive
    tester.set_transaction_active(false);
    
    info!("Transaction rolled back");
    debug!("ROLLBACK transaction executed successfully");
    Ok(())
}