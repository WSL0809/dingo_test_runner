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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::Args;
    use crate::tester::tester::Tester;

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
    fn test_execute_sets_pending_sorted_result() {
        let args = create_test_args();
        let mut tester = Tester::new(args).unwrap();
        let cmd = Command::default();
        
        // Initially should be false
        assert!(!tester.pending_sorted_result);
        
        let result = execute(&mut tester, &cmd);
        assert!(result.is_ok());
        assert!(tester.pending_sorted_result);
    }

    #[test]
    #[ignore = "Requires database connection - run with integration tests"]
    fn test_execute_multiple_calls() {
        let args = create_test_args();
        let mut tester = Tester::new(args).unwrap();
        let cmd = Command::default();
        
        // First call
        let result = execute(&mut tester, &cmd);
        assert!(result.is_ok());
        assert!(tester.pending_sorted_result);
        
        // Second call should still work
        let result = execute(&mut tester, &cmd);
        assert!(result.is_ok());
        assert!(tester.pending_sorted_result);
    }

    #[test]
    #[ignore = "Requires database connection - run with integration tests"]
    fn test_execute_with_different_tester_states() {
        let args = create_test_args();
        let mut tester = Tester::new(args).unwrap();
        let cmd = Command::default();
        
        // Set some other state
        tester.enable_query_log = true;
        tester.enable_result_log = true;
        
        let result = execute(&mut tester, &cmd);
        assert!(result.is_ok());
        assert!(tester.pending_sorted_result);
        
        // Other states should remain unchanged
        assert!(tester.enable_query_log);
        assert!(tester.enable_result_log);
    }
}
