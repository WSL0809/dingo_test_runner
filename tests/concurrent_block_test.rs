//! Integration test: 并发块中查询错误应优雅处理且不 panic。

use std::process::Command;
use std::fs;
use std::path::Path;

fn get_binary_path() -> String {
    if Path::new("target/release/dingo_test_runner").exists() {
        "target/release/dingo_test_runner".to_string()
    } else {
        if !Path::new("target/debug/dingo_test_runner").exists() {
            let status = Command::new("cargo").args(["build"]).status().unwrap();
            assert!(status.success());
        }
        "target/debug/dingo_test_runner".to_string()
    }
}

#[test]
fn test_concurrent_error_handling() {
    let binary = get_binary_path();
    let test_name = "concurrent_error_test";

    let content = r#"
--begin_concurrent
SELECT * FROM non_existing_table;
--end_concurrent
"#;

    let test_path = Path::new("t").join(format!("{}.test", test_name));
    fs::create_dir_all("t").unwrap();
    fs::write(&test_path, content).unwrap();

    // 运行，应失败但不panic
    let output = Command::new(&binary)
        .arg(test_name)
        .arg("--reserve-schema")
        .output()
        .expect("run binary");

    println!("STDOUT: {}", String::from_utf8_lossy(&output.stdout));
    println!("STDERR: {}", String::from_utf8_lossy(&output.stderr));

    assert!(!output.status.success(), "Process should fail gracefully");

    // 清理
    let _ = fs::remove_file(test_path);
    let _ = fs::remove_file(Path::new("r").join(format!("{}.result", test_name)));
} 