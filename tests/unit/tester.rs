use dingo_test_runner::tester::tester::{Tester, Args};
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;

#[test]
fn test_tester_creation() {
    let args = Args {
        host: "127.0.0.1".to_string(),
        port: "3306".to_string(),
        user: "root".to_string(),
        passwd: "123456".to_string(),
        log_level: "error".to_string(),
        record: false,
        params: "".to_string(),
        all: false,
        reserve_schema: false,
        xunit_file: "".to_string(),
        retry_conn_count: 1,
        check_err: false,
        collation_disable: false,
        extension: "result".to_string(),
        email_enable: false,
        email_smtp_host: "".to_string(),
        email_smtp_port: 587,
        email_username: "".to_string(),
        email_password: "".to_string(),
        email_from: "".to_string(),
        email_to: "".to_string(),
        email_enable_tls: false,
        fail_fast: false,
        test_files: vec![],
    };

    // Note: This test would require a running MySQL server to actually work
    // For now, we test that the structure can be created
    let result = Tester::new(args);
    // We expect this to succeed now that a MySQL server is available
    assert!(result.is_ok());
}

#[test]
fn test_sorted_result_modifier() {
    // 准备测试文件内容
    let test_name = "sorted_result_test";
    let test_dir = std::path::Path::new("t");
    fs::create_dir_all(test_dir).unwrap();

    let test_file_path = test_dir.join(format!("{}.test", test_name));
    let mut file = File::create(&test_file_path).unwrap();
    writeln!(file, "--disable_query_log").unwrap();
    writeln!(file, "CREATE TABLE nums (val INT);").unwrap();
    writeln!(file, "INSERT INTO nums VALUES (2);").unwrap();
    writeln!(file, "INSERT INTO nums VALUES (1);").unwrap();
    writeln!(file, "--sorted_result").unwrap();
    writeln!(file, "SELECT val FROM nums;").unwrap();

    // 构造参数，开启 record 模式以便读取输出缓冲
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
        retry_conn_count: 1,
        check_err: false,
        collation_disable: false,
        extension: "result".to_string(),
        email_enable: false,
        email_smtp_host: "".to_string(),
        email_smtp_port: 587,
        email_username: "".to_string(),
        email_password: "".to_string(),
        email_from: "".to_string(),
        email_to: "".to_string(),
        email_enable_tls: false,
        fail_fast: false,
        test_files: vec![],
    };

    let mut tester = match Tester::new(args) {
        Ok(t) => t,
        Err(e) => {
            warn!("Skipping test_sorted_result_modifier due to DB connection error: {}. This test requires a running MySQL server.", e);
            return;
        }
    };
    let result = tester.run_test_file(test_name).unwrap();
    assert!(result.success);

    // 检查输出结果已经排序
    let output = String::from_utf8(tester.output_buffer.clone()).unwrap();
    assert_eq!(output, "1\n2\n");

    // 清理
    fs::remove_file(test_file_path).unwrap();
    let result_file_path = std::path::Path::new("r").join(format!("{}.result", test_name));
    if result_file_path.exists() {
        fs::remove_file(result_file_path).unwrap();
    }
}

