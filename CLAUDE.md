# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is `dingo_test_runner`, a Rust-based MySQL test runner compatible with MySQL's official test format. The project supports parsing and executing `.test` files, result comparison, concurrent execution, and multiple report formats.

## Key Architecture

The system uses a layered architecture:

```
CLI → Loader → Parser (Pest) → Tester → Database → Reports
```

Core modules:
- **CLI layer** (`cli.rs`): Command-line argument parsing and input format resolution
- **Loader** (`loader.rs`): Test file discovery and loading from `t/` directory
- **Parser** (`tester/pest_parser.rs`): Pest-based syntax parser using `mysql_test.pest` grammar
- **Tester** (`tester/tester.rs`): Core test execution engine with serial and concurrent support
- **Database** (`tester/database.rs`): MySQL/SQLite abstraction with connection management
- **Reports** (`report/`): Multi-format reporting (Terminal, HTML, JUnit XML, Allure)

## Build Commands

```bash
# Build project
cargo build

# Release build
cargo build --release

# Run tests (Rust unit tests)
cargo test
```

## Core Usage Commands

### Basic Test Execution
```bash
# Run single test by name (searches t/<name>.test)
cargo run -- basic_test

# Run specific test file
cargo run -- t/basic_test.test

# Run all tests in directory
cargo run -- t/demo_tests/

# 🔥 Run tests from any directory (NEW FEATURE)
cargo run -- path/to/test_dir/
cargo run -- custom_tests/my_test.test
cargo run -- t_for_test/variables/

# Run all tests
cargo run -- --all
```

### Database Connection
```bash
# Default connection (127.0.0.1:3306, root user)
cargo run -- test_name

# Custom connection
cargo run -- --host 127.0.0.1 --port 3306 --user root --passwd password test_name
```

### Record vs Compare Modes
```bash
# Record mode: Generate expected results (creates r/<test>.result)
cargo run -- --record test_name

# Compare mode: Validate against expected results (default)
cargo run -- test_name

# 🔥 使用自定义扩展名进行环境隔离
cargo run -- --extension dev --record test_name    # 生成 r/test_name.dev
cargo run -- --extension dev test_name             # 与 r/test_name.dev 比对
```

### Parallel Execution
```bash
# File-level concurrency (NEW FEATURE)
cargo run -- --parallel 4 test1 test2 test3 test4

# Serial execution (default, backward compatible)
cargo run -- test1 test2 test3
```

### Reporting Formats
```bash
# Terminal output (default)
cargo run -- --report-format terminal test_name

# HTML report
cargo run -- --report-format html test_name

# JUnit XML for CI/CD
cargo run -- --report-format xunit --xunit-file report.xml test_name

# Allure enterprise reporting
cargo run -- --report-format allure --allure-dir ./allure-results test_name
```

## Test File Structure

**用户目录（客户使用）**：
- `t/examples/` - 用户友好的测试示例（3个精选示例）
- `t/br/` - 特殊 BR 功能测试（保留）
- `t/demo_tests/` - 演示测试套件（保留）
- `t/include/` - Include files (`.inc`) for `--source` functionality
- `r/examples/` - 用户示例的期望结果文件

**开发者目录（集成测试）**：
- `tests/integration/basic/` - 基础功能集成测试
- `tests/integration/variables/` - 变量系统集成测试
- `tests/integration/control_flow/` - 控制流集成测试
- `tests/integration/concurrent/` - 并发功能集成测试
- `tests/integration/connection/` - 连接管理集成测试
- `tests/integration/error_handling/` - 错误处理集成测试
- `tests/integration/source/` - Source 包含功能集成测试
- `tests/integration/advanced/` - 高级功能集成测试
- `tests/integration/performance/` - 性能测试
- `tests/results/` - 集成测试的期望结果文件（按扩展名分类）

**🔥 任意目录支持（新功能）**：
- `t_for_test/` - 开发过程测试目录，完全支持任意路径访问
- `custom_tests/` - 用户自定义测试目录
- 支持绝对路径和相对路径访问任意 `.test` 文件

## Supported Test Language Features

The test format supports 48+ query types and directives:

- **Basic SQL**: Standard SQL queries and statements
- **Variables**: `let $var = value` and `$var` expansion, including SQL backtick expressions
- **Control flow**: `if ($condition)` / `while ($condition)` / `end` statements
- **Concurrency**: `--BEGIN_CONCURRENT` / `--END_CONCURRENT` blocks
- **Output control**: `--echo`, `--sorted_result`, `--replace_regex`
- **Error handling**: `--error <code>` for expected errors
- **File inclusion**: `--source <file>` for modular test scripts
- **Connection management**: `--connect` / `--disconnect`
- **System commands**: `--exec <command>`

