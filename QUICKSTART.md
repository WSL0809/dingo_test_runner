# 🚀 快速入门指南

5 分钟内开始使用 MySQL Test Runner (Rust)！

## ⚡ 一键开始

### 1. 克隆和构建

```bash
git clone <repository-url>
cd dingo_test_runner
cargo build --release
```

### 2. 创建第一个测试

```bash
cat > my_first_test.sql << 'EOF'
--echo 🎉 欢迎使用 MySQL Test Runner (Rust)

# 创建示例表
CREATE TABLE demo_table (
    id INTEGER PRIMARY KEY,
    message TEXT
);

# 插入数据
INSERT INTO demo_table (message) VALUES ('Hello, Rust!');
INSERT INTO demo_table (message) VALUES ('SQLite 调试很简单');

# 查询数据
SELECT * FROM demo_table;

--echo ✅ 测试完成！
EOF
```

### 3. 运行测试

```bash
# 使用 SQLite 模式（推荐新手）
cargo run -- --database-type sqlite --log-level info --record my_first_test.sql
```

### 4. 查看结果

```bash
# 查看生成的结果文件
cat r/my_first_test.result
```

输出示例：
```
🎉 欢迎使用 MySQL Test Runner (Rust)
-- Query 4
CREATE TABLE demo_table (
    id INTEGER PRIMARY KEY,
    message TEXT
)
-- Query 6
INSERT INTO demo_table (message) VALUES ('Hello, Rust!')
-- Query 7
INSERT INTO demo_table (message) VALUES ('SQLite 调试很简单')
-- Query 9
SELECT * FROM demo_table
1       Hello, Rust!
2       SQLite 调试很简单
✅ 测试完成！
```

## 🎯 常用命令

### SQLite 模式（本地调试）

```bash
# 基础用法
cargo run -- --database-type sqlite test.sql

# 详细日志 + 结果记录
cargo run -- --database-type sqlite --log-level info --record test.sql

# 使用文件数据库（持久化）
cargo run -- --database-type sqlite --sqlite-file my_test.db test.sql
```

### MySQL 模式（生产环境）

```bash
# 连接本地 MySQL
cargo run -- --database-type mysql --user root --passwd password test.sql

# 连接远程 MySQL
cargo run -- --database-type mysql --host 192.168.1.100 --user testuser --passwd secret test.sql
```

### 批量测试

```bash
# 运行多个测试文件
cargo run -- --database-type sqlite --record test1.sql test2.sql test3.sql
```

## 📝 测试文件语法速查

| 功能 | 语法 | 示例 |
|------|------|------|
| 注释 | `# 内容` | `# 这是注释` |
| 回显 | `--echo 文本` | `--echo 开始测试` |
| SQL | 直接写 | `SELECT 1;` |
| 睡眠 | `--sleep 秒数` | `--sleep 2.5` |
| 关闭查询日志 | `--disable_query_log` | |
| 开启查询日志 | `--enable_query_log` | |

## 🔧 常见使用场景

### 场景 1：开发新功能

```bash
# 使用内存数据库快速验证 SQL
cargo run -- --database-type sqlite --log-level debug my_feature.sql
```

### 场景 2：回归测试

```bash
# 生成基准结果
cargo run -- --database-type sqlite --record baseline.sql

# 验证修改后的结果
cargo run -- --database-type sqlite --record baseline.sql
# 然后比较 r/baseline.result 文件
```

### 场景 3：生产测试

```bash
# 连接生产数据库进行验证
cargo run -- \
  --database-type mysql \
  --host prod.mysql.com \
  --user readonly_user \
  --passwd secure_password \
  --record \
  production_tests.sql
```

## 🆘 遇到问题？

### 构建失败
```bash
# 确保 Rust 版本足够新
rustc --version  # 应该 >= 1.70

# 清理重试
cargo clean && cargo build
```

### 权限问题
```bash
# 确保结果目录可写
mkdir -p r && chmod 755 r
```

### 连接 MySQL 失败
```bash
# 增加重试次数
cargo run -- --retry-connection-count 5 --database-type mysql test.sql
```

## 📚 更多信息

- 📖 完整文档：[README.md](README.md)
- 🔧 开发文档：[DEVELOPMENT.md](DEVELOPMENT.md)
- 💡 示例文件：[example.test](example.test)

## 🎉 下一步

恭喜！你已经成功运行了第一个测试。现在你可以：

1. 📝 编写更复杂的测试文件
2. 🔄 尝试 MySQL 连接模式
3. 📊 使用 `--record` 模式生成基准结果
4. 🤝 向项目贡献代码

**开始你的测试之旅吧！** 🚀 