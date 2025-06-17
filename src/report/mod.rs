use crate::tester::tester::{TestResult, TestStatus};

pub mod xunit;
pub mod summary;

/// Test suite result aggregating multiple test cases
#[derive(Debug, Clone)]
pub struct TestSuiteResult {
    /// Test suite name
    pub name: String,
    /// Individual test case results
    pub cases: Vec<TestResult>,
    /// Total execution time in milliseconds
    pub total_duration_ms: u64,
    /// Test execution timestamp
    pub timestamp: String,
    /// Environment information
    pub environment: EnvironmentInfo,
}

/// Environment and execution context information
#[derive(Debug, Clone)]
pub struct EnvironmentInfo {
    /// Git commit SHA (if available)
    pub git_commit: Option<String>,
    /// Operating system
    pub os: String,
    /// Rust version
    pub rust_version: String,
    /// MySQL version (if available)
    pub mysql_version: Option<String>,
    /// Command line arguments
    pub cli_args: Vec<String>,
}

impl TestSuiteResult {
    /// Create a new test suite result
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            cases: Vec::new(),
            total_duration_ms: 0,
            timestamp: chrono::Utc::now().to_rfc3339(),
            environment: EnvironmentInfo::new(),
        }
    }

    /// Add a test case result
    pub fn add_case(&mut self, case: TestResult) {
        self.total_duration_ms += case.duration_ms;
        self.cases.push(case);
    }

    /// Get total number of tests
    pub fn total_tests(&self) -> usize {
        self.cases.len()
    }

    /// Get number of passed tests
    pub fn passed_tests(&self) -> usize {
        self.cases.iter().filter(|c| c.status == TestStatus::Passed).count()
    }

    /// Get number of failed tests
    pub fn failed_tests(&self) -> usize {
        self.cases.iter().filter(|c| c.status == TestStatus::Failed).count()
    }

    /// Get number of skipped tests
    pub fn skipped_tests(&self) -> usize {
        self.cases.iter().filter(|c| c.status == TestStatus::Skipped).count()
    }

    /// Get pass rate as percentage
    pub fn pass_rate(&self) -> f64 {
        if self.total_tests() == 0 {
            return 0.0;
        }
        (self.passed_tests() as f64 / self.total_tests() as f64) * 100.0
    }

    /// Check if all tests passed
    pub fn all_passed(&self) -> bool {
        self.failed_tests() == 0
    }
}

impl EnvironmentInfo {
    /// Create new environment info
    pub fn new() -> Self {
        Self {
            git_commit: Self::get_git_commit(),
            os: std::env::consts::OS.to_string(),
            rust_version: env!("CARGO_PKG_VERSION").to_string(),
            mysql_version: None, // Will be set later if available
            cli_args: std::env::args().collect(),
        }
    }

    /// Get git commit SHA if available
    fn get_git_commit() -> Option<String> {
        // Try to get git commit from environment or git command
        if let Ok(commit) = std::env::var("GIT_COMMIT") {
            return Some(commit);
        }
        
        // Try to execute git command
        if let Ok(output) = std::process::Command::new("git")
            .args(&["rev-parse", "HEAD"])
            .output() 
        {
            if output.status.success() {
                let commit = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if !commit.is_empty() {
                    return Some(commit);
                }
            }
        }
        
        None
    }
}
