use crate::tester::tester::{TestResult, TestStatus};
use anyhow::Result;

pub mod xunit;
pub mod summary;
pub mod html;
pub mod allure;

/// 统一的报告渲染接口
pub trait ReportRenderer {
    /// 渲染报告并返回内容
    fn render(&self, suite: &TestSuiteResult) -> Result<String>;
    
    /// 获取渲染器的输出格式名称
    fn format_name(&self) -> &'static str;
}

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

/// 创建指定格式的报告渲染器
pub fn create_renderer(format: &str) -> Result<Box<dyn ReportRenderer>> {
    match format.to_lowercase().as_str() {
        "plain" | "text" => Ok(Box::new(PlainTextRenderer)),
        "html" => Ok(Box::new(SimpleHtmlRenderer)),
        "xunit" | "xml" | "junit" => Ok(Box::new(XunitRenderer)),
        "terminal" | "console" => Ok(Box::new(TerminalRenderer)),
        _ => Err(anyhow::anyhow!("Unsupported report format: {}", format)),
    }
}

/// 创建指定格式的报告渲染器，支持 Allure 输出目录
pub fn create_renderer_with_allure_dir(format: &str, allure_dir: Option<&str>) -> Result<Box<dyn ReportRenderer>> {
    match format.to_lowercase().as_str() {
        "plain" | "text" => Ok(Box::new(PlainTextRenderer)),
        "html" => Ok(Box::new(SimpleHtmlRenderer)),
        "xunit" | "xml" | "junit" => Ok(Box::new(XunitRenderer)),
        "terminal" | "console" => Ok(Box::new(TerminalRenderer)),
        "allure" => {
            if let Some(dir) = allure_dir {
                Ok(Box::new(allure::AllureRenderer::new(dir.to_string())))
            } else {
                Err(anyhow::anyhow!("Allure format requires --allure-dir parameter"))
            }
        },
        _ => Err(anyhow::anyhow!("Unsupported report format: {}", format)),
    }
}

/// 纯文本渲染器
pub struct PlainTextRenderer;

impl ReportRenderer for PlainTextRenderer {
    fn render(&self, suite: &TestSuiteResult) -> Result<String> {
        Ok(html::generate_plain_text_report(suite))
    }

    fn format_name(&self) -> &'static str {
        "plain"
    }
}

/// 简陋但实用的 HTML 渲染器
pub struct SimpleHtmlRenderer;

