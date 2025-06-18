//! Handlers for result log control commands.

use crate::tester::command::Command;
use crate::tester::tester::Tester;
use anyhow::Result;
use log::debug;

pub fn disable_result_log(tester: &mut Tester, _cmd: &Command) -> Result<()> {
    tester.enable_result_log = false;
    debug!("Result log disabled");
    Ok(())
}

pub fn enable_result_log(tester: &mut Tester, _cmd: &Command) -> Result<()> {
    tester.enable_result_log = true;
    debug!("Result log enabled");
    Ok(())
}
