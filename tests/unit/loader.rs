use dingo_test_runner::loader::load_all_tests;
use std::fs::{self, File};
use std::env;
use std::path::Path;
use std::sync::Mutex;

// 使用静态锁确保测试之间不会互相干扰
static TEST_LOCK: Mutex<()> = Mutex::new(());

#[test]
fn test_load_all_tests_from_mock_dir() {
    let _lock = TEST_LOCK.lock().expect("Failed to acquire test lock");
    
    // Save current directory and change to a temporary location
    let original_dir = env::current_dir().expect("Failed to get current directory");
    let temp_dir = env::temp_dir().join(format!("mysql_tester_clean_test_{}_{}", 
        std::process::id(), 
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("Failed to get system time")
            .as_nanos()
    ));
    
    // Ensure clean start - remove any existing directory
    let _ = fs::remove_dir_all(&temp_dir);
    fs::create_dir_all(&temp_dir).expect("Failed to create temp directory");
    env::set_current_dir(&temp_dir).expect("Failed to change to temp directory");

    // Verify we're in a clean directory by checking no t/ exists
    assert!(!Path::new("t").exists(), "Should start with no t/ directory");

    // Setup a mock directory structure
    let base_dir = Path::new("t");
    let sub_dir = base_dir.join("sub");
    fs::create_dir_all(&sub_dir).expect("Failed to create test directory structure");

    // Create mock test files
    File::create(base_dir.join("test1.test")).expect("Failed to create test1.test");
    File::create(sub_dir.join("test2.test")).expect("Failed to create test2.test");
    File::create(base_dir.join("not_a_test.txt")).expect("Failed to create not_a_test.txt");

    // Run the loader
    let tests = load_all_tests().expect("Failed to load tests");

    // Debug output
    println!("Found tests in clean directory: {:?}", tests);

    // Strict assertions - should only find exactly the files we created
    assert_eq!(tests.len(), 2, "Expected exactly 2 test files, found: {:?}", tests);
    assert!(tests.contains(&"test1".to_string()), "Should contain test1");
    assert!(tests.contains(&"sub/test2".to_string()), "Should contain sub/test2");
    assert!(!tests.contains(&"not_a_test".to_string()), "Should not contain non-.test files");

    // Verify no unexpected files
    for test in &tests {
        assert!(test == "test1" || test == "sub/test2", "Unexpected test file found: {}", test);
    }

    // Cleanup
    env::set_current_dir(original_dir).expect("Failed to restore original directory");
    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_load_all_tests_finds_project_files() {
    let _lock = TEST_LOCK.lock().expect("Failed to acquire test lock");
    
    // Test with the actual project structure
    // Save and potentially restore current directory
    let original_dir = env::current_dir().expect("Failed to get current directory");
    
    // Try to find the project root if we're not already there
    let mut project_root = original_dir.clone();
    if !project_root.join("t").exists() {
        let mut search_dir = original_dir.clone();
        while !search_dir.join("t").exists() && search_dir.parent().is_some() {
            search_dir = search_dir.parent()
                .expect("Failed to get parent directory")
                .to_path_buf();
        }
        
        if search_dir.join("t").exists() {
            project_root = search_dir;
            env::set_current_dir(&project_root).expect("Failed to change to project root");
        }
    }
    
    let tests = load_all_tests().expect("Failed to load tests");
    
    // We should have at least some test files if we found the project
    if Path::new("t").exists() {
        assert!(!tests.is_empty(), "Should find test files in existing t/ directory");
        // Verify we can find expected test patterns
        let test_names_lower: Vec<String> = tests.iter().map(|s| s.to_lowercase()).collect();
        assert!(test_names_lower.iter().any(|name| name.contains("simple") || name.contains("error") || name.contains("example")),
                "Should find at least one recognizable test file, found: {:?}", tests);
    }
    
    // Restore original directory
    env::set_current_dir(original_dir).expect("Failed to restore original directory");
}

#[test]
fn test_load_all_tests_empty_when_no_directory() {
    let _lock = TEST_LOCK.lock().expect("Failed to acquire test lock");
    
    // Save current directory and change to a location without t/ directory
    let original_dir = env::current_dir().expect("Failed to get current directory");
    let temp_dir = env::temp_dir().join(format!("mysql_tester_empty_{}", std::process::id()));
    
    // Clean up any existing temp directory
    let _ = fs::remove_dir_all(&temp_dir);
    fs::create_dir_all(&temp_dir).expect("Failed to create temp directory");
    env::set_current_dir(&temp_dir).expect("Failed to change to temp directory");

    // Run the loader (should return empty since no t/ directory)
    let tests = load_all_tests().expect("Failed to load tests");

    // Should be empty
    assert!(tests.is_empty(), "Should return empty list when t/ directory doesn't exist");

    // Cleanup
    env::set_current_dir(original_dir).expect("Failed to restore original directory");
    let _ = fs::remove_dir_all(&temp_dir);
}
