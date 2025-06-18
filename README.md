[![Ask DeepWiki](https://deepwiki.com/badge.svg)](https://deepwiki.com/WSL0809/dingo_test_runner)
# MySQL Test Runner (Rust)

一个兼容 MySQL 官方测试格式的测试运行器，用 Rust 重写自 Go 版本，保持功能等价性。

## 项目状态

当前版本支持 MySQL 测试文件的解析、执行和结果比对，已实现 48 种查询类型和主要功能模块。

## 核心功能

- **测试执行**: 支持单个和批量测试文件执行
- **结果比对**: Record 模式生成基准结果，Comparison 模式进行逐行比对  
- **并发支持**: 实现 `--BEGIN_CONCURRENT` / `--END_CONCURRENT` 并发块
- **多数据库**: 支持 MySQL 和 SQLite（用于本地调试）
- **连接管理**: 支持多连接池和连接切换
- **变量系统**: 支持 `--let` 变量定义和展开
- **控制流**: 支持 `if` / `while` 条件和循环语句
- **报告输出**: 支持 Terminal、HTML、XUnit XML、Allure 多种格式
- **邮件通知**: 支持 SMTP 邮件发送测试报告

## 安装构建

**环境要求**:
- Rust ≥ 1.78
- 可选：MySQL 8.0

**构建**:
```bash
git clone <repository-url>
cd dingo_test_runner
cargo build --release
```

## 基本使用

### 运行单个测试

```bash
# 按测试名运行
cargo run -- basic

# 运行 .test 文件
cargo run -- basic.test

# 指定数据库连接
cargo run -- --host 127.0.0.1 --port 3306 --user root --passwd password basic
```

### 批量运行

```bash
# 运行所有测试
cargo run -- --all

# 运行目录下所有测试
cargo run -- t/demo_tests

# 混合格式
cargo run -- basic advanced.test t/regression
```

### 生成结果文件

```bash
# Record 模式：生成 .result 文件
cargo run -- --record basic

# Comparison 模式：与已有 .result 文件比对（默认）
cargo run -- basic
```

## 数据库连接

**MySQL 连接**:
```bash
cargo run -- --host 127.0.0.1 --port 3306 --user root --passwd password basic
```

**SQLite 回落**（无需配置 MySQL）:
```bash
cargo run -- basic
```

## 报告生成

### JUnit XML 报告

```bash
# 生成 JUnit XML（用于 CI/CD）
cargo run -- --all --xunit-file test_report.xml
```

### 多格式报告

```bash
# 彩色终端输出（默认）
cargo run -- basic

# HTML 报告
cargo run -- basic --report-format html --xunit-file report.xml

# 纯文本报告
cargo run -- basic --report-format plain

# Allure 报告
cargo run -- basic --allure-dir allure-results
```

### 邮件通知

```bash
# 启用邮件功能需要 feature flag
cargo build --features email

# 发送测试报告邮件
cargo run --features email -- --all \
  --email-smtp-server smtp.gmail.com \
  --email-smtp-port 587 \
  --email-username user@gmail.com \
  --email-password app-password \
  --email-from user@gmail.com \
  --email-to team@company.com \
  --email-subject "Test Report"
```

## 测试文件格式

### 基本语法

```sql
# 注释
--echo 输出文本

# SQL 查询
SELECT 1;

# 多行查询  
SELECT * 
FROM users 
WHERE id = 1;

# 更改分隔符
--delimiter //
CREATE PROCEDURE test() BEGIN SELECT 1; END //
--delimiter ;
```

### 支持的指令

| 指令 | 说明 |
|------|------|
| `--echo <text>` | 输出文本 |
| `--sleep <seconds>` | 暂停执行 |
| `--error <code>` | 预期错误码 |
| `--sorted_result` | 结果排序 |
| `--replace_regex /<regex>/<replacement>/` | 正则替换 |
| `--let $var = value` | 变量定义 |
| `--exec <command>` | 执行系统命令 |
| `--connect (name,host,user,password,db)` | 连接管理 |
| `--disable_query_log` / `--enable_query_log` | 查询日志控制 |
| `--disable_result_log` / `--enable_result_log` | 结果日志控制 |

### 控制流语句

```sql
# if 语句（支持 end 或花括号结尾）
let $count = 5
if ($count > 0)
  SELECT 'positive';
end

# while 循环
let $i = 0  
while ($i < 3)
  SELECT $i;
  let $i = $i + 1
end
```

### 并发执行

```sql
--BEGIN_CONCURRENT
SELECT 1;
SELECT 2; 
SELECT 3;
--END_CONCURRENT
```

## 完整使用示例

### 示例 1：基础 MySQL 测试

```sql
# basic_test.test
--echo 开始基础测试

# 创建测试表
CREATE TABLE users (
    id INTEGER PRIMARY KEY AUTO_INCREMENT,
    name VARCHAR(100) NOT NULL,
    email VARCHAR(100) UNIQUE
);

# 插入测试数据
INSERT INTO users (name, email) VALUES ('Alice', 'alice@example.com');
INSERT INTO users (name, email) VALUES ('Bob', 'bob@example.com');

# 查询测试
--echo 查询所有用户：
SELECT * FROM users;

--echo 查询用户数量：
SELECT COUNT(*) FROM users;

# 清理
DROP TABLE users;
--echo 基础测试完成
```

**运行命令**：
```bash
# Record 模式生成期望结果
cargo run -- --record basic_test

# 比对模式验证结果
cargo run -- basic_test
```

### 示例 2：变量和表达式测试

```sql
# variable_test.test
--echo 测试变量系统

# 定义变量（注意 let 可以省略 --）
let $user_count = 5
let $table_name = test_users  
let $greeting = hello world with spaces

# 使用变量
--echo 用户数量: $user_count
--echo 表名: $table_name
--echo 问候语: $greeting

# 表达式计算
let $result = $user_count * 2
let $sum = $user_count + 10
let $condition = $user_count > 3
--echo 计算结果: $result
--echo 总和: $sum
--echo 条件判断: $condition

# SQL反引号查询
let $row_count = `SELECT COUNT(*) FROM information_schema.tables`
--echo 系统表数量: $row_count

# 在SQL中使用变量
CREATE TABLE $table_name (id INT, name VARCHAR(50));
INSERT INTO $table_name VALUES (1, 'User1'), (2, 'User2');
SELECT * FROM $table_name ORDER BY id;
DROP TABLE $table_name;
```

**运行命令**：
```bash
cargo run -- --record variable_test
```

### 示例 3：控制流测试

```sql
# control_flow_test.test
--echo 测试控制流

# 准备测试数据
CREATE TABLE test_items (id INT, value INT);
INSERT INTO test_items VALUES (1, 10), (2, 5), (3, 15);

# if 语句测试
let $count = 10
if ($count > 5)
  --echo 数量大于5
  SELECT 'Large count' as result;
end

if ($count < 5)
  --echo 数量小于5
end

if ($count >= 5)
  --echo 数量不小于5
end

# while 循环测试
let $i = 1
--echo 开始循环处理
while ($i <= 3)
  let $current_value = `SELECT value FROM test_items WHERE id = $i`
  --echo 处理项目 $i, 值: $current_value
  
  if ($current_value > 8)
    --echo "  值较大，进行特殊处理"
    UPDATE test_items SET value = value + 5 WHERE id = $i;
  end
  
  let $i = $i + 1
end
--echo 循环结束

# 查看处理结果
SELECT * FROM test_items ORDER BY id;

# 清理
DROP TABLE test_items;
```

**运行命令**：
```bash
cargo run -- --record control_flow_test
```

### 示例 4：并发执行测试

```sql
# concurrent_test.test
--echo 并发执行测试

# 准备测试数据
CREATE TABLE concurrent_test (id INT, value VARCHAR(50));

--echo 开始并发执行
--BEGIN_CONCURRENT
INSERT INTO concurrent_test VALUES (1, 'Thread1');
INSERT INTO concurrent_test VALUES (2, 'Thread2');
INSERT INTO concurrent_test VALUES (3, 'Thread3');
SELECT COUNT(*) FROM concurrent_test;
SELECT * FROM concurrent_test ORDER BY id;
--END_CONCURRENT
--echo 并发执行完成

# 验证结果
--sorted_result
SELECT * FROM concurrent_test;

# 清理
DROP TABLE concurrent_test;
```

**运行命令**：
```bash
cargo run -- --record concurrent_test
```

### 示例 5：错误处理和正则替换

```sql
# error_handling_test.test
--echo 错误处理和正则替换测试

# 预期错误测试 - 使用错误码名称
CREATE TABLE dup_test (id INT PRIMARY KEY, val INT);
INSERT INTO dup_test VALUES (1, 100);

--echo 下面将插入重复主键，期待 ER_DUP_ENTRY 错误
--error ER_DUP_ENTRY
INSERT INTO dup_test VALUES (1, 200);

# 正则替换测试 - 替换时间戳
--echo 生成带时间戳的输出，然后用正则替换
--replace_regex /[0-9]{4}-[0-9]{2}-[0-9]{2} [0-9]{2}:[0-9]{2}:[0-9]{2}/<TIMESTAMP>/
--exec date '+%F %T'

# 排序结果测试
CREATE TABLE sort_test (id INT, name VARCHAR(50));
INSERT INTO sort_test VALUES (3, 'Charlie'), (1, 'Alice'), (2, 'Bob');

--echo 无序查询结果：
SELECT * FROM sort_test;

--echo 排序后的结果：
--sorted_result
SELECT * FROM sort_test;

# 清理
DROP TABLE dup_test;
DROP TABLE sort_test;
```

**运行命令**：
```bash
cargo run -- --record error_handling_test
```

### 示例 6：连接管理测试

```sql
# connection_test.test
--echo 连接管理测试

# 准备测试数据库
let $db1 = conn_test_db1
let $db2 = conn_test_db2
DROP DATABASE IF EXISTS $db1;
DROP DATABASE IF EXISTS $db2;
CREATE DATABASE $db1;
CREATE DATABASE $db2;

# 创建新连接（语法：host,user,password,database）
--connect (conn1,127.0.0.1,root,123456,$db1)
--connect (conn2,127.0.0.1,root,123456,$db2)

# 在连接1中操作
--connection conn1
CREATE TABLE t1 (id INT, name VARCHAR(50));
INSERT INTO t1 VALUES (1, 'Connection1');

# 在连接2中操作  
--connection conn2
CREATE TABLE t2 (id INT, name VARCHAR(50));
INSERT INTO t2 VALUES (2, 'Connection2');

# 验证连接隔离
--echo 在连接1中查询：
--connection conn1
SELECT * FROM t1;

--echo 在连接2中查询：
--connection conn2
SELECT * FROM t2;

# 断开连接
--disconnect conn1
--disconnect conn2
--connection default

# 清理
DROP DATABASE $db1;
DROP DATABASE $db2;
```

**运行命令**：
```bash
cargo run -- --record connection_test
```

### 示例 7：系统命令执行

```sql
# exec_test.test
--echo 系统命令执行测试

# 执行系统命令
--exec echo "Hello from system command"
--exec date
--exec ls -la | head -5

# 在SQL中使用命令结果
CREATE TABLE exec_test (id INT, info VARCHAR(100));
INSERT INTO exec_test VALUES (1, 'System info');
SELECT * FROM exec_test;
DROP TABLE exec_test;
```

**运行命令**：
```bash
cargo run -- --record exec_test
```

### 示例 8：综合测试

```sql
# comprehensive_test.test
--echo 综合功能测试

# 变量定义
let $db_name = comprehensive_test_db
let $table_prefix = tbl_
let $user_count = 3

--echo 使用数据库: $db_name
--echo 表前缀: $table_prefix  
--echo 用户数量: $user_count

# 准备测试环境
DROP DATABASE IF EXISTS $db_name;
CREATE DATABASE $db_name;
USE $db_name;

# 创建测试表
CREATE TABLE ${table_prefix}users (
    id INT PRIMARY KEY,
    name VARCHAR(50),
    status VARCHAR(20) DEFAULT 'active'
);

# 循环插入数据
let $i = 1
while ($i <= $user_count)
  INSERT INTO ${table_prefix}users (id, name) VALUES ($i, CONCAT('User', $i));
  let $i = $i + 1
end

# 条件查询
let $actual_count = `SELECT COUNT(*) FROM ${table_prefix}users`
if ($actual_count > 2)
  --echo 用户数量超过2，显示所有用户
  --sorted_result
  SELECT id, name FROM ${table_prefix}users;
end

if ($actual_count <= 2)
  --echo 用户数量较少
  SELECT COUNT(*) as user_count FROM ${table_prefix}users;
end

# 并发操作测试
--echo 并发更新测试
--BEGIN_CONCURRENT
UPDATE ${table_prefix}users SET status = 'updated' WHERE id = 1;
UPDATE ${table_prefix}users SET status = 'updated' WHERE id = 2;
UPDATE ${table_prefix}users SET status = 'updated' WHERE id = 3;
--END_CONCURRENT

# 验证更新结果（使用正则替换隐藏具体时间）
--replace_regex /updated/[PROCESSED]/
SELECT id, name, status FROM ${table_prefix}users ORDER BY id;

# 清理
DROP DATABASE $db_name;
--echo 综合测试完成
```

**运行命令**：
```bash
# 记录模式
cargo run -- --record comprehensive_test

# 生成HTML报告
cargo run -- comprehensive_test --report-format html --xunit-file report.xml

# 生成Allure报告
cargo run -- comprehensive_test --allure-dir allure-results

# 发送邮件报告
cargo run --features email -- comprehensive_test \
  --email-smtp-server smtp.gmail.com \
  --email-smtp-port 587 \
  --email-username test@example.com \
  --email-password app-password \
  --email-from test@example.com \
  --email-to team@example.com \
  --email-subject "Comprehensive Test Report"
```

## 命令行参数

### 数据库连接
- `--host <host>`: 数据库主机 (默认: 127.0.0.1)
- `--port <port>`: 数据库端口 (默认: 3306)  
- `--user <user>`: 用户名 (默认: root)
- `--passwd <password>`: 密码 (默认: "")

### 测试选项
- `--record`: 启用 Record 模式
- `--all`: 运行所有测试
- `--extension <ext>`: 结果文件扩展名 (默认: result)
- `--log-level <level>`: 日志级别 (error/warn/info/debug/trace)

### 报告输出
- `--xunit-file <file>`: JUnit XML 报告文件
- `--report-format <format>`: 报告格式 (terminal/html/plain/xunit)
- `--allure-dir <dir>`: Allure 报告目录

### 邮件配置
- `--email-smtp-server <server>`: SMTP 服务器
- `--email-smtp-port <port>`: SMTP 端口
- `--email-username <user>`: 邮箱用户名
- `--email-password <password>`: 邮箱密码
- `--email-from <email>`: 发件人邮箱
- `--email-to <emails>`: 收件人邮箱（逗号分隔）
- `--email-subject <subject>`: 邮件主题
- `--email-attach-xml`: 附带 XML 报告

## 项目结构

```
src/
├── main.rs              # 程序入口
├── cli.rs               # 命令行参数解析
├── loader.rs            # 测试文件加载器
├── tester/              # 核心测试模块
│   ├── tester.rs        # 测试执行器
│   ├── parser.rs        # 手写解析器
│   ├── pest_parser.rs   # Pest 解析器
│   ├── query.rs         # 查询类型定义
│   ├── database.rs      # 数据库抽象层
│   ├── connection_manager.rs # 连接管理
│   ├── variables.rs     # 变量系统
│   ├── expression.rs    # 表达式求值
│   └── handlers/        # 命令处理器
├── report/              # 报告生成
│   ├── mod.rs           # 报告架构
│   ├── summary.rs       # 终端输出
│   ├── html.rs          # HTML 报告
│   ├── xunit.rs         # XML 报告
│   └── allure.rs        # Allure 报告
├── util/                # 工具模块
└── stub/                # 桩代码
    └── email.rs         # 邮件通知
```

## 开发状态

- **解析层**: 完成 (支持双解析器架构)
- **执行引擎**: 完成 (串行+并发)
- **数据库支持**: 完成 (MySQL)
- **报告系统**: 完成 (多格式输出)
- **邮件通知**: 完成 (HTML + 纯文本)
- **变量系统**: 完成 (let 语句 + 展开)
- **控制流**: 完成 (if/while + 嵌套)
- **连接管理**: 完成 (多连接池)

当前版本支持大部分 MySQL 官方测试格式，与原 Go 版本功能基本等价。

## 批量测试和CI/CD集成

### 批量执行所有测试

```bash
# 运行所有测试并生成完整报告
cargo run -- --all --xunit-file full_report.xml --report-format html

# 运行特定目录的测试
cargo run -- t/demo_tests --allure-dir allure-results

# 运行多个测试文件
cargo run -- basic_test variable_test control_flow_test
```

### CI/CD 集成示例

**GitHub Actions 配置** (`.github/workflows/test.yml`):
```yaml
name: MySQL Tests
on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    services:
      mysql:
        image: mysql:8.0
        env:
          MYSQL_ROOT_PASSWORD: testpass
          MYSQL_DATABASE: testdb
        options: >-
          --health-cmd="mysqladmin ping"
          --health-interval=10s
          --health-timeout=5s
          --health-retries=3

    steps:
    - uses: actions/checkout@v3
    - uses: actions-rs/toolchain@v1
      with:
        toolchain: stable
    
    - name: Build
      run: cargo build --release
      
    - name: Run Tests
      run: |
        cargo run -- --all \
          --host 127.0.0.1 \
          --port 3306 \
          --user root \
          --passwd testpass \
          --xunit-file test_results.xml \
          --allure-dir allure-results
          
    - name: Publish Test Results
      uses: dorny/test-reporter@v1
      if: success() || failure()
      with:
        name: MySQL Tests
        path: test_results.xml
        reporter: java-junit
        
    - name: Upload Allure Results
      uses: actions/upload-artifact@v3
      with:
        name: allure-results
        path: allure-results/
```

**Jenkins Pipeline 示例**:
```groovy
pipeline {
    agent any
    
    environment {
        MYSQL_HOST = '127.0.0.1'
        MYSQL_USER = 'root'
        MYSQL_PASS = credentials('mysql-password')
    }
    
    stages {
        stage('Build') {
            steps {
                sh 'cargo build --release'
            }
        }
        
        stage('Test') {
            steps {
                sh '''
                    cargo run -- --all \
                        --host ${MYSQL_HOST} \
                        --user ${MYSQL_USER} \
                        --passwd ${MYSQL_PASS} \
                        --xunit-file junit_results.xml \
                        --allure-dir allure-results
                '''
            }
            post {
                always {
                    junit 'junit_results.xml'
                    allure includeProperties: false, 
                           jdk: '', 
                           results: [[path: 'allure-results']]
                }
            }
        }
        
        stage('Email Report') {
            when { 
                anyOf { 
                    branch 'main'
                    expression { currentBuild.result == 'FAILURE' }
                } 
            }
            steps {
                sh '''
                    cargo run --features email -- --all \
                        --host ${MYSQL_HOST} \
                        --user ${MYSQL_USER} \
                        --passwd ${MYSQL_PASS} \
                        --email-smtp-server smtp.company.com \
                        --email-smtp-port 587 \
                        --email-username ${EMAIL_USER} \
                        --email-password ${EMAIL_PASS} \
                        --email-from testbot@company.com \
                        --email-to dev-team@company.com \
                        --email-subject "MySQL Test Report - Build ${BUILD_NUMBER}" \
                        --email-attach-xml
                '''
            }
        }
    }
}
```

### Docker 集成

**Dockerfile**:
```dockerfile
FROM rust:1.78 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM ubuntu:22.04
RUN apt-get update && apt-get install -y \
    mysql-client \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/dingo_test_runner /usr/local/bin/
COPY t/ /app/t/
COPY r/ /app/r/
WORKDIR /app

ENTRYPOINT ["dingo_test_runner"]
```

**docker-compose.yml**:
```yaml
version: '3.8'
services:
  mysql:
    image: mysql:8.0
    environment:
      MYSQL_ROOT_PASSWORD: testpass
      MYSQL_DATABASE: testdb
    ports:
      - "3306:3306"
    healthcheck:
      test: ["CMD", "mysqladmin", "ping", "-h", "localhost"]
      timeout: 20s
      retries: 10

  test-runner:
    build: .
    depends_on:
      mysql:
        condition: service_healthy
    volumes:
      - ./reports:/app/reports
    command: >
      --all
      --host mysql
      --port 3306
      --user root
      --passwd testpass
      --xunit-file /app/reports/test_results.xml
      --allure-dir /app/reports/allure-results
```

**运行命令**:
```bash
# 启动完整测试环境
docker-compose up --build

# 仅运行特定测试
docker-compose run test-runner basic_test --record
```