impl ReportRenderer for SimpleHtmlRenderer {
    fn render(&self, suite: &TestSuiteResult) -> Result<String> {
        let mut html = String::new();
        
        // HTML 基础结构
        html.push_str("<!DOCTYPE html>\n");
        html.push_str("<html lang=\"zh-CN\">\n");
        html.push_str("<head>\n");
        html.push_str("  <meta charset=\"UTF-8\">\n");
        html.push_str("  <meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">\n");
        html.push_str("  <title>MySQL 测试报告</title>\n");
        html.push_str("  <style>\n");
        html.push_str("    body { font-family: Arial, sans-serif; margin: 20px; }\n");
        html.push_str("    .summary { background: #f5f5f5; padding: 15px; border-radius: 5px; margin-bottom: 20px; }\n");
        html.push_str("    .passed { color: #28a745; font-weight: bold; }\n");
        html.push_str("    .failed { color: #dc3545; font-weight: bold; }\n");
        html.push_str("    .skipped { color: #ffc107; font-weight: bold; }\n");
        html.push_str("    table { border-collapse: collapse; width: 100%; }\n");
        html.push_str("    th, td { border: 1px solid #ddd; padding: 8px; text-align: left; }\n");
        html.push_str("    th { background-color: #f2f2f2; }\n");
        html.push_str("    .error { background-color: #f8d7da; }\n");
        html.push_str("  </style>\n");
        html.push_str("</head>\n");
        html.push_str("<body>\n");
        
        // 标题
        html.push_str(&format!("  <h1>🧪 MySQL 测试报告: {}</h1>\n", suite.name));
        
        // 统计摘要
        html.push_str("  <div class=\"summary\">\n");
        html.push_str("    <h2>📊 执行摘要</h2>\n");
        html.push_str(&format!("    <p><strong>总测试数:</strong> {}</p>\n", suite.total_tests()));
        html.push_str(&format!("    <p><strong>通过:</strong> <span class=\"passed\">{}</span></p>\n", suite.passed_tests()));
        html.push_str(&format!("    <p><strong>失败:</strong> <span class=\"failed\">{}</span></p>\n", suite.failed_tests()));
        html.push_str(&format!("    <p><strong>跳过:</strong> <span class=\"skipped\">{}</span></p>\n", suite.skipped_tests()));
        html.push_str(&format!("    <p><strong>通过率:</strong> {:.1}%</p>\n", suite.pass_rate()));
        html.push_str(&format!("    <p><strong>总用时:</strong> {:.2}s</p>\n", suite.total_duration_ms as f64 / 1000.0));
        html.push_str(&format!("    <p><strong>生成时间:</strong> {}</p>\n", chrono::Local::now().format("%Y-%m-%d %H:%M:%S")));
        html.push_str("  </div>\n");
        
        // 测试详情表格
        if !suite.cases.is_empty() {
            html.push_str("  <h2>🧪 测试详情</h2>\n");
            html.push_str("  <table>\n");
            html.push_str("    <thead>\n");
            html.push_str("      <tr>\n");
            html.push_str("        <th>序号</th>\n");
            html.push_str("        <th>测试名称</th>\n");
            html.push_str("        <th>状态</th>\n");
            html.push_str("        <th>用时 (ms)</th>\n");
            html.push_str("        <th>通过查询</th>\n");
            html.push_str("        <th>失败查询</th>\n");
            html.push_str("        <th>首个错误</th>\n");
            html.push_str("      </tr>\n");
            html.push_str("    </thead>\n");
            html.push_str("    <tbody>\n");
            
            for (index, case) in suite.cases.iter().enumerate() {
                let status_class = match case.status {
                    TestStatus::Passed => "passed",
                    TestStatus::Failed => "failed",
                    TestStatus::Skipped => "skipped",
                };
                let status_symbol = match case.status {
                    TestStatus::Passed => "✓",
                    TestStatus::Failed => "✗",
                    TestStatus::Skipped => "⊘",
                };
                let row_class = if case.status == TestStatus::Failed { " class=\"error\"" } else { "" };
                
                html.push_str(&format!("      <tr{}>\n", row_class));
                html.push_str(&format!("        <td>{}</td>\n", index + 1));
                html.push_str(&format!("        <td>{}</td>\n", html_escape(&case.test_name)));
                html.push_str(&format!("        <td><span class=\"{}\">{} {}</span></td>\n", 
                              status_class, status_symbol, status_class.to_uppercase()));
                html.push_str(&format!("        <td>{}</td>\n", case.duration_ms));
                html.push_str(&format!("        <td>{}</td>\n", case.passed_queries));
                html.push_str(&format!("        <td>{}</td>\n", case.failed_queries));
                
                let first_error = case.errors.first().map(|e| html_escape(e)).unwrap_or_else(|| "-".to_string());
                html.push_str(&format!("        <td>{}</td>\n", first_error));
                html.push_str("      </tr>\n");
            }
            
            html.push_str("    </tbody>\n");
            html.push_str("  </table>\n");
        }
        
        // 环境信息
        html.push_str("  <h2>🔧 环境信息</h2>\n");
        html.push_str("  <ul>\n");
        html.push_str(&format!("    <li><strong>操作系统:</strong> {}</li>\n", suite.environment.os));
        html.push_str(&format!("    <li><strong>Rust 版本:</strong> {}</li>\n", suite.environment.rust_version));
        if let Some(ref git_commit) = suite.environment.git_commit {
            html.push_str(&format!("    <li><strong>Git 提交:</strong> {}</li>\n", &git_commit[..8.min(git_commit.len())]));
        }
        if let Some(ref mysql_version) = suite.environment.mysql_version {
            html.push_str(&format!("    <li><strong>MySQL 版本:</strong> {}</li>\n", mysql_version));
        }
        html.push_str("  </ul>\n");
        
        html.push_str("</body>\n");
        html.push_str("</html>\n");
        
        Ok(html)
    }

    fn format_name(&self) -> &'static str {
        "html"
    }
}

/// XUnit/JUnit XML 渲染器
pub struct XunitRenderer;

