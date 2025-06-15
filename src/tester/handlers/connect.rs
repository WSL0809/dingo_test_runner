//! Handler for the --connect command.

use crate::tester::command::Command;
use crate::tester::tester::Tester;
use anyhow::Result;
use log::info;

pub fn execute(tester: &mut Tester, cmd: &Command) -> Result<()> {
    // Expand variables in connection parameters
    let expanded_args = tester.variable_context.expand(&cmd.args)?;
    tester.connection_manager.connect(&expanded_args)?;
    info!("Connected to new database connection: {}", expanded_args);
    Ok(())
} 