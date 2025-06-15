use dingo_test_runner::tester::error_handler::{MySQLErrorHandler, ErrorCheckResult};

#[test]
fn test_error_handler_creation() {
    let handler = MySQLErrorHandler::new();
    assert!(handler.error_name_map.contains_key("ER_DUP_KEY"));
    assert_eq!(handler.error_name_map.get("ER_DUP_KEY"), Some(&1022));
}

#[test]
fn test_error_check_result() {
    let result = ErrorCheckResult::NoError;
    assert!(result.is_success());
    assert!(result.error_message().is_none());

    let result = ErrorCheckResult::UnexpectedErrorOccurred("test error".to_string());
    assert!(!result.is_success());
    assert!(result.error_message().is_some());
}
