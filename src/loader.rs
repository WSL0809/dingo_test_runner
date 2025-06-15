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
    if let Some(cached) = TESTS_CACHE.get() {
        return Ok(cached.clone());
    }

    let mut tests = Vec::new();
    let test_dir = Path::new("t");

    if !test_dir.exists() || !test_dir.is_dir() {
        TESTS_CACHE.set(Vec::new()).ok();
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

    TESTS_CACHE.set(tests.clone()).ok();
    Ok(tests)
} 