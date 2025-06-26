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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::Args;
    use crate::tester::tester::Tester;
    use std::time::Instant;

    fn create_test_args() -> Args {
        Args {
            host: "127.0.0.1".to_string(),
            port: "3306".to_string(),
            user: "root".to_string(),
            passwd: "password".to_string(),
            ..Default::default()
        }
    }

    #[test]
    #[ignore = "Requires database connection - run with integration tests"]
    fn test_sleep_valid_duration() {
        let args = create_test_args();
        let mut tester = Tester::new(args).unwrap();
        let cmd = Command {
            args: "0.1".to_string(),
            ..Default::default()
        };

        let start = Instant::now();
        let result = execute(&mut tester, &cmd);
        let elapsed = start.elapsed();

        assert!(result.is_ok());
        assert!(elapsed.as_millis() >= 100); // Should sleep at least 100ms
        assert!(elapsed.as_millis() < 200); // But not too much longer
    }

    #[test]
    #[ignore = "Requires database connection - run with integration tests"]
    fn test_sleep_zero_duration() {
        let args = create_test_args();
        let mut tester = Tester::new(args).unwrap();
        let cmd = Command {
            args: "0".to_string(),
            ..Default::default()
        };

        let result = execute(&mut tester, &cmd);
        assert!(result.is_ok());
    }

    #[test]
    #[ignore = "Requires database connection - run with integration tests"]
    fn test_sleep_invalid_duration() {
        let args = create_test_args();
        let mut tester = Tester::new(args).unwrap();
        let cmd = Command {
            args: "invalid".to_string(),
            ..Default::default()
        };

        let result = execute(&mut tester, &cmd);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid sleep duration"));
    }

    #[test]
    #[ignore = "Requires database connection - run with integration tests"]
    fn test_sleep_negative_duration() {
        let args = create_test_args();
        let mut tester = Tester::new(args).unwrap();
        let cmd = Command {
            args: "-1".to_string(),
            ..Default::default()
        };

        // Negative duration should be accepted by f64::parse but handled by Duration
        let result = execute(&mut tester, &cmd);
        // std::time::Duration::from_secs_f64 will panic on negative values
        // but our code should handle it gracefully
        assert!(result.is_err() || result.is_ok()); // Depends on implementation
    }

    #[test]
    #[ignore = "Requires database connection - run with integration tests"]
    fn test_sleep_empty_args() {
        let args = create_test_args();
        let mut tester = Tester::new(args).unwrap();
        let cmd = Command {
            args: "".to_string(),
            ..Default::default()
        };

        let result = execute(&mut tester, &cmd);
        assert!(result.is_err());
    }
}
