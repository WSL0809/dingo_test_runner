# --source 功能实现总结

## 功能概述

成功为 dingo_test_runner 项目实现了 `--source` 功能，该功能允许测试文件包含并执行其他测试文件的内容，实现测试脚本的模块化和代码复用。

## 实现要点

### 1. 核心功能
- **文件包含**: 支持 `--source <filename>` 语法加载并执行其他 .inc 文件
- **变量展开**: 支持在文件路径中使用变量，如 `--source $include_dir/setup.inc`
- **相对路径解析**: 文件路径相对于 `t/` 目录解析
- **递归支持**: 支持嵌套 source（一个被 source 的文件可以再 source 其他文件）
- **状态共享**: 变量、连接等状态在主文件和被 source 的文件间完全共享

### 2. 安全机制
- **递归深度限制**: 最大嵌套深度为 16 层，防止无限递归
- **错误处理**: 支持 `--error` 指令预期 source 命令的错误
- **文件存在性检查**: 在执行前检查被 source 的文件是否存在

### 3. 代码实现

#### 结构体扩展
在 `Tester` 结构体中添加了 `source_depth: u32` 字段用于跟踪递归深度。

#### 核心方法
实现了 `handle_source()` 方法，包含以下逻辑：
- 递归深度检查
- 变量展开和路径解析
- 文件存在性验证
- 错误预期处理
- 递归执行被 source 文件的查询

#### 解析器支持
现有的解析器已经支持 `QueryType::Source` 类型，无需额外修改。

## 测试覆盖

创建了完整的测试套件，涵盖以下场景：

1. **source_basic.test**: 基本的 source 功能测试
2. **source_nested.test**: 嵌套 source 功能测试（一个文件 source 另一个文件）
3. **source_variables.test**: 变量在 source 文件间的共享测试
4. **source_syntax.test**: 语法格式测试（包括变量路径）
5. **source_error.test**: 错误处理测试（不存在的文件）
6. **source_depth.test**: 嵌套深度测试
7. **source_comprehensive.test**: 综合功能测试

所有测试都通过，pass rate: 100%。

## 使用示例

### 基本用法
```sql
# 主测试文件
--source include/setup.inc
SELECT * FROM common_table;
--source include/cleanup.inc
```

### 变量路径
```sql
let $include_dir = include
--source $include_dir/setup.inc
```

### 错误处理
```sql
--error 1
--source include/nonexistent.inc
```

## 文件结构

```
t/
├── include/
│   ├── setup.inc          # 通用设置脚本
│   ├── cleanup.inc        # 通用清理脚本
│   ├── variable_test.inc  # 变量测试脚本
│   ├── nested_setup.inc   # 嵌套测试脚本
│   └── deep*.inc         # 深度嵌套测试脚本
└── source_*.test         # 各种 source 功能测试
```

## 与 MySQL 官方兼容性

实现完全兼容 MySQL 官方测试格式：
- 使用 `--source` 前缀（必须带 `--`）
- 支持相对路径解析
- 支持变量展开
- 支持错误预期
- 支持递归嵌套

## 性能考虑

- 文件只在需要时读取和解析
- 递归深度限制防止栈溢出
- 状态共享避免重复初始化
- 错误快速失败，避免不必要的处理