impl ReportRenderer for XunitRenderer {
    fn render(&self, suite: &TestSuiteResult) -> Result<String> {
        let mut xml = String::new();
        
        // XML 声明
        xml.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
        
        // testsuite 开始标签
        xml.push_str(&format!(
            r#"<testsuite name="{}" tests="{}" failures="{}" skipped="{}" time="{:.3}" timestamp="{}">"#,
            xunit::escape_xml(&suite.name),
            suite.total_tests(),
            suite.failed_tests(),
            suite.skipped_tests(),
            suite.total_duration_ms as f64 / 1000.0,
            xunit::escape_xml(&suite.timestamp)
        ));
        xml.push('\n');
        
        // properties 节
        xml.push_str("  <properties>\n");
        xml.push_str(&format!(r#"    <property name="os" value="{}"/>"#, xunit::escape_xml(&suite.environment.os)));
        xml.push('\n');
        xml.push_str(&format!(r#"    <property name="rust_version" value="{}"/>"#, xunit::escape_xml(&suite.environment.rust_version)));
        xml.push('\n');
        
        if let Some(ref git_commit) = suite.environment.git_commit {
            xml.push_str(&format!(r#"    <property name="git_commit" value="{}"/>"#, xunit::escape_xml(git_commit)));
            xml.push('\n');
        }
        
        if let Some(ref mysql_version) = suite.environment.mysql_version {
            xml.push_str(&format!(r#"    <property name="mysql_version" value="{}"/>"#, xunit::escape_xml(mysql_version)));
            xml.push('\n');
        }
        
        let cli_args = suite.environment.cli_args.join(" ");
        xml.push_str(&format!(r#"    <property name="cli_args" value="{}"/>"#, xunit::escape_xml(&cli_args)));
        xml.push('\n');
        xml.push_str("  </properties>\n");
        
        // 测试用例
        for case in &suite.cases {
            match case.status {
                TestStatus::Passed => {
                    xml.push_str(&format!(
                        r#"  <testcase name="{}" classname="{}" time="{:.3}"/>"#,
                        xunit::escape_xml(&case.test_name),
                        xunit::escape_xml(&case.classname),
                        case.duration_ms as f64 / 1000.0
                    ));
                    xml.push('\n');
                }
                TestStatus::Failed => {
                    xml.push_str(&format!(
                        r#"  <testcase name="{}" classname="{}" time="{:.3}">"#,
                        xunit::escape_xml(&case.test_name),
                        xunit::escape_xml(&case.classname),
                        case.duration_ms as f64 / 1000.0
                    ));
                    xml.push('\n');
                    
                    xml.push_str(r#"    <failure message="Test failed" type="TestFailure">"#);
                    xml.push('\n');
                    if !case.errors.is_empty() {
                        let error_text = case.errors.join("\n");
                        xml.push_str(&format!("<![CDATA[{}]]>", error_text));
                        xml.push('\n');
                    }
                    xml.push_str("    </failure>\n");
                    
                    if !case.stdout.is_empty() {
                        xml.push_str(&format!("    <system-out><![CDATA[{}]]></system-out>\n", case.stdout));
                    }
                    
                    if !case.stderr.is_empty() {
                        xml.push_str(&format!("    <system-err><![CDATA[{}]]></system-err>\n", case.stderr));
                    }
                    
                    xml.push_str("  </testcase>\n");
                }
                TestStatus::Skipped => {
                    xml.push_str(&format!(
                        r#"  <testcase name="{}" classname="{}" time="{:.3}">"#,
                        xunit::escape_xml(&case.test_name),
                        xunit::escape_xml(&case.classname),
                        case.duration_ms as f64 / 1000.0
                    ));
                    xml.push('\n');
                    xml.push_str("    <skipped/>\n");
                    xml.push_str("  </testcase>\n");
                }
            }
        }
        
        xml.push_str("</testsuite>\n");
        
        Ok(xml)
    }

    fn format_name(&self) -> &'static str {
        "xunit"
    }
}

/// 终端彩色输出渲染器
pub struct TerminalRenderer;

impl ReportRenderer for TerminalRenderer {
    fn render(&self, suite: &TestSuiteResult) -> Result<String> {
        let mut output = String::new();
        
        // 使用现有的 summary 模块生成彩色输出
        // 这里我们需要模拟 print 行为，捕获到字符串中
        output.push_str(&format!("Total: {} ", suite.total_tests()));
        
        if suite.passed_tests() > 0 {
            output.push_str(&format!("Passed: {} ", suite.passed_tests()));
        }
        
        if suite.failed_tests() > 0 {
            output.push_str(&format!("Failed: {} ", suite.failed_tests()));
        }
        
        if suite.skipped_tests() > 0 {
            output.push_str(&format!("Skipped: {} ", suite.skipped_tests()));
        }
        
        output.push_str(&format!("⏱ {:.1} s\n", suite.total_duration_ms as f64 / 1000.0));
        
        // 通过率
        if suite.total_tests() > 0 {
            output.push_str(&format!("Pass rate: {:.1}%\n", suite.pass_rate()));
        }
        
        // 失败详情
        if suite.failed_tests() > 0 {
            output.push_str("Failed tests:\n");
            for case in &suite.cases {
                if case.status == TestStatus::Failed {
                    output.push_str(&format!("  • {}\n", case.test_name));
                    for error in &case.errors {
                        output.push_str(&format!("    {}\n", error));
                    }
                }
            }
        }
        
        Ok(output)
    }

    fn format_name(&self) -> &'static str {
        "terminal"
    }
}

/// HTML 转义函数
fn html_escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}
