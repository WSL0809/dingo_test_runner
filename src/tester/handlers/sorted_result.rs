//! Handler for the --sorted_result command.

use crate::tester::command::Command;
use crate::tester::tester::Tester;
use anyhow::Result;
use log::debug;

pub fn execute(tester: &mut Tester, _cmd: &Command) -> Result<()> {
    tester.pending_sorted_result = true;
    debug!("Sorted result enabled for next query");
    Ok(())
}
