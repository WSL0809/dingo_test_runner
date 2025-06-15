//! Handler for the --connection command (switch connection).

use crate::tester::command::Command;
use crate::tester::tester::Tester;
use anyhow::Result;
use log::info;

pub fn execute(tester: &mut Tester, cmd: &Command) -> Result<()> {
    // Expand variables in connection name
    let expanded_conn_name = tester.variable_context.expand(&cmd.args)?;
    let conn_name = expanded_conn_name.trim();
    tester.connection_manager.switch_connection(conn_name)?;
    info!("Switched to connection: {}", conn_name);
    Ok(())
} 