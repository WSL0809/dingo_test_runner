use dingo_test_runner::util::error_utils::{OptionExt, SafeParse};

#[test]
fn test_option_ext() {
    let some_value: Option<i32> = Some(42);
    let result = some_value.or_context("Expected value");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 42);
    
    let none_value: Option<i32> = None;
    let result = none_value.or_context("Value was None");
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Value was None"));
}

#[test]
fn test_safe_parse() {
    let result = SafeParse::parse_u16("123", "port");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 123);
    
    let result = SafeParse::parse_u16("abc", "port");
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Failed to parse port"));
}
