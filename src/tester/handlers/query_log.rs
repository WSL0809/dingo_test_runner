//! Handlers for query log control commands.

use crate::tester::command::Command;
use crate::tester::tester::Tester;
use anyhow::Result;
use log::debug;

pub fn disable_query_log(tester: &mut Tester, _cmd: &Command) -> Result<()> {
    tester.enable_query_log = false;
    debug!("Query log disabled");
    Ok(())
}

pub fn enable_query_log(tester: &mut Tester, _cmd: &Command) -> Result<()> {
    tester.enable_query_log = true;
    debug!("Query log enabled");
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
    fn test_enable_query_log() {
        let args = create_test_args();
        let mut tester = Tester::new(args).unwrap();
        let cmd = Command::default();
        
        // Initially should be false (default)
        assert!(!tester.enable_query_log);
        
        let result = enable_query_log(&mut tester, &cmd);
        assert!(result.is_ok());
        assert!(tester.enable_query_log);
    }

    #[test]
    #[ignore = "Requires database connection - run with integration tests"]
    fn test_disable_query_log() {
        let args = create_test_args();
        let mut tester = Tester::new(args).unwrap();
        let cmd = Command::default();
        
        // First enable it
        tester.enable_query_log = true;
        assert!(tester.enable_query_log);
        
        let result = disable_query_log(&mut tester, &cmd);
        assert!(result.is_ok());
        assert!(!tester.enable_query_log);
    }

    #[test]
    #[ignore = "Requires database connection - run with integration tests"]
    fn test_toggle_query_log() {
        let args = create_test_args();
        let mut tester = Tester::new(args).unwrap();
        let cmd = Command::default();
        
        // Start disabled
        assert!(!tester.enable_query_log);
        
        // Enable
        let result = enable_query_log(&mut tester, &cmd);
        assert!(result.is_ok());
        assert!(tester.enable_query_log);
        
        // Disable
        let result = disable_query_log(&mut tester, &cmd);
        assert!(result.is_ok());
        assert!(!tester.enable_query_log);
        
        // Enable again
        let result = enable_query_log(&mut tester, &cmd);
        assert!(result.is_ok());
        assert!(tester.enable_query_log);
    }
}
