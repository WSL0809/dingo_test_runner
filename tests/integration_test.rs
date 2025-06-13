//! 集成测试 - 端到端功能测试
//! 
//! 这些测试验证整个 mysql-tester-rs 程序的功能，包括：
//! - CLI 参数解析
//! - .test 文件执行
//! - .result 文件生成和比对
//! - 各种命令标签的行为

use std::process::Command;
use std::path::Path;
use std::fs;

fn get_binary_path() -> String {
    // 在 CI 环境中，二进制文件可能在不同的位置
    if Path::new("target/release/dingo_test_runner").exists() {
        "target/release/dingo_test_runner".to_string()
    } else if Path::new("target/debug/dingo_test_runner").exists() {
        "target/debug/dingo_test_runner".to_string()
    } else {
        // 如果都不存在，尝试构建
        let build_output = Command::new("cargo")
            .args(&["build", "--bin", "dingo_test_runner"])
            .output()
            .expect("Failed to build binary");
        
        if !build_output.status.success() {
            panic!("Failed to build binary: {}", String::from_utf8_lossy(&build_output.stderr));
        }
        
        "target/debug/dingo_test_runner".to_string()
    }
}

#[test]
#[ignore] // Requires MySQL server
fn test_simple_execution() {
    let binary = get_binary_path();
    
    // 运行 simple_test，需要MySQL服务器
    let output = Command::new(&binary)
        .arg("simple_test")
        .arg("--record")
        .output()
        .expect("Failed to execute binary");

    // 检查程序至少启动了
    println!("STDOUT: {}", String::from_utf8_lossy(&output.stdout));
    println!("STDERR: {}", String::from_utf8_lossy(&output.stderr));
    
    // 在没有MySQL服务器的情况下，程序应该会失败但不会崩溃
    // 我们主要验证程序能正常启动和处理错误
}

#[test]
fn test_help_output() {
    let binary = get_binary_path();
    
    // 测试 --help 参数
    let output = Command::new(&binary)
        .arg("--help")
        .output()
        .expect("Failed to execute binary");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("A MySQL testing framework"));
    assert!(stdout.contains("--host"));
    assert!(stdout.contains("--port"));
    assert!(stdout.contains("--user"));
}

#[test]
fn test_version_output() {
    let binary = get_binary_path();
    
    // 测试 --version 参数
    let output = Command::new(&binary)
        .arg("--version")
        .output()
        .expect("Failed to execute binary");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("0.2.0"));
}

#[test]
#[ignore] // Requires MySQL server
fn test_regex_replacement() {
    let binary = get_binary_path();
    
    // 运行 regex_test，需要MySQL服务器
    let output = Command::new(&binary)
        .arg("regex_test")
        .arg("--record")
        .output()
        .expect("Failed to execute binary");

    // 检查程序至少启动了
    println!("Regex test STDOUT: {}", String::from_utf8_lossy(&output.stdout));
    println!("Regex test STDERR: {}", String::from_utf8_lossy(&output.stderr));
}

#[test]
#[ignore] // Requires MySQL server
fn test_advanced_features() {
    let binary = get_binary_path();
    
    // 运行 advanced_test，需要MySQL服务器
    let output = Command::new(&binary)
        .arg("advanced_test")
        .arg("--record")
        .output()
        .expect("Failed to execute binary");

    // 检查程序至少启动了
    println!("Advanced test STDOUT: {}", String::from_utf8_lossy(&output.stdout));
    println!("Advanced test STDERR: {}", String::from_utf8_lossy(&output.stderr));
}

#[test]
#[ignore] // Requires MySQL server
fn test_error_handling() {
    let binary = get_binary_path();
    
    // 运行 error_test，需要MySQL服务器
    let _output = Command::new(&binary)
        .arg("error_test")
        .arg("--record")
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
#[ignore] // Requires MySQL server
fn test_all_tests_execution() {
    let binary = get_binary_path();
    
    // 运行 --all 参数，需要MySQL服务器
    let _output = Command::new(&binary)
        .arg("--all")
        .arg("--record")
        .output()
        .expect("Failed to execute binary with --all");

    // 检查程序至少启动了
    println!("All tests STDOUT: {}", String::from_utf8_lossy(&_output.stdout));
    println!("All tests STDERR: {}", String::from_utf8_lossy(&_output.stderr));
    
    // 即使有些测试失败，--all 命令也应该尝试运行所有测试
    // 我们主要验证程序不会崩溃
}

#[test]
fn test_invalid_arguments() {
    let binary = get_binary_path();
    
    // 测试无效的参数
    let output = Command::new(&binary)
        .arg("--invalid-arg")
        .output()
        .expect("Failed to execute binary");

    // 应该返回错误状态码
    assert!(!output.status.success());
    
    let stderr = String::from_utf8_lossy(&output.stderr);
    // 应该包含错误信息
    assert!(!stderr.is_empty());
}

#[test]
fn test_missing_test_files() {
    let binary = get_binary_path();
    
    // 运行不存在的文件，应该返回错误
    let output = Command::new(&binary)
        .arg("non_existent_test")
        .output()
        .expect("Failed to execute binary");

    // 应该返回错误状态码
    assert!(!output.status.success());
    
    println!("Missing file STDOUT: {}", String::from_utf8_lossy(&output.stdout));
    println!("Missing file STDERR: {}", String::from_utf8_lossy(&output.stderr));
} 