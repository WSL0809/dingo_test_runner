//! Test case loader
//!
//! This module discovers and loads test files from the filesystem.

use anyhow::Result;
use once_cell::sync::OnceCell;
use std::path::Path;
use walkdir::WalkDir;

/// Finds all `.test` files in the specified directory (`t/`) and returns their names.
///
/// The function recursively walks through the `t/` directory. Test names are returned
/// relative to the `t/` directory, without the `.test` extension.
///
/// For example, a file at `t/feature/sub/my_test.test` will result in the name
/// `feature/sub/my_test`.
static TESTS_CACHE: OnceCell<Vec<String>> = OnceCell::new();

pub fn load_all_tests() -> Result<Vec<String>> {
    // 在单元测试环境下，可能多次切换当前工作目录。
    // 为避免 OnceCell 缓存跨目录污染，这里在 `cfg(test)` 下禁用缓存逻辑。
    let use_cache = !cfg!(test);

    if use_cache {
        if let Some(cached) = TESTS_CACHE.get() {
            return Ok(cached.clone());
        }
    }

    let mut tests = Vec::new();
    let test_dir = Path::new("t");

    if !test_dir.exists() || !test_dir.is_dir() {
        if use_cache {
            TESTS_CACHE.set(Vec::new()).ok();
        }
        return Ok(tests); // Return empty list if `t/` directory doesn't exist
    }

    for entry in WalkDir::new(test_dir)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| !e.file_type().is_dir())
    {
        let path = entry.path();
        if path.extension().map_or(false, |ext| ext == "test") {
            if let Ok(relative_path) = path.strip_prefix(test_dir) {
                let test_name = relative_path.with_extension("");
                tests.push(test_name.to_string_lossy().to_string());
            }
        }
    }

    if use_cache {
        TESTS_CACHE.set(tests.clone()).ok();
    }
    Ok(tests)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use std::path::Path;
    use tempfile::tempdir;

    #[test]
    fn test_load_all_tests_from_mock_dir() {
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let project_root = temp_dir.path();

        // Setup a mock directory structure
        let base_dir = project_root.join("t");
        let sub_dir = base_dir.join("sub");
        fs::create_dir_all(&sub_dir).expect("Failed to create test directory structure");

        // Create mock test files
        File::create(base_dir.join("test1.test")).expect("Failed to create test1.test");
        File::create(sub_dir.join("test2.test")).expect("Failed to create test2.test");
        File::create(base_dir.join("not_a_test.txt")).expect("Failed to create not_a_test.txt");

        // Temporarily change CWD for this test scope.
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(project_root).unwrap();

        let tests = load_all_tests().expect("Failed to load tests");

        // Restore CWD
        std::env::set_current_dir(original_dir).unwrap();

        // Debug output
        println!("Found tests in mock directory: {:?}", tests);

        // Strict assertions - should only find exactly the files we created
        assert_eq!(tests.len(), 2, "Expected exactly 2 test files, found: {:?}", tests);
        assert!(tests.contains(&"test1".to_string()), "Should contain test1");
        assert!(tests.contains(&"sub/test2".to_string()), "Should contain sub/test2");
        assert!(!tests.contains(&"not_a_test".to_string()), "Should not contain non-.test files");
    }

    #[test]
    fn test_load_all_tests_finds_project_files() {
        let project_root = Path::new(env!("CARGO_MANIFEST_DIR"));
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(project_root).unwrap();

        // Only run assertions if the `t` directory actually exists
        if project_root.join("t").exists() {
            let tests = load_all_tests().expect("Failed to load tests");
            assert!(!tests.is_empty(), "Should find test files in the project's t/ directory");
        }

        std::env::set_current_dir(original_dir).unwrap();
    }

    #[test]
    fn test_load_all_tests_empty_when_no_directory() {
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let project_root = temp_dir.path();

        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(project_root).unwrap();

        // Run the loader (should return empty since no t/ directory)
        let tests = load_all_tests().expect("Failed to load tests");

        std::env::set_current_dir(original_dir).unwrap();

        // Should be empty
        assert!(tests.is_empty(), "Should return empty list when t/ directory doesn't exist");
    }
} 