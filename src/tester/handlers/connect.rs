//! Handler for the --connect command.

use crate::tester::command::Command;
use crate::tester::tester::Tester;
use anyhow::Result;
use log::info;

pub fn execute(tester: &mut Tester, cmd: &Command) -> Result<()> {
    tester.connection_manager.connect(&cmd.args)?;
    info!("Connected to new database connection: {}", cmd.args);
    Ok(())
} 