# 标签功能测试套件

本目录包含了 dingo_test_runner 标签功能的全面测试套件，涵盖了所有已实现的标签命令和边界情况。

## 📋 测试文件概览

### 1. `tags_test.test`
**原始标签功能测试**
- 基础标签命令测试
- 查询日志控制 (`--disable_query_log`, `--enable_query_log`)
- 结果日志控制 (`--disable_result_log`, `--enable_result_log`)
- 结果排序 (`--sorted_result`)
- 正则表达式替换 (`--replace_regex`)
- 错误处理 (`--error`)

### 2. `tags_basic_features.test`
**基础功能扩展测试**
- 日志控制组合测试
- 多个 replace_regex 模式组合
- 复杂数据的 sorted_result 测试
- 错误处理的边界情况
- 标签状态管理

### 3. `tags_advanced_scenarios.test`
**高级场景测试**
- 标签状态切换测试
- 复杂正则表达式模式
- NULL 值处理
- 嵌套错误处理
- 标签与变量系统结合

### 4. `tags_concurrent_scenarios.test`
**并发环境测试**
- 并发模式下的标签行为
- 多线程标签状态隔离
- 并发 replace_regex 处理
- 并发错误处理
- 状态恢复测试

### 5. `tags_edge_cases.test`
**边界条件测试**
- 空结果集处理
- 单行结果处理
- 无匹配的 replace_regex
- 特殊字符处理
- Unicode 字符支持
- 极长文本处理

### 6. `tags_performance_test.test`
**性能测试**
- 大数据量的 sorted_result 性能
- 多个 replace_regex 的性能
- 频繁状态切换的性能
- 复杂查询的日志控制性能
- 大结果集处理性能

### 7. `tags_integration_test.test`
**集成测试**
- 标签与变量系统集成
- 标签与控制流集成
- 标签与连接管理集成
- 标签与事务集成
- 标签与存储过程集成
- 标签与视图集成

## 🏷️ 支持的标签命令

### 日志控制标签
- `--disable_query_log` - 禁用查询日志
- `--enable_query_log` - 启用查询日志
- `--disable_result_log` - 禁用结果日志
- `--enable_result_log` - 启用结果日志

### 结果处理标签
- `--sorted_result` - 对查询结果进行排序
- `--replace_regex /pattern/replacement/` - 使用正则表达式替换结果中的模式

### 错误处理标签
- `--error <code>` - 期望下一个查询返回特定错误码

### 输出控制标签
- `--echo <message>` - 输出消息到测试结果

## 🚀 运行测试

### 单独运行测试
```bash
# 运行单个测试文件
cargo run -- --extension dev tests/integration/advanced/tags_basic_features.test

# 记录测试基线
cargo run -- --extension dev --record tests/integration/advanced/tags_basic_features.test
```

### 批量运行测试
```bash
# 运行所有标签功能测试
./run_tag_tests.sh

# 运行特定类型的测试
cargo run -- --extension dev tests/integration/advanced/tags_*.test
```

### 并发运行测试
```bash
# 并发运行多个测试文件
cargo run -- --extension dev --parallel 4 tests/integration/advanced/tags_*.test
```

## 📈 测试覆盖率

### 功能覆盖
- ✅ 所有已实现的标签命令
- ✅ 标签状态管理
- ✅ 标签组合使用
- ✅ 错误处理
- ✅ 边界条件
- ✅ 性能测试
- ✅ 集成测试

### 场景覆盖
- ✅ 基础功能测试
- ✅ 高级场景测试
- ✅ 并发环境测试
- ✅ 边界条件测试
- ✅ 性能压力测试
- ✅ 系统集成测试

## 🛠️ 测试环境配置

### 数据库配置
```bash
# 默认配置
HOST=127.0.0.1
PORT=3306
USER=root
PASSWORD=123456
```

### 环境变量
```bash
# 启用调试日志
export RUST_LOG=debug

# 使用开发环境扩展
EXTENSION=dev
```

## 📝 添加新测试

### 测试文件命名规范
- `tags_<category>_<type>.test` - 标签功能测试
- `tags_<specific_feature>.test` - 特定功能测试

### 测试内容规范
1. **测试目标明确** - 每个测试文件有明确的测试目标
2. **场景完整** - 覆盖正常情况和异常情况
3. **数据清理** - 测试后清理创建的数据
4. **结果可重复** - 测试结果应该是可重复的
5. **注释清晰** - 使用 `--echo` 提供清晰的测试说明

### 示例测试结构
```sql
# 测试描述
--echo Testing specific tag functionality

# 准备测试数据
CREATE TABLE test_table (...);
INSERT INTO test_table VALUES (...);

# 执行测试
--<tag_command>
SELECT * FROM test_table;

# 清理数据
DROP TABLE test_table;

--echo Test completed!
```

## 🔧 故障排除

### 常见问题
1. **测试失败** - 检查数据库连接配置
2. **结果不匹配** - 重新生成测试基线
3. **性能问题** - 调整测试数据量
4. **并发问题** - 检查数据库连接池配置

### 调试技巧
```bash
# 启用详细日志
RUST_LOG=debug cargo run -- --extension dev test_file.test

# 查看具体错误
cargo run -- --extension dev test_file.test 2>&1 | grep -A 5 -B 5 "ERROR"
```

## 📚 参考资料

- [CLAUDE.md](../../../CLAUDE.md) - 项目开发指南
- [MySQL Test Format](https://dev.mysql.com/doc/dev/mysql-server/latest/PAGE_MYSQL_TEST_RUN.html) - MySQL 测试格式参考
- [Rust Regex](https://docs.rs/regex/latest/regex/) - Rust 正则表达式文档