# dingo_test_runner 测试命令指南

## 🚀 快速开始

### 构建项目
```bash
# 基础构建
cargo build

# 优化构建
cargo build --release
```

### 基本测试命令
```bash
# 运行单个测试
cargo run -- simple_test

# 运行指定的 .test 文件  
cargo run -- t/simple_test.test

# 运行新分类结构中的测试
cargo run -- t_for_test/basic/simple_test
```

## 📊 Record 模式 vs 比对模式

### Record 模式 - 生成期望结果
```bash
# 为测试生成 .result 文件
cargo run -- --record simple_test
cargo run -- --record replace_regex_test
cargo run -- --record sorted_result_test

# Record 模式会将测试输出保存到 r/ 目录
# 例如: simple_test.test -> r/simple_test.result
```

### 比对模式 - 验证测试结果
```bash
# 默认模式，与期望结果比对
cargo run -- simple_test
cargo run -- replace_regex_test  
cargo run -- sorted_result_test

# 通过比对 r/ 目录中的 .result 文件验证输出
```

## 🗂️ 新分类测试结构

### 运行分类测试
```bash
# 基础功能测试
cargo run -- t_for_test/basic/simple_test
cargo run -- t_for_test/basic/echo_test
cargo run -- t_for_test/basic/mysql_connect

# 变量系统测试
cargo run -- t_for_test/variables/variable_basic
cargo run -- t_for_test/variables/eval_test

# 控制流测试
cargo run -- t_for_test/control_flow/if_simple
cargo run -- t_for_test/control_flow/while_simple
cargo run -- t_for_test/control_flow/nested_control_flow

# 错误处理测试
cargo run -- t_for_test/error_handling/error_test
cargo run -- t_for_test/error_handling/expected_error_test

# 高级功能测试
cargo run -- t_for_test/advanced/replace_regex_test
cargo run -- t_for_test/advanced/sorted_result_test
cargo run -- t_for_test/advanced/regex_test

# 连接管理测试
cargo run -- t_for_test/connection/connection_test
cargo run -- t_for_test/connection/connection_management

# Source/Include 功能测试
cargo run -- t_for_test/source/source_basic
cargo run -- t_for_test/source/source_comprehensive

# 性能边界测试
cargo run -- t_for_test/performance/parser_edge_cases
cargo run -- t_for_test/performance/create_tables_loop

# 并发执行测试
cargo run -- t_for_test/concurrent/concurrent_basic
```

### 运行整个分类目录
```bash
# 运行基础功能分类的所有测试
cargo run -- t_for_test/basic/

# 运行变量系统分类的所有测试
cargo run -- t_for_test/variables/

# 运行控制流分类的所有测试
cargo run -- t_for_test/control_flow/
```

## 🔧 数据库连接参数

### 基本连接参数
```bash
# 指定数据库连接信息
cargo run -- --host 127.0.0.1 --port 3306 --user root --passwd password simple_test

# 使用默认连接（127.0.0.1:3306, root用户）
cargo run -- simple_test
```

### 完整连接示例
```bash
# 连接到自定义数据库
cargo run -- \
  --host 192.168.1.100 \
  --port 3307 \
  --user testuser \
  --passwd testpass \
  --database testdb \
  simple_test
```

## 📝 日志和调试

### 日志级别控制
```bash
# 错误级别日志
cargo run -- --log-level error simple_test

# 详细调试日志
cargo run -- --log-level debug simple_test

# 完整追踪日志
cargo run -- --log-level trace simple_test

# 使用环境变量控制日志
RUST_LOG=debug cargo run -- simple_test
RUST_LOG=trace cargo run -- simple_test
```

### 解析器调试
```bash
# 使用手写解析器
cargo run --no-default-features -- simple_test

# 使用 Pest 解析器（默认）
cargo run --features pest -- simple_test
```

## 📋 批量测试

### 运行所有测试
```bash
# 运行所有测试
cargo run -- --all
```

### 使用分类测试脚本
```bash
# 使用新的分类测试脚本
./run_categorized_tests.sh --help

# 列出所有分类
./run_categorized_tests.sh --list

# 运行特定分类
./run_categorized_tests.sh basic
./run_categorized_tests.sh variables control_flow

# 运行所有分类
./run_categorized_tests.sh --all

# 预览模式（不实际执行）
./run_categorized_tests.sh --dry-run basic

# 验证新旧结构兼容性
./run_categorized_tests.sh --verify
```

## 📊 报告格式

### 不同格式的报告输出
```bash
# 终端彩色输出（默认）
cargo run -- --report-format terminal simple_test

# HTML 报告
cargo run -- --report-format html simple_test

# JUnit XML 报告
cargo run -- --report-format xunit --xunit-file report.xml simple_test

# Allure 企业级报告
cargo run -- --report-format allure --allure-dir ./allure-results simple_test

# 纯文本报告
cargo run -- --report-format plain simple_test
```

## ⚡ 性能优化

### 快速失败模式
```bash
# 遇到错误立即停止（默认）
cargo run -- --fail-fast true simple_test

# 继续执行所有测试
cargo run -- --fail-fast false simple_test
```

## 🧪 测试验证

### 新增测试的验证流程
```bash
# 1. 首先生成期望结果
cargo run -- --record new_test

# 2. 验证测试通过
cargo run -- new_test

# 3. 检查结果文件
ls -la r/new_test.result
```

### 测试结果验证
```bash
# 验证测试执行时间和通过率
cargo run -- simple_test
# 输出示例:
# ▶ running simple_test ... ✓ simple_test (65547 ms)
# Total: 1 Passed: 1 ⏱ 65.5 s
# Pass rate: 100.0%
```

## 🔍 故障排查

### 常见问题解决
```bash
# 如果测试失败显示 "Result file not found"
# 需要先生成结果文件
cargo run -- --record failing_test_name

# 如果连接数据库失败
# 检查数据库服务状态并验证连接参数
cargo run -- --host localhost --port 3306 --user root --passwd your_password simple_test

# 如果解析错误
# 尝试使用不同的解析器
cargo run --no-default-features -- simple_test
```

### 性能分析
```bash
# 查看详细执行时间
RUST_LOG=info cargo run -- simple_test

# 分析并发执行性能
cargo run -- t_for_test/concurrent/concurrent_basic
```

## 📈 最佳实践

### 开发新功能时的测试流程
```bash
# 1. 开发相关功能的基础测试
cargo run -- t_for_test/basic/

# 2. 测试变量系统相关功能
cargo run -- t_for_test/variables/

# 3. 测试控制流功能
cargo run -- t_for_test/control_flow/

# 4. 测试错误处理
cargo run -- t_for_test/error_handling/

# 5. 运行所有测试验证
./run_categorized_tests.sh --all
```

### 持续集成测试
```bash
# CI/CD 环境推荐命令
cargo build --release
cargo run --release -- --all --report-format xunit --xunit-file ci_report.xml
```

## 📚 相关文档

- [README.md](README.md) - 项目总体介绍
- [SOURCE_IMPLEMENTATION.md](SOURCE_IMPLEMENTATION.md) - Source功能实现详情
- [TEST_RESTRUCTURE_SUMMARY.md](TEST_RESTRUCTURE_SUMMARY.md) - 测试重构总结
- [t_for_test/README.md](t_for_test/README.md) - 新测试结构说明

---

> 💡 **提示**: 首次运行测试时，建议先使用 `--record` 模式生成期望结果，然后再进行正常的比对测试。