//! Handler for the --error command.

use crate::tester::command::Command;
use crate::tester::tester::Tester;
use anyhow::Result;
use log::debug;

pub fn execute(tester: &mut Tester, cmd: &Command) -> Result<()> {
    let error_spec = &cmd.args;
    tester.expected_errors = error_spec
        .split(',')
        .map(|s| s.trim().to_string())
        .collect();
    debug!("Expected errors set to: {:?}", tester.expected_errors);
    Ok(())
}
