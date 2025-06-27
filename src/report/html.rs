//! HTML report generation for test results
//!
//! This module provides HTML report generation using the Askama template engine.
//! It creates beautiful, responsive HTML reports with dark/light mode support.

use crate::report::TestSuiteResult;
use crate::tester::tester::TestResult;
use askama::Template;
use chrono::Local;

/// HTML report template data
#[derive(Template)]
#[template(path = "report.html")]
pub struct HtmlReport<'a> {
    pub cases: &'a [TestResult],
    pub generated_at: String,
    pub passed_tests: usize,
    pub failed_tests: usize,
    pub total_tests: usize,
    pub duration_seconds: f64,
}

impl<'a> HtmlReport<'a> {
    /// Create a new HTML report
    pub fn new(summary: &'a TestSuiteResult, cases: &'a [TestResult]) -> Self {
        Self {
            cases,
            generated_at: Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            passed_tests: summary.passed_tests(),
            failed_tests: summary.failed_tests(),
            total_tests: summary.total_tests(),
            duration_seconds: summary.total_duration_ms as f64 / 1000.0,
        }
    }

    /// Generate HTML content
    pub fn generate(&self) -> Result<String, askama::Error> {
        Template::render(self)
    }
}

/// Generate plain text report for email fallback
pub fn generate_plain_text_report(suite_result: &TestSuiteResult) -> String {
    let mut report = String::new();

    report.push_str("=".repeat(60).as_str());
    report.push_str("\n📊 MySQL 测试报告\n");
    report.push_str("=".repeat(60).as_str());
    report.push('\n');

    // 统计信息
    report.push_str(&format!("📋 执行摘要:\n"));
    report.push_str(&format!("  • 总测试数: {}\n", suite_result.total_tests()));
    report.push_str(&format!("  • 通过: {} ✓\n", suite_result.passed_tests()));
    report.push_str(&format!("  • 失败: {} ✗\n", suite_result.failed_tests()));
    report.push_str(&format!("  • 跳过: {} ⊘\n", suite_result.skipped_tests()));
    report.push_str(&format!("  • 通过率: {:.1}%\n", suite_result.pass_rate()));
    report.push_str(&format!(
        "  • 总用时: {:.2}s\n",
        suite_result.total_duration_ms as f64 / 1000.0
    ));
    report.push('\n');

    // 测试详情
    if !suite_result.cases.is_empty() {
        report.push_str("🧪 测试详情:\n");
        report.push_str("-".repeat(60).as_str());
        report.push('\n');

        for (index, case) in suite_result.cases.iter().enumerate() {
            let status_icon = match case.status {
                crate::tester::tester::TestStatus::Passed => "✓",
                crate::tester::tester::TestStatus::Failed => "✗",
                crate::tester::tester::TestStatus::Skipped => "⊘",
            };

            report.push_str(&format!(
                "{:3}. {} {} ({} ms)\n",
                index + 1,
                status_icon,
                case.test_name,
                case.duration_ms
            ));

            // 显示错误信息（如果有）
            if !case.errors.is_empty() {
                for error in &case.errors {
                    report.push_str(&format!("     错误: {}\n", error));
                }
            }
        }
        report.push('\n');
    }

    // 环境信息
    report.push_str("🔧 环境信息:\n");
    report.push_str(&format!("  • 操作系统: {}\n", suite_result.environment.os));
    if let Some(ref git_commit) = suite_result.environment.git_commit {
        report.push_str(&format!(
            "  • Git 提交: {}\n",
            &git_commit[..8.min(git_commit.len())]
        ));
    }
    report.push_str(&format!(
        "  • Rust 版本: {}\n",
        suite_result.environment.rust_version
    ));
    report.push_str(&format!(
        "  • 生成时间: {}\n",
        Local::now().format("%Y-%m-%d %H:%M:%S")
    ));
    report.push('\n');

    report.push_str("=".repeat(60).as_str());
    report.push_str("\n由 MySQL Test Runner (Rust) 自动生成\n");
    report.push_str("=".repeat(60).as_str());

    report
}