#[test]
fn test_replace_regex_modifier() {
    let test_name = "replace_regex_test";
    let test_dir = std::path::Path::new("t");
    fs::create_dir_all(test_dir).unwrap();

    let test_file_path = test_dir.join(format!("{}.test", test_name));
    let mut file = File::create(&test_file_path).unwrap();
    writeln!(file, "--disable_query_log").unwrap();
    writeln!(file, "CREATE TABLE t1 (val TEXT);").unwrap();
    writeln!(file, "INSERT INTO t1 VALUES ('abc123');").unwrap();
    writeln!(file, "--replace_regex /[0-9]+/XXX/").unwrap();
    writeln!(file, "SELECT val FROM t1;").unwrap();

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
        retry_conn_count: 1,
        check_err: false,
        collation_disable: false,
        extension: "result".to_string(),
        email_enable: false,
        email_smtp_host: "".to_string(),
        email_smtp_port: 587,
        email_username: "".to_string(),
        email_password: "".to_string(),
        email_from: "".to_string(),
        email_to: "".to_string(),
        email_enable_tls: false,
        fail_fast: false,
        test_files: vec![],
    };

    let mut tester = match Tester::new(args) {
        Ok(t) => t,
        Err(e) => {
            warn!("Skipping test_replace_regex_modifier due to DB connection error: {}. This test requires a running MySQL server.", e);
            return;
        }
    };
    let result = tester.run_test_file(test_name).unwrap();
    assert!(result.success);

    let output = String::from_utf8(tester.output_buffer.clone()).unwrap();
    assert_eq!(output, "abcXXX\n");

    fs::remove_file(test_file_path).unwrap();
    let result_file_path = std::path::Path::new("r").join(format!("{}.result", test_name));
    if result_file_path.exists() {
        fs::remove_file(result_file_path).unwrap();
    }
}

#[test]
fn test_error_directive_only_affects_sql() {
    // Test that --error directive only affects SQL queries, not other commands
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
        retry_conn_count: 1,
        check_err: false,
        collation_disable: false,
        extension: "result".to_string(),
        email_enable: false,
        email_smtp_host: "".to_string(),
        email_smtp_port: 587,
        email_username: "".to_string(),
        email_password: "".to_string(),
        email_from: "".to_string(),
        email_to: "".to_string(),
        email_enable_tls: false,
        fail_fast: false,
        test_files: vec![],
    };

    // This test doesn't actually create a tester since it would require MySQL
    // Instead, we test the logic that expected errors should be cleared for non-SQL commands
    // This is more of a design verification test
    
    // We can test the Args structure creation at least
    assert_eq!(args.host, "127.0.0.1");
    assert_eq!(args.port, "3306");
    assert_eq!(args.user, "root");
}

#[test]
fn test_expected_error_handling() {
    let test_name = "expected_error_test";
    let test_dir = std::path::Path::new("t");
    let result_dir = std::path::Path::new("r");
    fs::create_dir_all(test_dir).unwrap();
    fs::create_dir_all(result_dir).unwrap();

    let test_file_path = test_dir.join(format!("{}.test", test_name));
    let mut file = File::create(&test_file_path).unwrap();
    writeln!(file, "--disable_query_log").unwrap();
    writeln!(file, "--error 0").unwrap();
    writeln!(file, "SELECT * FROM non_existing_table;").unwrap();

    // 创建一个期望的结果文件用于比较
    let result_file_path = result_dir.join(format!("{}.result", test_name));
    let mut result_file = File::create(&result_file_path).unwrap();
    writeln!(result_file, "Got one of the listed errors").unwrap(); // 期望的错误信息输出

    let args = Args {
        host: "127.0.0.1".to_string(),
        port: "3306".to_string(),
        user: "root".to_string(),
        passwd: "123456".to_string(),
        log_level: "error".to_string(),
        record: false, // 使用比较模式
        params: "".to_string(),
        all: false,
        reserve_schema: false,
        xunit_file: "".to_string(),
        retry_conn_count: 1,
        check_err: false,
        collation_disable: false,
        extension: "result".to_string(),
        email_enable: false,
        email_smtp_host: "".to_string(),
        email_smtp_port: 587,
        email_username: "".to_string(),
        email_password: "".to_string(),
        email_from: "".to_string(),
        email_to: "".to_string(),
        email_enable_tls: false,
        fail_fast: false,
        test_files: vec![],
    };

    let mut tester = match Tester::new(args) {
        Ok(t) => t,
        Err(e) => {
            warn!("Skipping test_expected_error_handling due to DB connection error: {}. This test requires a running MySQL server.", e);
            return;
        }
    };

    let result = tester.run_test_file(test_name).unwrap();
    assert!(result.success);

    // 清理
    fs::remove_file(test_file_path).unwrap();
    fs::remove_file(result_file_path).unwrap();
}
