# Exec 命令实现总结

## 概述

本文档总结了 `--exec` 命令在 `mysql-test-runner` Rust 版本中的实现。

## 功能特性

### 基本功能
- ✅ 执行 shell 命令并捕获标准输出
- ✅ 支持跨平台（Windows 使用 `cmd /C`，Unix 系统使用 `sh -c`）
- ✅ 将命令输出写入测试结果文件
- ✅ 支持 Record 模式和比对模式

### 错误处理
- ✅ 支持 `--error` 指令声明预期错误
- ✅ 正确处理命令退出码
- ✅ 区分预期错误和意外错误
- ✅ 支持 `--check-err` 严格错误检查模式

### 输出控制
- ✅ 遵循 `--enable_result_log` / `--disable_result_log` 设置
- ✅ 支持 `--replace_regex` 正则表达式替换
- ✅ 只捕获标准输出（stdout），不包含标准错误（stderr）

### 变量替换
- 🔄 TODO: 变量替换功能（等待变量系统完善）

## 实现架构

### 文件结构
```
src/tester/handlers/exec.rs     # exec 命令处理器
src/tester/registry.rs          # 命令注册表（已注册 exec）
src/tester/tester.rs           # 主执行器（已分离 Exec 和 Query）
tests/exec_integration_test.rs  # 集成测试
```

### 核心逻辑
1. **命令执行**: 使用 `std::process::Command` 执行 shell 命令
2. **输出捕获**: 捕获 stdout 并转换为 UTF-8 字符串
3. **错误处理**: 检查退出码并与预期错误进行匹配
4. **结果输出**: 根据模式写入缓冲区或进行比对

### 错误处理流程
```
命令执行
    ↓
检查退出码
    ↓
有预期错误？
    ├─ 是 → 错误匹配？
    │       ├─ 是 → 输出错误信息，继续
    │       └─ 否 → 根据 check_err 决定失败或警告
    └─ 否 → 退出码非0？
            ├─ 是 → 测试失败
            └─ 否 → 输出结果，继续
```

## 测试覆盖

### 集成测试
- ✅ 简单命令执行（echo）
- ✅ 预期错误处理（exit 1）
- ✅ 多行输出（printf）
- ✅ 无输出命令（true）
- ✅ Record 模式和比对模式
- ✅ 错误处理验证

### 测试用例
```bash
# 基本功能测试
cargo run -- --record simple_exec --passwd "123456" --reserve-schema
cargo run -- simple_exec --passwd "123456" --reserve-schema

# 复杂功能测试
cargo run -- --record exec_test --passwd "123456" --reserve-schema
cargo run -- exec_test --passwd "123456" --reserve-schema

# 集成测试
cargo test --test exec_integration_test
```

## 使用示例

### 基本用法
```sql
# 简单命令
--exec echo "Hello World"

# 多行输出
--exec printf "Line 1\nLine 2\nLine 3\n"

# 文件操作
--exec ls -la /tmp
```

### 错误处理
```sql
# 预期失败
--error 1
--exec exit 1

# 预期成功但可能失败
--error 0,1
--exec some_command_that_might_fail
```

### 与其他指令结合
```sql
# 使用正则替换
--replace_regex /[0-9]+/<NUMBER>/
--exec date +%s

# 禁用结果日志
--disable_result_log
--exec some_setup_command
--enable_result_log
```

## 兼容性

### 与 MySQL 官方 MTR 的兼容性
- ✅ 命令语法完全兼容
- ✅ 错误处理行为一致
- ✅ 输出格式一致
- ✅ 支持相同的修饰符

### 平台兼容性
- ✅ macOS (Darwin)
- ✅ Linux
- ✅ Windows（理论支持，使用 cmd /C）

## 性能考虑

- 每个 `--exec` 命令都会启动一个新的子进程
- 大量输出的命令可能占用较多内存
- 命令执行是同步的，会阻塞测试执行

## 未来改进

1. **变量替换**: 实现 mysqltest 变量替换功能
2. **异步执行**: 考虑支持异步命令执行
3. **输出流式处理**: 对于大输出命令的优化
4. **更多错误码支持**: 扩展错误码匹配逻辑

## 开发文档更新

已在 `DEVELOPMENT.md` 中标记 `--exec` 功能为已完成：
```markdown
#### Phase 8 – 功能补齐
- [x] 实现 `--exec` 功能
``` 