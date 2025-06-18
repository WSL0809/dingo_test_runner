//! Handlers for query log control commands.

use crate::tester::command::Command;
use crate::tester::tester::Tester;
use anyhow::Result;
use log::debug;

pub fn disable_query_log(tester: &mut Tester, _cmd: &Command) -> Result<()> {
    tester.enable_query_log = false;
    debug!("Query log disabled");
    Ok(())
}

pub fn enable_query_log(tester: &mut Tester, _cmd: &Command) -> Result<()> {
    tester.enable_query_log = true;
    debug!("Query log enabled");
    Ok(())
}
