//! Handler for the --sleep command.

use crate::tester::command::Command;
use crate::tester::tester::Tester;
use anyhow::{anyhow, Result};
use log::debug;

pub fn execute(_tester: &mut Tester, cmd: &Command) -> Result<()> {
    let duration: f64 = cmd
        .args
        .parse()
        .map_err(|_| anyhow!("Invalid sleep duration: '{}'", cmd.args))?;
    std::thread::sleep(std::time::Duration::from_secs_f64(duration));
    debug!("Slept for {} seconds", duration);
    Ok(())
} 