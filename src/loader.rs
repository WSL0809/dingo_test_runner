//! Test case loader
//!
//! This module discovers and loads test files from the filesystem.

use anyhow::Result;
use std::path::Path;
use walkdir::WalkDir;

/// Finds all `.test` files in the specified directory (`t/`) and returns their names.
///
/// The function recursively walks through the `t/` directory. Test names are returned
/// relative to the `t/` directory, without the `.test` extension.
///
/// For example, a file at `t/feature/sub/my_test.test` will result in the name
/// `feature/sub/my_test`.
pub fn load_all_tests() -> Result<Vec<String>> {
    let mut tests = Vec::new();
    let test_dir = Path::new("t");

    if !test_dir.exists() || !test_dir.is_dir() {
        return Ok(tests); // Return empty list if `t/` directory doesn't exist
    }

    for entry in WalkDir::new(test_dir)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| !e.file_type().is_dir())
    {
        let path = entry.path();
        if let Some(extension) = path.extension() {
            if extension == "test" {
                // Strip the `t/` prefix and `.test` extension
                if let Ok(relative_path) = path.strip_prefix(test_dir) {
                    let test_name = relative_path.with_extension("");
                    tests.push(test_name.to_string_lossy().to_string());
                }
            }
        }
    }

    Ok(tests)
} 