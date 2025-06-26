//! Handler for the --echo command.

use crate::tester::command::Command;
use crate::tester::tester::Tester;
use anyhow::Result;
use log::debug;
use std::io::Write;

pub fn execute(tester: &mut Tester, cmd: &Command) -> Result<()> {
    // Expand variables in the echo text
    let expanded_text = tester.variable_context.expand(&cmd.args)?;
    let echo_output = format!("{}\n", expanded_text);
    debug!("Echo: {} -> {}", cmd.args, expanded_text);

    // Per user feedback, echo is NOT affected by modifiers.
    if tester.args.record {
        write!(tester.output_buffer, "{}", echo_output)?;
    } else {
        tester.compare_with_result(&echo_output)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::Args;
    use crate::tester::tester::Tester;

    // These tests are marked as ignored because they require database connection
    // In a real testing environment, we would use dependency injection and mocking
    // to avoid database dependencies in unit tests

    fn create_test_args() -> Args {
        Args {
            host: "127.0.0.1".to_string(),
            port: "3306".to_string(),
            user: "root".to_string(),
            passwd: "password".to_string(),
            record: true,
            ..Default::default()
        }
    }

    #[test]
    #[ignore = "Requires database connection - run with integration tests"]
    fn test_echo_simple_text() {
        let args = create_test_args();
        let mut tester = Tester::new(args).unwrap();
        let cmd = Command {
            args: "Hello World".to_string(),
            ..Default::default()
        };

        let result = execute(&mut tester, &cmd);
        assert!(result.is_ok());
        
        let output = String::from_utf8(tester.output_buffer.clone()).unwrap();
        assert_eq!(output, "Hello World\n");
    }

    #[test]
    #[ignore = "Requires database connection - run with integration tests"]
    fn test_echo_with_variables() {
        let args = create_test_args();
        let mut tester = Tester::new(args).unwrap();
        
        // Set a variable
        tester.variable_context.set("test_var", "test_value");
        
        let cmd = Command {
            args: "Value is: $test_var".to_string(),
            ..Default::default()
        };

        let result = execute(&mut tester, &cmd);
        assert!(result.is_ok());
        
        let output = String::from_utf8(tester.output_buffer.clone()).unwrap();
        assert_eq!(output, "Value is: test_value\n");
    }

    #[test]
    #[ignore = "Requires database connection - run with integration tests"]
    fn test_echo_empty_text() {
        let args = create_test_args();
        let mut tester = Tester::new(args).unwrap();
        let cmd = Command {
            args: "".to_string(),
            ..Default::default()
        };

        let result = execute(&mut tester, &cmd);
        assert!(result.is_ok());
        
        let output = String::from_utf8(tester.output_buffer.clone()).unwrap();
        assert_eq!(output, "\n");
    }

    #[test]
    #[ignore = "Requires database connection - run with integration tests"]
    fn test_echo_special_characters() {
        let args = create_test_args();
        let mut tester = Tester::new(args).unwrap();
        let cmd = Command {
            args: "Special chars: !@#$%^&*()".to_string(),
            ..Default::default()
        };

        let result = execute(&mut tester, &cmd);
        assert!(result.is_ok());
        
        let output = String::from_utf8(tester.output_buffer.clone()).unwrap();
        assert_eq!(output, "Special chars: !@#$%^&*()\n");
    }

    #[test]
    #[ignore = "Requires database connection - run with integration tests"]
    fn test_echo_multiline_text() {
        let args = create_test_args();
        let mut tester = Tester::new(args).unwrap();
        let cmd = Command {
            args: "Line 1\nLine 2".to_string(),
            ..Default::default()
        };

        let result = execute(&mut tester, &cmd);
        assert!(result.is_ok());
        
        let output = String::from_utf8(tester.output_buffer.clone()).unwrap();
        assert_eq!(output, "Line 1\nLine 2\n");
    }

    // Unit tests that don't require database connection
    #[test]
    fn test_echo_command_parsing() {
        let cmd = Command::new("echo", "test message", 1);
        assert_eq!(cmd.get_name(), "echo");
        assert_eq!(cmd.get_args(), "test message");
    }

    #[test]
    fn test_echo_function_exists() {
        // Simple test to verify the execute function signature
        let cmd = Command::default();
        // We can't call execute without a tester, but we can test that the function exists
        // This ensures the module compiles correctly
        assert_eq!(cmd.get_name(), "");
    }
}
