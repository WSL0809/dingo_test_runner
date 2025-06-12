//! Handler for the --disconnect command.

use crate::tester::command::Command;
use crate::tester::tester::Tester;
use anyhow::Result;
use log::info;

pub fn execute(tester: &mut Tester, cmd: &Command) -> Result<()> {
    let conn_name = cmd.args.trim();
    tester.connection_manager.disconnect(conn_name)?;
    info!("Disconnected connection: {}", conn_name);
    Ok(())
} 