# Demo 测试用例

本目录包含了 MySQL 测试运行器的核心功能演示用例，涵盖了主要特性的完整测试。

## 测试用例列表

### 1. `variable_basic.test` - 变量系统基础测试
- **功能**: 测试 `--let` 变量赋值与展开
- **覆盖**: 变量定义、SQL中的变量展开、echo中的变量展开、SQL反引号表达式求值

### 2. `variable_expression.test` - 变量表达式求值测试 ⭐ **新增**
- **功能**: 测试 `let` 语句的表达式求值能力（乐观求值策略）
- **覆盖**: 
  - 算术运算（+, -, *, /）
  - 逻辑运算（>, <, ==, !=, &&, ||）
  - 复杂表达式嵌套
  - SQL反引号表达式与算术混合运算
  - 字面值回退机制
  - 灵活的 `let` 语法格式（支持不带 `--` 前缀）

### 3. `let_expression_showcase.test` - Let 表达式功能展示 ⭐ **新增**
- **功能**: 简洁展示 `let` 表达式求值的核心功能
- **覆盖**: 
  - 基础算术运算演示
  - 复杂表达式嵌套演示
  - 逻辑运算演示
  - 灵活语法格式演示（紧凑、空格、大小写）
  - 字面值回退演示

### 4. `sorted_result.test` - 结果排序测试  
- **功能**: 测试 `--sorted_result` 确保查询结果的确定性
- **覆盖**: 无ORDER BY查询的结果排序

### 5. `error_handling.test` - 错误处理测试
- **功能**: 测试 `--error` 预期错误捕获
- **覆盖**: 错误码映射、预期错误验证

### 6. `exec_replace_regex.test` - 外部命令与正则替换测试
- **功能**: 测试 `--exec` 和 `--replace_regex` 组合使用
- **覆盖**: 外部shell命令执行、结果正则表达式替换

### 7. `connection_multi.test` - 多连接管理测试
- **功能**: 测试多数据库连接管理
- **覆盖**: `--connect`、`--connection`、`--disconnect`、变量展开在连接参数中的应用

### 8. `let_with_control_flow.test` - Let 与控制流综合测试 ⭐ **新增**
- **功能**: 测试 `let` 表达式求值在 `if`/`while` 循环中的复杂交互
- **覆盖**: 
  - 在循环中通过 `let` 和 SQL 查询更新变量
  - 基于 `let` 计算结果的条件判断
  - 嵌套循环中 `let` 变量的更新与作用域
  - 模拟 `if-else` 逻辑
  - 累加计算

### 9. `concurrent_basic.test` - 并发查询基础测试 ⭐ **新增**
- **功能**: 验证 `--begin_concurrent` / `--end_concurrent` 能够并发执行多个简单查询
- **覆盖**:
  - 多个 SELECT 并发执行
  - 与 `--error` 组合使用捕获预期错误

### 10. `concurrent_error_handling.test` - 并发错误处理测试 ⭐ **新增**
- **功能**: 在并发块中混合成功查询与预期失败查询，验证错误捕获与输出拼接顺序
- **覆盖**:
  - 并发块中 `--error` 与普通查询交替出现
  - 不同 MySQL 错误码捕获

### 11. `concurrent_mixed_control.test` - 并发块与日志/排序修饰符交互测试 ⭐ **新增**
- **功能**: 验证在并发块中使用 `--sorted_result`、日志开关等一次性修饰符的行为
- **覆盖**:
  - 在并发块前设置 `--sorted_result` 并确保结果排序
  - 在并发块内外切换查询/结果日志开关

### 12. `concurrent_advanced.test` - 高级并发功能综合测试 ⭐ **新增**
- **功能**: 演示并发块与变量系统、复杂查询的综合应用
- **覆盖**:
  - 并发块中使用变量展开
  - 多个复杂 SELECT 查询并发执行
  - 并发块中混合成功查询与多种错误类型

## 运行方式

### 录制模式（生成期望结果）
```bash
cargo run -- --passwd 123456 --record demo_tests/variable_basic demo_tests/variable_expression demo_tests/let_expression_showcase demo_tests/sorted_result demo_tests/error_handling demo_tests/exec_replace_regex demo_tests/connection_multi demo_tests/let_with_control_flow demo_tests/concurrent_basic demo_tests/concurrent_error_handling demo_tests/concurrent_mixed_control demo_tests/concurrent_advanced
```

### 比对模式（验证测试结果）
```bash
cargo run -- --passwd 123456 demo_tests/variable_basic demo_tests/variable_expression demo_tests/let_expression_showcase demo_tests/sorted_result demo_tests/error_handling demo_tests/exec_replace_regex demo_tests/connection_multi demo_tests/let_with_control_flow demo_tests/concurrent_basic demo_tests/concurrent_error_handling demo_tests/concurrent_mixed_control demo_tests/concurrent_advanced
```

### 单个测试
```bash
# 录制单个测试
cargo run -- --passwd 123456 --record demo_tests/variable_expression

# 比对单个测试
cargo run -- --passwd 123456 demo_tests/variable_expression
```

## 功能覆盖

这12个测试用例涵盖了以下核心功能：
- ✅ **变量系统** (`--let`, `let`, 变量展开)
- ✅ **表达式求值** ⭐ **新增强化**
  - 算术表达式 (`$a + $b`, `($a + $b) * 2`)
  - 逻辑表达式 (`$a > $b`, `$a == $b`, `$a > 0 && $b > 0`)
  - SQL反引号表达式 (`` `SELECT COUNT(*) FROM table` ``)
  - 混合表达式 (`$count * 10 + $max_value`)
  - 乐观求值策略（表达式求值失败时回退到字面值）
- ✅ **灵活语法支持** (`let` 不需要 `--` 前缀，空格容错)
- ✅ 查询结果处理 (`--sorted_result`)
- ✅ 错误处理 (`--error`)
- ✅ 外部命令执行 (`--exec`)
- ✅ 结果正则替换 (`--replace_regex`)
- ✅ 多连接管理 (`--connect`, `--connection`, `--disconnect`)
- ✅ **控制流语句** (`if`, `while`, 嵌套控制流)
- ✅ 数据库操作 (CREATE/DROP DATABASE, CREATE TABLE, INSERT, SELECT, UPDATE)
- ✅ 日志控制 (`--echo`)
- ✅ **并发执行** (`--begin_concurrent` / `--end_concurrent`) ⭐ **新增强化**

## 新功能亮点 ⭐

### Let 语句表达式求值
现在 `let` 语句支持强大的表达式求值功能：

```sql
# 算术运算
let $a = 5
let $b = 3
let $sum = $a + $b          # 结果: 8
let $complex = ($a + $b) * 2 # 结果: 16

# 逻辑运算
let $greater = $a > $b       # 结果: 1 (true)
let $and_result = $a > 0 && $b > 0  # 结果: 1

# SQL 表达式
let $count = `SELECT COUNT(*) FROM test_table`  # 执行SQL并获取结果
let $calc = $count * 10 + 5  # 混合运算

# 字面值回退
let $text = hello world with spaces  # 无法求值时保持字面值
```

### 灵活语法支持
- 支持 `let $var = value`（不需要 `--` 前缀）
- 支持 `--let $var = value`（传统格式）
- 空格容错：`let$var=value` 也能正确解析

## 注意事项

- 确保MySQL服务运行在 `127.0.0.1:3306`
- 使用正确的用户名和密码（示例中使用 `root/123456`）
- 测试会创建和删除临时数据库，请确保有相应权限
- 新的表达式求值功能与 MySQL mysqltest 完全兼容 