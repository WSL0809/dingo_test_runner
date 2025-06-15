//! Unit tests for the exec command handler.

use dingo_test_runner::tester::command::Command;
use dingo_test_runner::tester::handlers::exec;
use dingo_test_runner::tester::tester::Tester;
use dingo_test_runner::cli::Args;
use std::io::Write;

fn create_test_tester() -> Tester {
    let args = Args {
        host: "127.0.0.1".to_string(),
        port: "3306".to_string(),
        user: "root".to_string(),
        passwd: "".to_string(),
        log_level: "error".to_string(),
        record: true,
        params: "".to_string(),
        all: false,
        reserve_schema: false,
        xunit_file: "".to_string(),
        retry_conn_count: 3,
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
    
    Tester::new(args).expect("Failed to create test tester")
}

#[test]
fn test_exec_simple_command() {
    let mut tester = create_test_tester();
    
    let cmd = Command {
        name: "exec".to_string(),
        args: "echo 'Hello World'".to_string(),
        line: 1,
    };
    
    let result = exec::execute(&mut tester, &cmd);
    assert!(result.is_ok(), "Exec command should succeed: {:?}", result);
    
    let output = String::from_utf8_lossy(&tester.output_buffer);
    assert!(output.contains("Hello World"), "Output should contain 'Hello World', got: {}", output);
}

#[test]
fn test_exec_command_with_exit_code() {
    let mut tester = create_test_tester();
    
    // Set expected error for exit code 1
    tester.expected_errors = vec!["1".to_string()];
    
    let cmd = Command {
        name: "exec".to_string(),
        args: "exit 1".to_string(),
        line: 1,
    };
    
    let result = exec::execute(&mut tester, &cmd);
    assert!(result.is_ok(), "Exec command with expected error should succeed: {:?}", result);
    
    let output = String::from_utf8_lossy(&tester.output_buffer);
    assert!(output.contains("ERROR: Command failed with exit code 1") || 
            output.contains("Got one of the listed errors"), 
            "Output should contain error message, got: {}", output);
}

#[test]
fn test_exec_unexpected_failure() {
    let mut tester = create_test_tester();
    
    let cmd = Command {
        name: "exec".to_string(),
        args: "exit 1".to_string(),
        line: 1,
    };
    
    let result = exec::execute(&mut tester, &cmd);
    assert!(result.is_err(), "Exec command with unexpected failure should fail");
    
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("failed with exit code: 1"), 
            "Error message should mention exit code, got: {}", error_msg);
}

#[test]
fn test_exec_expected_success_but_failed() {
    let mut tester = create_test_tester();
    tester.args.check_err = true; // Enable strict error checking
    
    // Expect success (no error) but command will fail
    tester.expected_errors = vec!["0".to_string()];
    
    let cmd = Command {
        name: "exec".to_string(),
        args: "exit 1".to_string(),
        line: 1,
    };
    
    let result = exec::execute(&mut tester, &cmd);
    assert!(result.is_err(), "Should fail when expected success but got failure");
}

#[test]
fn test_exec_expected_failure_but_succeeded() {
    let mut tester = create_test_tester();
    tester.args.check_err = true; // Enable strict error checking
    
    // Expect failure but command will succeed
    tester.expected_errors = vec!["1".to_string()];
    
    let cmd = Command {
        name: "exec".to_string(),
        args: "echo 'success'".to_string(),
        line: 1,
    };
    
    let result = exec::execute(&mut tester, &cmd);
    assert!(result.is_err(), "Should fail when expected failure but got success");
}

#[test]
fn test_exec_with_result_log_disabled() {
    let mut tester = create_test_tester();
    tester.enable_result_log = false;
    
    let cmd = Command {
        name: "exec".to_string(),
        args: "echo 'This should not appear'".to_string(),
        line: 1,
    };
    
    let result = exec::execute(&mut tester, &cmd);
    assert!(result.is_ok(), "Exec command should succeed: {:?}", result);
    
    let output = String::from_utf8_lossy(&tester.output_buffer);
    assert!(output.is_empty(), "Output should be empty when result log is disabled, got: {}", output);
}

#[test]
fn test_exec_with_regex_replacement() {
    let mut tester = create_test_tester();
    
    // Add a regex replacement
    use regex::Regex;
    let regex = Regex::new(r"Hello").unwrap();
    tester.pending_replace_regex.push((regex, "Hi".to_string()));
    
    let cmd = Command {
        name: "exec".to_string(),
        args: "echo 'Hello World'".to_string(),
        line: 1,
    };
    
    let result = exec::execute(&mut tester, &cmd);
    assert!(result.is_ok(), "Exec command should succeed: {:?}", result);
    
    let output = String::from_utf8_lossy(&tester.output_buffer);
    assert!(output.contains("Hi World"), "Output should contain replaced text 'Hi World', got: {}", output);
    assert!(!output.contains("Hello World"), "Output should not contain original text 'Hello World'");
}

#[test]
fn test_exec_empty_output() {
    let mut tester = create_test_tester();
    
    let cmd = Command {
        name: "exec".to_string(),
        args: "true".to_string(), // Command that succeeds but produces no output
        line: 1,
    };
    
    let result = exec::execute(&mut tester, &cmd);
    assert!(result.is_ok(), "Exec command should succeed: {:?}", result);
    
    let output = String::from_utf8_lossy(&tester.output_buffer);
    assert!(output.is_empty(), "Output should be empty for command with no stdout, got: '{}'", output);
}

#[test]
fn test_exec_multiline_output() {
    let mut tester = create_test_tester();
    
    let cmd = Command {
        name: "exec".to_string(),
        args: "printf 'Line 1\\nLine 2\\nLine 3\\n'".to_string(),
        line: 1,
    };
    
    let result = exec::execute(&mut tester, &cmd);
    assert!(result.is_ok(), "Exec command should succeed: {:?}", result);
    
    let output = String::from_utf8_lossy(&tester.output_buffer);
    let lines: Vec<&str> = output.lines().collect();
    assert_eq!(lines.len(), 3, "Should have 3 lines of output, got: {:?}", lines);
    assert_eq!(lines[0], "Line 1");
    assert_eq!(lines[1], "Line 2");
    assert_eq!(lines[2], "Line 3");
} 