## Debugging and Development

### Log Levels
```bash
# Debug logging
RUST_LOG=debug cargo run -- test_name

# Trace logging for parser
RUST_LOG=dingo_test_runner::tester::pest_parser=debug cargo run -- test_name

# Full trace
RUST_LOG=trace cargo run -- test_name
```

### Script Tools
```bash
# Run categorized tests
./run_categorized_tests.sh --help
./run_categorized_tests.sh basic variables

# Reorganize test structure
./reorganize_tests.sh
```

## Key Implementation Notes

- **Pest Parser**: Uses `src/tester/mysql_test.pest` grammar file for parsing
- **Connection Pooling**: Automatic connection management with configurable `--max-connections`
- **Database Isolation**: File-level concurrency uses temporary databases (`test_{name}_{thread}_{timestamp}_{pid}`)
- **Variable System**: Full variable expansion with expression evaluation support
- **Backward Compatibility**: All existing functionality preserved when adding new features

## Test Development Workflow

### 🎯 开发者集成测试工作流（推荐）

```bash
# 1. 开发新功能时，为集成测试创建基线
cargo run -- --extension dev --record tests/integration/

# 2. 开发过程中快速验证
cargo run -- --extension dev tests/integration/basic/
cargo run -- --extension dev tests/integration/variables/

# 3. 🔥 使用开发测试目录（新功能）
cargo run -- --extension dev --record t_for_test/basic/
cargo run -- --extension dev t_for_test/variables/variable_simple.test

# 4. 功能完成后全量并发回归测试
cargo run -- --extension dev --parallel 8 tests/integration/
```

### 📦 用户测试工作流

```bash
# 1. 创建用户测试文件到 t/ 目录
# 2. 生成期望结果: cargo run -- --record test_name
# 3. 验证测试: cargo run -- test_name
# 4. 对于演示测试，使用 t/demo_tests/ 结构
```

### 🔧 开发者常用命令别名

```bash
alias dev-test="cargo run -- --extension dev"
alias test-record="cargo run -- --extension dev --record"
alias integration-test="cargo run -- --extension integration"

# 使用示例
dev-test tests/integration/basic/                    # 验证基础功能
test-record tests/integration/variables/new_feature  # 记录新功能基线
dev-test --parallel 4 tests/integration/            # 全量并发测试

# 🔥 任意目录测试（新功能）
dev-test t_for_test/basic/                          # 开发测试目录
test-record t_for_test/variables/                   # 记录开发测试基线
dev-test ../other_project/tests/                    # 其他项目测试
```

## Common Issues

- If test fails with "Result file not found", run with `--record` first (or with `--extension <env>` for specific environment)
- For database connection issues, verify MySQL service and connection parameters
- For concurrent execution issues, check database connection limits and reduce `--parallel` value
- When switching between environments, ensure you're using the correct `--extension` parameter

## Development Guidelines

- 新的修改不要影响原有功能
- **🚨 重要：开发者请使用 `tests/integration/` 目录进行集成测试，不要在 `t/` 目录添加开发测试**
- **🔥 使用 `--extension dev` 进行日常开发测试，保持与用户测试基线隔离**
- **📁 目录职责分离：`t/` 目录专供用户使用，`tests/integration/` 目录专供开发者集成测试**
- **🆕 任意目录支持：现已支持在任意目录运行测试，可使用 `t_for_test/` 等开发目录**
- **🔧 路径解析修复：修复了 FileExecutor 路径传递问题，现在支持完整的相对和绝对路径**

## Extension-based Testing Strategy

### 环境扩展名约定
- `result` (默认) - 用户测试基线
- `dev` - 开发环境测试基线  
- `integration` - 集成测试基线
- `ci` - CI/CD 环境基线
- `release` - 发布前验证基线
- `mysql8` / `mysql57` - 不同数据库版本基线

### 测试结果文件管理
```bash
r/
├── basic_example.result      # 用户默认基线
├── simple_test.dev          # 开发环境基线
├── variable_test.integration # 集成测试基线
├── concurrent_test.ci       # CI 环境基线
└── examples/
    ├── basic_example.result
    └── connection_example.result
```

## Local Test Environment

- 本地 MySQL 配置：用户名 root，密码 123456，端口 3306
- **开发测试推荐命令**: `cargo run -- --extension dev --host 127.0.0.1 --port 3306 --user root --passwd 123456`

## Design Memory & Notes

- **测试用例目录规范**:
  - 我期望单元测试或集成测试用到的 case 放在 t_for_test 中