//! Integration test: result file有多余行时应失败

use std::fs;
use std::path::Path;
use std::process::Command;

fn get_binary_path() -> String {
    if Path::new("target/release/dingo_test_runner").exists() {
        "target/release/dingo_test_runner".to_string()
    } else {
        if !Path::new("target/debug/dingo_test_runner").exists() {
            // 尝试构建
            let build_status = Command::new("cargo")
                .args(["build"])
                .status()
                .expect("Failed to build binary");
            assert!(build_status.success(), "Build failed");
        }
        "target/debug/dingo_test_runner".to_string()
    }
}

#[test]
fn test_result_missing_lines_detection() {
    let binary = get_binary_path();

    // 1. 写入 .test 文件
    let test_name = "result_mismatch_test";
    let test_file_content = "--echo Line1\n";
    let test_path = Path::new("t").join(format!("{}.test", test_name));
    fs::create_dir_all("t").unwrap();
    fs::write(&test_path, test_file_content).expect("write test file");

    // 2. 写入 result 文件（多一行）
    let result_dir = Path::new("r");
    fs::create_dir_all(result_dir).unwrap();
    let result_path = result_dir.join(format!("{}.result", test_name));
    fs::write(&result_path, "Line1\nExtra\n").expect("write result file");

    // 3. 运行比较模式，应失败
    let output = Command::new(&binary)
        .arg(test_name)
        .arg("--reserve-schema")
        .output()
        .expect("execute binary");

    println!("STDOUT: {}", String::from_utf8_lossy(&output.stdout));
    println!("STDERR: {}", String::from_utf8_lossy(&output.stderr));

    assert!(
        !output.status.success(),
        "Process should fail due to missing lines"
    );

    // 清理
    let _ = fs::remove_file(test_path);
    let _ = fs::remove_file(result_path);
}
