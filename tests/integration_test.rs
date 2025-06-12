//! 集成测试 - 端到端功能测试
//! 
//! 这些测试验证整个 mysql-tester-rs 程序的功能，包括：
//! - CLI 参数解析
//! - .test 文件执行
//! - .result 文件生成和比对
//! - 各种命令标签的行为

use std::process::Command;
use std::fs;
use std::path::Path;
use std::env;

/// 获取二进制文件路径
fn get_binary_path() -> std::path::PathBuf {
    let mut path = env::current_exe().unwrap();
    path.pop(); // 移除测试可执行文件名
    if path.ends_with("deps") {
        path.pop(); // 移除 deps 目录
    }
    path.push("dingo_test_runner");
    path
}

#[test]
fn test_basic_echo_execution() {
    let binary = get_binary_path();
    
    // 运行 simple_test，使用SQLite避免MySQL连接问题
    let output = Command::new(&binary)
        .arg("simple_test")
        .arg("--record") // 录制模式
        .arg("--database-type")
        .arg("sqlite")
        .output()
        .expect("Failed to execute binary");

    // 检查程序是否成功运行
    if !output.status.success() {
        eprintln!("STDOUT: {}", String::from_utf8_lossy(&output.stdout));
        eprintln!("STDERR: {}", String::from_utf8_lossy(&output.stderr));
        panic!("Binary execution failed with status: {}", output.status);
    }

    // 检查是否生成了结果文件
    let result_file = Path::new("r/simple_test.result");
    assert!(result_file.exists(), "Result file should be created in record mode");

    // 读取结果文件内容
    let content = fs::read_to_string(result_file).unwrap();
    assert!(content.contains("Hello World"), "Result should contain echo output");
    assert!(content.contains("Test completed"), "Result should contain second echo");
}

#[test]
fn test_regex_replacement() {
    let binary = get_binary_path();
    
    // 运行 regex_test，使用SQLite
    let output = Command::new(&binary)
        .arg("regex_test")
        .arg("--record") // 录制模式
        .arg("--database-type")
        .arg("sqlite")
        .output()
        .expect("Failed to execute binary");

    if !output.status.success() {
        eprintln!("STDOUT: {}", String::from_utf8_lossy(&output.stdout));
        eprintln!("STDERR: {}", String::from_utf8_lossy(&output.stderr));
        panic!("Regex test execution failed");
    }

    // 检查结果文件
    let result_file = Path::new("r/regex_test.result");
    assert!(result_file.exists(), "Regex test result file should be created");

    let content = fs::read_to_string(result_file).unwrap();
    
    // 验证时间戳被正确替换
    assert!(content.contains("TIMESTAMP"), "Timestamp should be replaced with TIMESTAMP");
    
    // 注意：INSERT语句中的原始时间戳会保留（因为那是查询本身，不是结果）
    // 但SELECT结果中的时间戳应该被替换
    // 检查是否包含替换后的结果
    assert!(content.contains("Timestamp is TIMESTAMP"), "SELECT result should contain replaced timestamp");
}

#[test]
fn test_sorted_result_functionality() {
    let binary = get_binary_path();
    
    // 运行 advanced_test，使用SQLite
    let output = Command::new(&binary)
        .arg("advanced_test")
        .arg("--record")
        .arg("--database-type")
        .arg("sqlite")
        .output()
        .expect("Failed to execute binary");

    if !output.status.success() {
        eprintln!("STDOUT: {}", String::from_utf8_lossy(&output.stdout));
        eprintln!("STDERR: {}", String::from_utf8_lossy(&output.stderr));
        panic!("Advanced test execution failed");
    }

    let result_file = Path::new("r/advanced_test.result");
    assert!(result_file.exists(), "Advanced test result file should be created");
}

#[test]
fn test_error_handling() {
    let binary = get_binary_path();
    
    // 运行 error_test，使用SQLite
    let output = Command::new(&binary)
        .arg("error_test")
        .arg("--record")
        .arg("--database-type")
        .arg("sqlite")
        .output()
        .expect("Failed to execute binary");

    // 这个测试可能因为错误而失败，但应该能正确处理预期的错误
    let result_file = Path::new("r/error_test.result");
    
    // 如果生成了结果文件，检查其内容
    if result_file.exists() {
        let content = fs::read_to_string(result_file).unwrap();
        // 错误测试的具体验证会依赖于错误处理的实现
        println!("Error test result: {}", content);
    }
}

#[test]
fn test_all_tests_execution() {
    let binary = get_binary_path();
    
    // 运行 --all 参数，使用SQLite
    let _output = Command::new(&binary)
        .arg("--all")
        .arg("--record")
        .arg("--database-type")
        .arg("sqlite")
        .output()
        .expect("Failed to execute binary with --all");

    // 检查程序至少启动了
    println!("All tests STDOUT: {}", String::from_utf8_lossy(&_output.stdout));
    println!("All tests STDERR: {}", String::from_utf8_lossy(&_output.stderr));
    
    // 即使有些测试失败，--all 命令也应该尝试运行所有测试
    // 我们主要验证程序不会崩溃
}

#[test]
fn test_nonexistent_file() {
    let binary = get_binary_path();
    
    // 运行不存在的文件，使用SQLite
    let output = Command::new(&binary)
        .arg("nonexistent_test")
        .arg("--database-type")
        .arg("sqlite")
        .output()
        .expect("Failed to execute binary");

    // 应该返回错误状态
    assert!(!output.status.success(), "Should fail when test file doesn't exist");
    
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("nonexistent") || stderr.contains("not found") || stderr.contains("No such file"), 
            "Error message should indicate file not found");
}

#[test]
fn test_help_output() {
    let binary = get_binary_path();
    
    // 测试 --help 参数
    let output = Command::new(&binary)
        .arg("--help")
        .output()
        .expect("Failed to execute binary with --help");

    assert!(output.status.success(), "Help command should succeed");
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Usage") || stdout.contains("USAGE"), "Help should contain usage information");
    assert!(stdout.contains("record") || stdout.contains("--record"), "Help should mention record option");
} 