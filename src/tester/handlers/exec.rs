//! Handler for the --exec command.

use crate::tester::command::Command;
use crate::tester::tester::Tester;
use anyhow::{anyhow, Result};
use log::{debug, warn};
use std::io::Write;
use std::process;

pub fn execute(tester: &mut Tester, cmd: &Command) -> Result<()> {
    // Expand variables in the shell command
    let expanded_command = tester.variable_context.expand(&cmd.args)?;
    debug!("Executing shell command: {}", expanded_command);
    
    // Execute the shell command
    let output = if cfg!(target_os = "windows") {
        process::Command::new("cmd")
            .args(["/C", &expanded_command])
            .output()
    } else {
        process::Command::new("sh")
            .args(["-c", &expanded_command])
            .output()
    };
    
    match output {
        Ok(output) => {
            let exit_code = output.status.code().unwrap_or(-1);
            let stdout_str = String::from_utf8_lossy(&output.stdout).to_string();
            
            // Handle expected errors
            if !tester.expected_errors.is_empty() {
                let error_occurred = exit_code != 0;
                
                if error_occurred {
                    // Check if the exit code matches expected errors
                    let exit_code_str = exit_code.to_string();
                    let is_expected = tester.expected_errors.iter().any(|expected| {
                        expected == "0" || expected == &exit_code_str || expected == "1"
                    });
                    
                    if is_expected {
                        debug!("Shell command failed as expected with exit code: {}", exit_code);
                        
                        // Output error message if result logging is enabled
                        if tester.enable_result_log {
                            let mut error_output = if tester.expected_errors.len() == 1 && tester.expected_errors[0] != "0" {
                                format!("ERROR: Command failed with exit code {}\n", exit_code)
                            } else {
                                "Got one of the listed errors\n".to_string()
                            };
                            
                            // Apply regex replacements if any
                            tester.apply_regex_replacements(&mut error_output);
                            
                            if tester.args.record {
                                write!(tester.output_buffer, "{}", error_output)?;
                            } else {
                                tester.compare_with_result(&error_output)?;
                            }
                        }
                        return Ok(());
                    } else {
                        let err_msg = format!(
                            "Expected error(s) {:?}, but shell command failed with exit code: {}",
                            tester.expected_errors, exit_code
                        );
                        if tester.args.check_err {
                            return Err(anyhow!(err_msg));
                        } else {
                            warn!("{}", err_msg);
                        }
                    }
                } else {
                    // Command succeeded but we expected an error
                    let err_msg = format!(
                        "Expected error(s) {:?}, but shell command succeeded",
                        tester.expected_errors
                    );
                    if tester.args.check_err {
                        return Err(anyhow!(err_msg));
                    } else {
                        warn!("{}", err_msg);
                    }
                }
            } else if exit_code != 0 {
                // No expected errors, but command failed
                return Err(anyhow!(
                    "Shell command '{}' failed with exit code: {}",
                    expanded_command,
                    exit_code
                ));
            }
            
            // Handle successful execution output
            if tester.enable_result_log && !stdout_str.is_empty() {
                let mut output_str = stdout_str;
                
                // Apply regex replacements if any
                tester.apply_regex_replacements(&mut output_str);
                
                if tester.args.record {
                    write!(tester.output_buffer, "{}", output_str)?;
                } else {
                    tester.compare_with_result(&output_str)?;
                }
            }
            
            debug!("Shell command executed successfully with exit code: {}", exit_code);
            Ok(())
        }
        Err(e) => {
            Err(anyhow!("Failed to execute shell command '{}': {}", expanded_command, e))
        }
    }
} 