# MySQL Test Runner (Rust)

一个用 Rust 重写的 MySQL 测试运行器，专注于 MySQL 数据库测试。

## 🎯 项目特色

- 🚀 **高性能**: 使用 Rust 编写，提供出色的性能和内存安全
- 🔧 **专业化**: 专注于 MySQL 测试，提供完整的 MySQL 功能支持
- 🔄 **完全兼容**: CLI 参数与原 Go 版本完全兼容
- 📊 **结果记录**: 支持测试结果文件生成和比对
- 🏗️ **模块化设计**: 清晰的代码架构，易于维护和扩展

## 🏗️ 技术架构

整体采用 **分层 + 插件式** 设计，核心组件如下：

1. **CLI 层** (`src/cli.rs`)
   • 基于 `clap` 生成命令行界面，负责参数解析、校验与帮助信息。

2. **测试执行引擎** (`src/tester/`)
   • `tester.rs` 负责调度 `.test` 文件的执行及结果比对。
   • `parser.rs` 将文件解析为 `Query` 列表，支持 40+ 指令/标签。
   • `query.rs` 定义 `QueryType` 枚举。
   • `connection_manager.rs` 运行时维护多数据库连接池，实现 `--connect/--connection/--disconnect`。
   • `database/` 抽象底层数据库，目前实现 MySQL 后端，设计支持未来扩展到 PostgreSQL 等。
   • `handlers/` 目录下每个文件对应一个标签命令，通过 `registry.rs` 注入，真正做到 **添加新指令零侵入**。

3. **工具层** (`src/util/`)  
   • 正则工具、时间测量等通用辅助函数。

4. **报告层** (`src/report/`)  
   • 目前输出纯文本及 `.result` 文件，后续将支持 JUnit XML 与 HTML 报告。

该架构带来的收益：
* **可扩展** — 新增指令只需"创建 handler + 注册一行"。
* **可测试** — 解析、执行、比对三层完全解耦，单元/集成测试覆盖率高。
* **可并发** — 设计之初即考虑并发场景，每个线程拥有独立连接。
* **可移植** — 数据库后端通过 `Database` trait 抽象，可拓展到 PostgreSQL 等。

## 🛠️ 安装和构建

### 前置要求

- Rust 1.70 或更高版本
- Cargo (通常随 Rust 一起安装)
- MySQL 服务器 (用于运行测试)

### 构建

```bash
git clone <repository-url>
cd dingo_test_runner
cargo build --release
```

### 运行

```bash
# 开发模式
cargo run -- [参数]

# 或使用构建的二进制文件
./target/release/dingo_test_runner [参数]
```

## 📚 使用指南

### 基本命令格式

```bash
dingo_test_runner [选项] <测试文件>
```

### MySQL 连接

```bash
# 连接本地 MySQL
cargo run -- --host localhost --port 3306 --user root --passwd secret test.sql

# 连接远程 MySQL
cargo run -- --host 192.168.1.100 --user testuser --passwd secret test.sql
```

### 主要命令行参数

| 参数 | 默认值 | 说明 |
|------|--------|------|
| `--host` | `127.0.0.1` | MySQL 服务器地址 |
| `--port` | `3306` | MySQL 服务器端口 |
| `--user` | `root` | 数据库用户名 |
| `--passwd` | `""` | 数据库密码 |
| `--log-level` | `error` | 日志级别：`error`, `warn`, `info`, `debug`, `trace` |
| `--record` | `false` | 是否记录测试输出到结果文件 |
| `--extension` | `result` | 结果文件扩展名 |
| `--retry-connection-count` | `120` | 数据库连接重试次数 |

## 📝 测试文件格式

### 支持的语法

#### 1. SQL 查询
```sql
-- 基本查询
SELECT 1;

-- 多行查询
SELECT * 
FROM users 
WHERE id = 1;
```

#### 2. 测试指令

| 指令 | 语法 | 说明 |
|------|------|------|
| 注释 | `# 注释内容` | 文件注释，不会执行 |
| 回显 | `--echo 文本内容` | 输出指定文本 |
| 睡眠 | `--sleep 2.5` | 暂停指定秒数 |
| 分隔符 | `--delimiter //` | 更改 SQL 分隔符 |
| 查询日志 | `--disable_query_log` <br> `--enable_query_log` | 控制查询语句输出 |
| 结果日志 | `--disable_result_log` <br> `--enable_result_log` | 控制查询结果输出 |
| 排序结果 | `--sorted_result` | 对查询结果进行排序 |

