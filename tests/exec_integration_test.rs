//! Integration tests for the exec command functionality

use std::fs;
use std::path::Path;
use std::process::Command;

fn get_binary_path() -> String {
    if Path::new("target/release/dingo_test_runner").exists() {
        "target/release/dingo_test_runner".to_string()
    } else if Path::new("target/debug/dingo_test_runner").exists() {
        "target/debug/dingo_test_runner".to_string()
    } else {
        let build_output = Command::new("cargo")
            .args(&["build", "--bin", "dingo_test_runner"])
            .output()
            .expect("Failed to build binary");

        if !build_output.status.success() {
            panic!(
                "Failed to build binary: {}",
                String::from_utf8_lossy(&build_output.stderr)
            );
        }

        "target/debug/dingo_test_runner".to_string()
    }
}

#[test]
fn test_exec_simple_command() {
    let binary = get_binary_path();

    // Run simple_exec test with MySQL password using test extension
    let test_id = std::process::id();
    let extension = format!("test_{}", test_id);
    let output = Command::new(&binary)
        .arg("--record")
        .arg("--extension")
        .arg(&extension)
        .arg("t_for_test/basic/simple_exec.test")
        .arg("--host")
        .arg("127.0.0.1")
        .arg("--port")
        .arg("3306")
        .arg("--user")
        .arg("root")
        .arg("--passwd")
        .arg("123456")
        .output()
        .expect("Failed to execute binary");

    println!("STDOUT: {}", String::from_utf8_lossy(&output.stdout));
    println!("STDERR: {}", String::from_utf8_lossy(&output.stderr));

    // Should succeed
    assert!(output.status.success(), "simple_exec test should succeed");

    // Check if result file was created
    let result_file_path = format!("r/simple_exec.{}", extension);
    let result_file = Path::new(&result_file_path);
    assert!(result_file.exists(), "Result file should be created");

    // Check result file content
    let content = fs::read_to_string(result_file).expect("Failed to read result file");
    assert!(
        content.contains("Testing exec command"),
        "Should contain echo output"
    );
    assert!(
        content.contains("Hello World"),
        "Should contain exec output"
    );
    assert!(
        content.contains("Exec test completed"),
        "Should contain final echo"
    );
}

#[test]
fn test_exec_comparison_mode() {
    let binary = get_binary_path();

    // First ensure we have a result file by running record mode
    let record_output = Command::new(&binary)
        .arg("--record")
        .arg("t_for_test/basic/simple_exec.test")
        .arg("--host")
        .arg("127.0.0.1")
        .arg("--port")
        .arg("3306")
        .arg("--user")
        .arg("root")
        .arg("--passwd")
        .arg("123456")
        .output()
        .expect("Failed to execute binary in record mode");

    println!(
        "Record STDOUT: {}",
        String::from_utf8_lossy(&record_output.stdout)
    );
    println!(
        "Record STDERR: {}",
        String::from_utf8_lossy(&record_output.stderr)
    );

    if !record_output.status.success() {
        // If record mode fails, skip this test
        println!("Skipping comparison test because record mode failed");
        return;
    }

    // Now run in comparison mode
    let compare_output = Command::new(&binary)
        .arg("t_for_test/basic/simple_exec.test")
        .arg("--host")
        .arg("127.0.0.1")
        .arg("--port")
        .arg("3306")
        .arg("--user")
        .arg("root")
        .arg("--passwd")
        .arg("123456")
        .output()
        .expect("Failed to execute binary in comparison mode");

    println!(
        "Comparison STDOUT: {}",
        String::from_utf8_lossy(&compare_output.stdout)
    );
    println!(
        "Comparison STDERR: {}",
        String::from_utf8_lossy(&compare_output.stderr)
    );

    // Should succeed (output matches expected result)
    assert!(
        compare_output.status.success(),
        "Comparison mode should succeed"
    );
}

#[test]
fn test_exec_complex_commands() {
    let binary = get_binary_path();

    // Run exec_test which includes error handling and multiline output
    let output = Command::new(&binary)
        .arg("--record")
        .arg("t_for_test/advanced/exec_test.test")
        .arg("--host")
        .arg("127.0.0.1")
        .arg("--port")
        .arg("3306")
        .arg("--user")
        .arg("root")
        .arg("--passwd")
        .arg("123456")
        .output()
        .expect("Failed to execute binary");

    println!(
        "Complex exec STDOUT: {}",
        String::from_utf8_lossy(&output.stdout)
    );
    println!(
        "Complex exec STDERR: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Should succeed
    assert!(output.status.success(), "exec_test should succeed");

    // Check if result file was created
    let result_file = Path::new("r/exec_test.result");
    assert!(result_file.exists(), "Result file should be created");

    // Check result file content
    let content = fs::read_to_string(result_file).expect("Failed to read result file");
    assert!(
        content.contains("Starting exec command tests"),
        "Should contain start message"
    );
    assert!(
        content.contains("Hello from exec"),
        "Should contain simple exec output"
    );
    assert!(
        content.contains("Success test"),
        "Should contain success test output"
    );
    assert!(
        content.contains("ERROR: Command failed with exit code 1"),
        "Should contain expected error"
    );
    assert!(
        content.contains("Line 1"),
        "Should contain multiline output"
    );
    assert!(
        content.contains("Line 2"),
        "Should contain multiline output"
    );
    assert!(
        content.contains("Line 3"),
        "Should contain multiline output"
    );
    assert!(
        content.contains("Exec command tests completed"),
        "Should contain end message"
    );
}

#[test]
fn test_exec_error_handling() {
    let binary = get_binary_path();

    // Create a test file that should fail due to unexpected error
    let test_content = r#"
# Test exec with unexpected error
--echo Testing exec error handling
--exec exit 1
--echo This should not be reached
"#;

    fs::create_dir_all("t_for_test/temp").expect("Failed to create temp directory");
    fs::write("t_for_test/temp/exec_error_test.test", test_content).expect("Failed to write test file");

    // Run the test - it should fail
    let output = Command::new(&binary)
        .arg("--record")
        .arg("t_for_test/temp/exec_error_test.test")
        .arg("--host")
        .arg("127.0.0.1")
        .arg("--port")
        .arg("3306")
        .arg("--user")
        .arg("root")
        .arg("--passwd")
        .arg("123456")
        .output()
        .expect("Failed to execute binary");

    println!(
        "Error test STDOUT: {}",
        String::from_utf8_lossy(&output.stdout)
    );
    println!(
        "Error test STDERR: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Should fail due to unexpected exit code 1
    assert!(!output.status.success(), "exec_error_test should fail");

    // Clean up
    let _ = fs::remove_file("t_for_test/temp/exec_error_test.test");
    let _ = fs::remove_file("r/exec_error_test.result");
    let _ = fs::remove_dir("t_for_test/temp");
}
