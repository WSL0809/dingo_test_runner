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
                    let is_expected = tester
                        .expected_errors
                        .iter()
                        .any(|expected| expected == &exit_code_str);

                    if is_expected {
                        debug!(
                            "Shell command failed as expected with exit code: {}",
                            exit_code
                        );

                        // Output error message if result logging is enabled
                        if tester.enable_result_log {
                            let mut error_output = if tester.expected_errors.len() == 1
                                && tester.expected_errors[0] != "0"
                            {
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
                // 未声明任何预期错误，但命令失败
                return Err(anyhow!(
                    "Shell command '{}' failed with exit code: {}",
                    expanded_command,
                    exit_code
                ));
            } else if !tester.expected_errors.is_empty()
                && !tester.expected_errors.iter().any(|e| e == "0")
            {
                // 命令成功，但期望某种错误（且列表中不包含 0）
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

            debug!(
                "Shell command executed successfully with exit code: {}",
                exit_code
            );
            Ok(())
        }
        Err(e) => Err(anyhow!(
            "Failed to execute shell command '{}': {}",
            expanded_command,
            e
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::Args;
    use crate::tester::command::Command;
    use crate::tester::tester::Tester;

    fn create_test_tester() -> Option<Tester> {
        let args = Args {
            host: "127.0.0.1".to_string(),
            port: "3306".to_string(),
            user: "root".to_string(),
            passwd: "123456".to_string(),
            log_level: "error".to_string(),
            record: true,
            params: "".to_string(),
            all: false,
            reserve_schema: false,
            xunit_file: "".to_string(),
            report_format: "terminal".to_string(),
            allure_dir: "".to_string(),
            retry_conn_count: 1, // Reduce retry count for tests
            check_err: false,
            collation_disable: false,
            extension: "result".to_string(),
            email_enable: false,
            email_smtp_host: "".to_string(),
            email_smtp_port: 587,
            email_username: "".to_string(),
            email_password: "".to_string(),
            email_from: "MySQL Tester".to_string(),
            email_to: "".to_string(),
            email_enable_tls: false,
            test_files: vec![],
            fail_fast: false,
        };

        match Tester::new(args) {
            Ok(tester) => Some(tester),
            Err(e) => {
                eprintln!("Skipping test due to DB connection error: {}. This test requires a running MySQL server.", e);
                None
            }
        }
    }

    #[test]
    fn test_exec_simple_command() {
        let mut tester = match create_test_tester() {
            Some(t) => t,
            None => return, // Skip test if no DB connection
        };

        let cmd = Command {
            name: "exec".to_string(),
            args: "echo 'Hello World'".to_string(),
            line: 1,
        };

        let result = execute(&mut tester, &cmd);
        assert!(result.is_ok(), "Exec command should succeed: {:?}", result);

        let output = String::from_utf8_lossy(&tester.output_buffer);
        assert!(
            output.contains("Hello World"),
            "Output should contain 'Hello World', got: {}",
            output
        );
    }

    #[test]
    fn test_exec_command_with_exit_code() {
        let mut tester = match create_test_tester() {
            Some(t) => t,
            None => return, // Skip test if no DB connection
        };

        // Set expected error for exit code 1
        tester.expected_errors = vec!["1".to_string()];

        let cmd = Command {
            name: "exec".to_string(),
            args: "exit 1".to_string(),
            line: 1,
        };

        let result = execute(&mut tester, &cmd);
        assert!(
            result.is_ok(),
            "Exec command with expected error should succeed: {:?}",
            result
        );

        let output = String::from_utf8_lossy(&tester.output_buffer);
        assert!(
            output.contains("ERROR: Command failed with exit code 1")
                || output.contains("Got one of the listed errors"),
            "Output should contain error message, got: {}",
            output
        );
    }

    #[test]
    fn test_exec_unexpected_failure() {
        let mut tester = match create_test_tester() {
            Some(t) => t,
            None => return, // Skip test if no DB connection
        };

        let cmd = Command {
            name: "exec".to_string(),
            args: "exit 1".to_string(),
            line: 1,
        };

        let result = execute(&mut tester, &cmd);
        assert!(
            result.is_err(),
            "Exec command with unexpected failure should fail"
        );

        let error_msg = result.unwrap_err().to_string();
        assert!(
            error_msg.contains("failed with exit code: 1"),
            "Error message should mention exit code, got: {}",
            error_msg
        );
    }

    #[test]
    fn test_exec_expected_success_but_failed() {
        let mut tester = match create_test_tester() {
            Some(t) => t,
            None => return, // Skip test if no DB connection
        };
        tester.args.check_err = true; // Enable strict error checking

        // Expect success (no error) but command will fail
        tester.expected_errors = vec!["0".to_string()];

        let cmd = Command {
            name: "exec".to_string(),
            args: "exit 1".to_string(),
            line: 1,
        };

        let result = execute(&mut tester, &cmd);
        assert!(
            result.is_err(),
            "Should fail when expected success but got failure"
        );
    }

    #[test]
    fn test_exec_expected_failure_but_succeeded() {
        let mut tester = match create_test_tester() {
            Some(t) => t,
            None => return, // Skip test if no DB connection
        };
        tester.args.check_err = true; // Enable strict error checking

        // Expect failure but command will succeed
        tester.expected_errors = vec!["1".to_string()];

        let cmd = Command {
            name: "exec".to_string(),
            args: "echo 'success'".to_string(),
            line: 1,
        };

        let result = execute(&mut tester, &cmd);
        assert!(
            result.is_err(),
            "Should fail when expected failure but got success"
        );
    }

    #[test]
    fn test_exec_with_result_log_disabled() {
        let mut tester = match create_test_tester() {
            Some(t) => t,
            None => return, // Skip test if no DB connection
        };
        tester.enable_result_log = false;

        let cmd = Command {
            name: "exec".to_string(),
            args: "echo 'This should not appear'".to_string(),
            line: 1,
        };

        let result = execute(&mut tester, &cmd);
        assert!(result.is_ok(), "Exec command should succeed: {:?}", result);

        let output = String::from_utf8_lossy(&tester.output_buffer);
        assert!(
            output.is_empty(),
            "Output should be empty when result log is disabled, got: {}",
            output
        );
    }

    #[test]
    fn test_exec_with_regex_replacement() {
        let mut tester = match create_test_tester() {
            Some(t) => t,
            None => return, // Skip test if no DB connection
        };

        // Add a regex replacement
        use regex::Regex;
        let regex = Regex::new(r"Hello").unwrap();
        tester.pending_replace_regex.push((regex, "Hi".to_string()));

        let cmd = Command {
            name: "exec".to_string(),
            args: "echo 'Hello World'".to_string(),
            line: 1,
        };

        let result = execute(&mut tester, &cmd);
        assert!(result.is_ok(), "Exec command should succeed: {:?}", result);

        let output = String::from_utf8_lossy(&tester.output_buffer);
        assert!(
            output.contains("Hi World"),
            "Output should contain replaced text 'Hi World', got: {}",
            output
        );
        assert!(
            !output.contains("Hello World"),
            "Output should not contain original text 'Hello World'"
        );
    }

    #[test]
    fn test_exec_empty_output() {
        let mut tester = match create_test_tester() {
            Some(t) => t,
            None => return, // Skip test if no DB connection
        };

        let cmd = Command {
            name: "exec".to_string(),
            args: "true".to_string(), // Command that succeeds but produces no output
            line: 1,
        };

        let result = execute(&mut tester, &cmd);
        assert!(result.is_ok(), "Exec command should succeed: {:?}", result);

        let output = String::from_utf8_lossy(&tester.output_buffer);
        assert!(
            output.is_empty(),
            "Output should be empty for command with no stdout, got: '{}'",
            output
        );
    }

    #[test]
    fn test_exec_multiline_output() {
        let mut tester = match create_test_tester() {
            Some(t) => t,
            None => return, // Skip test if no DB connection
        };

        let cmd = Command {
            name: "exec".to_string(),
            args: "printf 'Line 1\\nLine 2\\nLine 3\\n'".to_string(),
            line: 1,
        };

        let result = execute(&mut tester, &cmd);
        assert!(result.is_ok(), "Exec command should succeed: {:?}", result);

        let output = String::from_utf8_lossy(&tester.output_buffer);
        let lines: Vec<&str> = output.lines().collect();
        assert_eq!(
            lines.len(),
            3,
            "Should have 3 lines of output, got: {:?}",
            lines
        );
        assert_eq!(lines[0], "Line 1");
        assert_eq!(lines[1], "Line 2");
        assert_eq!(lines[2], "Line 3");
    }
}
