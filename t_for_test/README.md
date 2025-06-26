# 重构后的测试目录结构

## 目录说明

- **basic/** - 基础功能测试（连接、简单查询、echo等）
- **variables/** - 变量系统测试（let语句、变量展开、表达式）
- **control_flow/** - 控制流测试（if/while语句、嵌套控制）
- **concurrent/** - 并发执行测试
- **connection/** - 连接管理测试（多连接、连接切换）
- **error_handling/** - 错误处理测试（预期错误、失败处理）
- **advanced/** - 高级功能测试（正则替换、结果排序、标签等）
- **source/** - Source/Include 功能测试
- **performance/** - 性能和边界测试（解析器边界、循环测试）
- **include/** - 共享的包含文件
- **demo_tests/** - 演示测试（保持原结构）
- **br/** - BR相关测试（保持原结构）

## 迁移说明

- 测试文件按功能分类重新组织到相应子目录
- 结果文件(.result)仍保持在原有的 `../r/` 目录结构中
- Include 文件路径已保持兼容
- 可通过目录名快速定位相关功能测试
- 测试运行器会自动查找对应的结果文件

## 运行测试

```bash
# 运行特定分类的测试
cargo run -- t_for_test/basic/
cargo run -- t_for_test/variables/
cargo run -- t_for_test/control_flow/

# 运行单个测试
cargo run -- t_for_test/basic/simple_test.test

# 注意：系统会自动在 r/ 目录查找对应的 .result 文件
# 例如：t_for_test/basic/simple_test.test -> r/simple_test.result
```

## 重要说明

测试运行器设计为：
- 测试文件可以在任意位置组织
- 结果文件固定在 `r/` 目录查找
- 测试名称与结果文件名保持一对一映射关系