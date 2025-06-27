# 文件级并发执行功能

本文档介绍 dingo_test_runner 的文件级并发执行功能。

## 功能概述

新增的文件级并发执行功能允许多个测试文件同时运行，大幅提升大型测试套件的执行效率。该功能具有以下特点：

- **完全向后兼容**：默认为串行执行，现有用法不受影响
- **数据库隔离**：每个并发测试文件使用独立的临时数据库
- **资源管理**：智能连接池管理和自动清理
- **进度监控**：实时显示执行进度和统计信息

## 使用方法

### 串行执行（默认模式）
```bash
# 默认串行执行，与之前行为完全一致
cargo run -- test1 test2 test3

# 等价于显式指定 --parallel 1
cargo run -- --parallel 1 test1 test2 test3
```

### 并发执行
```bash
# 使用 4 个并发线程执行测试文件
cargo run -- --parallel 4 test1 test2 test3 test4

# 执行所有测试，使用 8 个并发线程
cargo run -- --parallel 8 --all

# 自定义连接池大小（可选）
cargo run -- --parallel 4 --max-connections 16 test_suite/
```

## 数据库隔离机制

### 自动隔离
每个并发测试文件会自动获得独立的数据库实例：

- **数据库命名格式**：`test_{sanitized_test_name}_{thread_id}_{timestamp}_{process_id}`
- **自动创建**：测试开始前自动创建临时数据库
- **自动清理**：测试完成后自动删除临时数据库
- **无冲突**：不同线程的测试绝不会相互干扰

### 示例
```
并发执行时的数据库分配：
┌─────────────────┬──────────────────────────────────────┐
│ 测试文件        │ 临时数据库名                          │
├─────────────────┼──────────────────────────────────────┤
│ user_test.test  │ test_user_test_0_1703123456789_12345 │
│ order_test.test │ test_order_test_1_1703123456790_12345│
│ api_test.test   │ test_api_test_2_1703123456791_12345  │
└─────────────────┴──────────────────────────────────────┘
```

## 性能对比

### 执行时间对比（示例）
```
测试套件：20 个测试文件，每个文件平均执行时间 30 秒

串行执行：  20 × 30s = 600 秒 (10 分钟)
4 线程并发：20 ÷ 4 × 30s = 150 秒 (2.5 分钟)
8 线程并发：20 ÷ 8 × 30s = 75 秒 (1.25 分钟)

性能提升：4 倍到 8 倍加速
```

### 资源使用
- **CPU**: 充分利用多核处理器
- **内存**: 每个线程独立内存空间，可控制
- **数据库连接**: 智能连接池，避免连接耗尽

## 参数说明

### --parallel
指定并发执行的线程数：
- **默认值**: 1（串行执行）
- **建议值**: CPU 核心数的 1-2 倍
- **最大值**: 建议不超过数据库最大连接数

### --max-connections
指定数据库连接池大小：
- **默认值**: 0（自动计算，通常为 parallel × 2）
- **用途**: 限制总连接数，避免数据库过载
- **建议**: 根据数据库服务器配置调整

## 最佳实践

### 1. 并发度选择
```bash
# 对于 CPU 密集型测试
cargo run -- --parallel $(nproc) test_suite/

# 对于 I/O 密集型测试
cargo run -- --parallel $(($(nproc) * 2)) test_suite/

# 保守选择（避免资源争用）
cargo run -- --parallel 4 test_suite/
```

### 2. 大型测试套件
```bash
# 分批执行，避免一次性创建过多数据库
cargo run -- --parallel 8 --max-connections 32 large_test_suite/
```

### 3. CI/CD 集成
```bash
# 在 CI 环境中使用固定并发度
cargo run -- --parallel 4 --report-format xunit --xunit-file results.xml --all
```

## 兼容性保证

### 向后兼容
- **现有脚本**: 无需修改，自动使用串行模式
- **测试文件**: 格式完全兼容，无需更改
- **命令行**: 所有现有参数保持不变

### 功能兼容
- **查询级并发**: `--BEGIN_CONCURRENT` / `--END_CONCURRENT` 照常工作
- **连接管理**: `--connect` / `--disconnect` 在每个文件内正常使用
- **错误处理**: `--error` 指令完全兼容
- **变量系统**: `--let` 变量在文件级别隔离

## 故障排除

### 常见问题

1. **数据库连接不足**
   ```
   错误: Failed to get database connection
   解决: 减少 --parallel 值或增加 --max-connections
   ```

2. **临时数据库未清理**
   ```
   原因: 程序异常退出
   解决: 手动清理 test_* 数据库
   ```

3. **性能不如预期**
   ```
   检查: 数据库服务器 CPU/内存/磁盘 I/O 是否为瓶颈
   调整: 减少并发度或优化测试文件
   ```

### 调试模式
```bash
# 启用详细日志
RUST_LOG=debug cargo run -- --parallel 4 test_suite/

# 监控数据库连接
RUST_LOG=dingo_test_runner::tester::database=trace cargo run -- --parallel 4 test_suite/
```

## 实现细节

### 架构设计
```
文件级并发执行器 (FileExecutor)
├── 串行执行模式 (execute_serial)    # 向后兼容
├── 并发执行模式 (execute_parallel)  # 新功能
└── 数据库隔离机制 (database isolation)
    ├── 唯一数据库名生成
    ├── 自动创建/清理
    └── 连接池管理
```

### 关键代码模块
- `src/executor/file_executor.rs`: 文件级并发执行器
- `src/executor/progress.rs`: 进度监控
- `src/tester/database.rs`: 数据库隔离逻辑
- `src/cli.rs`: 命令行参数扩展

## 示例用法

创建几个测试文件验证并发功能：

```bash
# 创建测试文件
echo "SELECT 1 as test1;" > t/concurrent_test_1.test
echo "SELECT 2 as test2;" > t/concurrent_test_2.test
echo "SELECT 3 as test3;" > t/concurrent_test_3.test

# 串行执行（默认）
time cargo run -- concurrent_test_1 concurrent_test_2 concurrent_test_3

# 并发执行
time cargo run -- --parallel 3 concurrent_test_1 concurrent_test_2 concurrent_test_3
```

通过时间对比可以直观看到并发执行的性能提升。