#### 3. 示例测试文件

```sql
# 用户管理测试
--echo 开始用户管理测试

# 创建测试表
CREATE TABLE users (
    id INTEGER PRIMARY KEY AUTO_INCREMENT,
    name VARCHAR(100) NOT NULL,
    email VARCHAR(100) UNIQUE
);

# 插入测试数据
INSERT INTO users (name, email) VALUES ('Alice', 'alice@example.com');
INSERT INTO users (name, email) VALUES ('Bob', 'bob@example.com');

# 查询所有用户
--echo 查询所有用户：
SELECT * FROM users;

# 查询特定用户
--echo 查询特定用户：
SELECT name FROM users WHERE id = 1;

--echo 测试完成
```

## 🔄 支持的查询类型

当前版本支持以下查询类型：

### ✅ 完全支持
- `Query` - 标准 SQL 查询
- `Exec` - SQL 执行语句
- `Comment` - 注释
- `Echo` - 文本输出
- `Sleep` - 延时执行
- `Delimiter` - 分隔符设置
- `DisableQueryLog` / `EnableQueryLog` - 查询日志控制
- `DisableResultLog` / `EnableResultLog` - 结果日志控制
- `SortedResult` - 结果排序
- `ReplaceRegex` - 正则替换
- `Error` - 错误处理
- `Connect` / `Connection` / `Disconnect` - 连接管理

### 🔄 部分支持
- `Admin` - 管理命令（基础支持）

### 📋 计划支持
- `ReplaceColumn` - 列替换
- `BeginConcurrent` / `EndConcurrent` - 并发执行
- `Source` - 文件包含
- 其他高级特性

## 🚀 使用示例

### 示例 1：基本 MySQL 测试

```bash
# 创建测试文件 demo.test
cat > demo.test << 'EOF'
--echo MySQL 演示开始

CREATE TABLE products (
    id INTEGER PRIMARY KEY AUTO_INCREMENT,
    name VARCHAR(100) NOT NULL,
    price DECIMAL(10,2)
);

INSERT INTO products (name, price) VALUES ('苹果', 3.50);
INSERT INTO products (name, price) VALUES ('香蕉', 2.80);

SELECT * FROM products;

--echo 演示结束
EOF

# 运行测试
cargo run -- --host 127.0.0.1 --user root --port 3307 --passwd 123123 --log-level info --record ddlbr
cargo run -- --host 127.0.0.1 --user root --port 3306 --passwd 123456 --log-level info --record ddlbr
```

### 示例 2：远程 MySQL 测试

```bash
# 连接到远程 MySQL 服务器
cargo run -- \
  --host mysql.example.com \
  --port 3306 \
  --user testuser \
  --passwd testpass \
  --log-level info \
  --record \
  demo.test
```

### 示例 3：批量测试

```bash
# 运行多个测试文件
cargo run -- --record test1.sql test2.sql test3.sql
```

## 📁 输出文件

### 结果文件

当使用 `--record` 参数时，测试结果会保存到 `r/` 目录：

```
r/
├── demo.result      # 测试结果文件
├── test1.result     # 其他测试结果
└── test2.result
```

### 结果文件格式

```
-- Query 1
SELECT * FROM users
1       Alice   alice@example.com
2       Bob     bob@example.com

-- Query 2  
SELECT COUNT(*) FROM users
2
```

## 🔧 高级配置

### 环境变量

可以通过环境变量控制日志级别：

```bash
export RUST_LOG=debug
cargo run -- demo.test
```

### 参数文件

对于复杂的配置，建议使用脚本文件：

```bash
#!/bin/bash
# run_tests.sh

cargo run -- \
  --host localhost \
  --user root \
  --passwd password \
  --log-level info \
  --record \
  --extension result \
  "$@"
```

## 🐛 故障排除

### 常见问题

1. **MySQL 连接失败**
   ```bash
   # 检查连接参数和网络
   --retry-connection-count 5
   ```

2. **结果文件权限**
   ```bash
   # 确保有写入权限
   mkdir -p r && chmod 755 r
   ```

### 调试模式

```bash
# 启用详细日志
cargo run -- --log-level debug demo.test

# 查看解析结果
RUST_LOG=trace cargo run -- demo.test
```
