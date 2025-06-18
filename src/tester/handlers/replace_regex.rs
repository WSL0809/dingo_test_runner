//! Handler for the --replace_regex command.

use crate::tester::command::Command;
use crate::tester::tester::Tester;
use anyhow::{anyhow, Result};
use log::debug;
use regex::Regex;

pub fn execute(tester: &mut Tester, cmd: &Command) -> Result<()> {
    let pattern = &cmd.args;

    if !pattern.starts_with('/') || !pattern.ends_with('/') || pattern.len() < 3 {
        return Err(anyhow!(
            "Invalid replace_regex: must be /regex/replacement/. Got: {}",
            pattern
        ));
    }

    let inner = &pattern[1..pattern.len() - 1];

    let mut parts = Vec::with_capacity(2);
    let mut current_part = String::new();
    let mut in_escape = false;

    for char in inner.chars() {
        if in_escape {
            current_part.push(char);
            in_escape = false;
        } else if char == '\\' {
            in_escape = true;
            current_part.push(char);
        } else if char == '/' && parts.is_empty() {
            parts.push(current_part);
            current_part = String::new();
        } else {
            current_part.push(char);
        }
    }
    parts.push(current_part);

    if parts.len() != 2 {
        return Err(anyhow!("Invalid replace_regex format. Got: {}", pattern));
    }

    let regex = Regex::new(&parts[0])?;
    tester
        .pending_replace_regex
        .push((regex, parts[1].to_string()));

    debug!("Replace regex added: {} -> {}", parts[0], parts[1]);
    Ok(())
}
