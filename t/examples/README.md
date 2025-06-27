# 用户测试示例

这个目录包含了一些基本的测试示例，帮助用户快速上手 dingo_test_runner。

## 示例文件

### basic_example.test
基础的 SQL 测试示例，演示：
- 创建表
- 插入数据
- 查询验证

### variable_example.test  
变量使用示例，演示：
- 变量定义和使用
- 变量在 SQL 中的应用

### connection_example.test
数据库连接示例，演示：
- 连接数据库
- 执行查询
- 断开连接

## 使用方法

```bash
# 运行单个示例
cargo run -- examples/basic_example

# 运行所有示例
cargo run -- examples/

# 生成期望结果（首次运行时）
cargo run -- --record examples/basic_example
```

## 添加你的测试

1. 在 `t/` 目录下创建 `.test` 文件
2. 使用 `--record` 模式生成期望结果
3. 正常运行测试进行验证

更多功能请参考项目根目录的 CLAUDE.md 文档。