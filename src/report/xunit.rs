//! JUnit/XUnit XML report generation

use super::TestSuiteResult;
use crate::tester::tester::TestStatus;
use anyhow::Result;
use std::fs::File;
use std::io::Write;

/// Write test suite results to JUnit XML format
pub fn write_xunit_report(suite: &TestSuiteResult, file_path: &str) -> Result<()> {
    let mut file = File::create(file_path)?;

    // Write XML declaration
    writeln!(file, r#"<?xml version="1.0" encoding="UTF-8"?>"#)?;

    // Write testsuite opening tag
    writeln!(
        file,
        r#"<testsuite name="{}" tests="{}" failures="{}" skipped="{}" time="{:.3}" timestamp="{}">"#,
        escape_xml(&suite.name),
        suite.total_tests(),
        suite.failed_tests(),
        suite.skipped_tests(),
        suite.total_duration_ms as f64 / 1000.0,
        escape_xml(&suite.timestamp)
    )?;

    // Write properties section
    write_properties(&mut file, suite)?;

    // Write test cases
    for case in &suite.cases {
        write_test_case(&mut file, case)?;
    }

    // Close testsuite
    writeln!(file, "</testsuite>")?;

    Ok(())
}

/// Write properties section with environment information
fn write_properties(file: &mut File, suite: &TestSuiteResult) -> Result<()> {
    writeln!(file, "  <properties>")?;

    writeln!(
        file,
        r#"    <property name="os" value="{}"/>"#,
        escape_xml(&suite.environment.os)
    )?;
    writeln!(
        file,
        r#"    <property name="rust_version" value="{}"/>"#,
        escape_xml(&suite.environment.rust_version)
    )?;

    if let Some(ref git_commit) = suite.environment.git_commit {
        writeln!(
            file,
            r#"    <property name="git_commit" value="{}"/>"#,
            escape_xml(git_commit)
        )?;
    }

    if let Some(ref mysql_version) = suite.environment.mysql_version {
        writeln!(
            file,
            r#"    <property name="mysql_version" value="{}"/>"#,
            escape_xml(mysql_version)
        )?;
    }

    let cli_args = suite.environment.cli_args.join(" ");
    writeln!(
        file,
        r#"    <property name="cli_args" value="{}"/>"#,
        escape_xml(&cli_args)
    )?;

    writeln!(file, "  </properties>")?;
    Ok(())
}

/// Write a single test case
fn write_test_case(file: &mut File, case: &crate::tester::tester::TestResult) -> Result<()> {
    match case.status {
        TestStatus::Passed => {
            writeln!(
                file,
                r#"  <testcase name="{}" classname="{}" time="{:.3}"/>"#,
                escape_xml(&case.test_name),
                escape_xml(&case.classname),
                case.duration_ms as f64 / 1000.0
            )?;
        }
        TestStatus::Failed => {
            writeln!(
                file,
                r#"  <testcase name="{}" classname="{}" time="{:.3}">"#,
                escape_xml(&case.test_name),
                escape_xml(&case.classname),
                case.duration_ms as f64 / 1000.0
            )?;

            writeln!(
                file,
                r#"    <failure message="Test failed" type="TestFailure">"#
            )?;
            if !case.errors.is_empty() {
                let error_text = case.errors.join("\n");
                writeln!(file, "<![CDATA[{}]]>", error_text)?;
            }
            writeln!(file, "    </failure>")?;

            if !case.stdout.is_empty() {
                writeln!(
                    file,
                    "    <system-out><![CDATA[{}]]></system-out>",
                    case.stdout
                )?;
            }

            if !case.stderr.is_empty() {
                writeln!(
                    file,
                    "    <system-err><![CDATA[{}]]></system-err>",
                    case.stderr
                )?;
            }

            writeln!(file, "  </testcase>")?;
        }
        TestStatus::Skipped => {
            writeln!(
                file,
                r#"  <testcase name="{}" classname="{}" time="{:.3}">"#,
                escape_xml(&case.test_name),
                escape_xml(&case.classname),
                case.duration_ms as f64 / 1000.0
            )?;
            writeln!(file, "    <skipped/>")?;
            writeln!(file, "  </testcase>")?;
        }
    }

    Ok(())
}

/// Escape XML special characters
pub fn escape_xml(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::report::TestSuiteResult;
    use crate::tester::tester::TestResult;
    use tempfile::NamedTempFile;

    #[test]
    fn test_write_xunit_report() {
        let mut suite = TestSuiteResult::new("test-suite");

        // Add a passed test
        let mut passed_test = TestResult::new("test_passed");
        passed_test.duration_ms = 100;
        suite.add_case(passed_test);

        // Add a failed test
        let mut failed_test = TestResult::new("test_failed");
        failed_test.duration_ms = 200;
        failed_test.mark_failed();
        failed_test.add_error("Assertion failed".to_string());
        failed_test.set_stdout("Some output".to_string());
        suite.add_case(failed_test);

        // Write to temporary file
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path().to_str().unwrap();

        write_xunit_report(&suite, path).unwrap();

        // Verify file was created and contains expected content
        let content = std::fs::read_to_string(path).unwrap();
        assert!(content.contains("testsuite"));
        assert!(content.contains("test_passed"));
        assert!(content.contains("test_failed"));
        assert!(content.contains("failure"));
        assert!(content.contains("Assertion failed"));
    }

    #[test]
    fn test_escape_xml() {
        assert_eq!(escape_xml("hello & world"), "hello &amp; world");
        assert_eq!(escape_xml("<test>"), "&lt;test&gt;");
        assert_eq!(escape_xml("\"quoted\""), "&quot;quoted&quot;");
    }
}
