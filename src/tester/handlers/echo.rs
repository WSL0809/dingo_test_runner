//! Handler for the --echo command.

use crate::tester::command::Command;
use crate::tester::tester::Tester;
use anyhow::Result;
use log::debug;
use std::io::Write;

pub fn execute(tester: &mut Tester, cmd: &Command) -> Result<()> {
    let echo_output = format!("{}\n", cmd.args);
    debug!("Echo: {}", cmd.args);
    
    // Per user feedback, echo is NOT affected by modifiers.
    if tester.args.record {
        write!(tester.output_buffer, "{}", echo_output)?;
    } else {
        tester.compare_with_result(&echo_output)?;
    }
    
    Ok(())
} 