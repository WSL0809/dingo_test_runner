# Demo 测试用例

本目录包含了 MySQL 测试运行器的核心功能演示用例，涵盖了主要特性的完整测试。

## 测试用例列表

### 1. `variable_basic.test` - 变量系统测试
- **功能**: 测试 `--let` 变量赋值与展开
- **覆盖**: 变量定义、SQL中的变量展开、echo中的变量展开

### 2. `sorted_result.test` - 结果排序测试  
- **功能**: 测试 `--sorted_result` 确保查询结果的确定性
- **覆盖**: 无ORDER BY查询的结果排序

### 3. `error_handling.test` - 错误处理测试
- **功能**: 测试 `--error` 预期错误捕获
- **覆盖**: 错误码映射、预期错误验证

### 4. `exec_replace_regex.test` - 外部命令与正则替换测试
- **功能**: 测试 `--exec` 和 `--replace_regex` 组合使用
- **覆盖**: 外部shell命令执行、结果正则表达式替换

### 5. `connection_multi.test` - 多连接管理测试
- **功能**: 测试多数据库连接管理
- **覆盖**: `--connect`、`--connection`、`--disconnect`、变量展开在连接参数中的应用

## 运行方式

### 录制模式（生成期望结果）
```bash
cargo run -- --passwd 123456 --record demo_tests/variable_basic demo_tests/sorted_result demo_tests/error_handling demo_tests/exec_replace_regex demo_tests/connection_multi
```

### 比对模式（验证测试结果）
```bash
cargo run -- --passwd 123456 demo_tests/variable_basic demo_tests/sorted_result demo_tests/error_handling demo_tests/exec_replace_regex demo_tests/connection_multi
```

### 单个测试
```bash
# 录制单个测试
cargo run -- --passwd 123456 --record demo_tests/connection_multi

# 比对单个测试
cargo run -- --passwd 123456 demo_tests/connection_multi
```

## 功能覆盖

这5个测试用例涵盖了以下核心功能：
- ✅ 变量系统 (`--let`, 变量展开)
- ✅ 查询结果处理 (`--sorted_result`)
- ✅ 错误处理 (`--error`)
- ✅ 外部命令执行 (`--exec`)
- ✅ 结果正则替换 (`--replace_regex`)
- ✅ 多连接管理 (`--connect`, `--connection`, `--disconnect`)
- ✅ 数据库操作 (CREATE/DROP DATABASE, CREATE TABLE, INSERT, SELECT)
- ✅ 日志控制 (`--echo`)

## 注意事项

- 确保MySQL服务运行在 `127.0.0.1:3306`
- 使用正确的用户名和密码（示例中使用 `root/123456`）
- 测试会创建和删除临时数据库，请确保有相应权限 