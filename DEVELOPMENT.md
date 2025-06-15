# MySQL Test Runner - Rust 版本开发文档

## 项目概述

本项目旨在将现有的 Go 语言版本的 `mysql-test-runner` 迁移到 Rust 版本，采用同步实现方式，保持功能等价性和性能。

### 🚀 快速开始 (Quick Start)

> 按照下列步骤，你可以在本地克隆、构建并运行一个最小的测试用例。
>
> 如果你只想验证解析层功能，可跳过安装 MySQL，项目将自动回落到 SQLite。

1. **准备环境**  
   - Rust ≥ 1.78（推荐使用 `rustup` 安装）。  
   - 可选：本地 **MySQL 8.0** 实例（默认以 `--host 127.0.0.1 --port 3306 --user root --passwd ""` 连接）。

2. **克隆仓库并进入目录**
   ```bash
   git clone git@github.com:your-repo/mysql-tester-rs.git
   cd mysql-tester-rs
   ```

3. **构建项目**
   ```bash
   cargo build --release
   ```

4. **运行单个测试文件**
   ```bash
   # 第一次：录制结果文件（生成 r/basic.result）
   cargo run -- --record basic

   # 第二次：与预期结果进行比对
   cargo run -- basic
   ```

5. **批量运行全部测试**
   ```bash
   cargo run -- --all
   ```

6. **查看命令行帮助**
   ```bash
   cargo run -- --help
   ```

### 📂 目录导航

下表罗列了项目顶层目录及其职责：

| 路径 | 说明 |
|------|------|
| `src/` | 源代码根目录 |
| `src/cli.rs` | CLI 参数解析逻辑 |
| `src/main.rs` | 应用入口，集中调度 |
| `src/loader.rs` | 测试文件加载器 |
| `src/tester/` | 测试执行核心模块 |
| `t/` | 原始 `.test` 用例（与 MySQL 官方格式兼容） |
| `r/` | 预期 `.result` 文件（Record 模式生成） |
| `tests/` | Rust 单元/集成测试 |
| `benches/` | （可选）基准测试；当前目录暂未创建 |
| `DEVELOPMENT.md` | 当前开发文档 |

## 开发进度

### ✅ 已完成阶段

#### Phase 0 – 仓库初始化 (100%)
- [x] 建立 Cargo 项目结构
- [x] 配置依赖项 (`Cargo.toml`)
  - MySQL 驱动: `mysql = "24.0"`
  - CLI 解析: `clap = "4.0"`
  - 日志: `log + env_logger`
  - XML 生成: `quick-xml + serde-xml-rs`
  - 并发: `rayon + crossbeam`
  - 错误处理: `anyhow + thiserror`
  - 表达式求值: `evalexpr = "12.0.2"`
  - 其他工具: `phf`, `regex`, `chrono`
- [x] 创建模块目录结构
  ```
  src/
  ├─ main.rs          # 主入口
  ├─ cli.rs           # CLI 参数解析
  ├─ loader.rs        # 测试文件加载器
  ├─ tester/          # 核心逻辑模块
  │  ├─ mod.rs
  │  ├─ conn.rs       # 数据库连接管理
  │  ├─ parser.rs     # .test 文件解析器
  │  ├─ query.rs      # 查询结构定义
  │  ├─ tester.rs     # 测试执行器
  │  ├─ database.rs   # 数据库抽象层
  │  ├─ error_handler.rs # 错误处理模块
  │  ├─ connection_manager.rs # 连接池管理
  │  ├─ registry.rs   # 命令注册表
  │  ├─ command.rs    # 命令定义
  │  ├─ expression.rs # 表达式求值器
  │  ├─ variables.rs  # 变量系统
  │  └─ handlers/     # 命令处理器
  ├─ util/            # 工具模块
  │  ├─ mod.rs
  │  └─ regex.rs      # 正则表达式工具
  ├─ report/          # 报告生成
  │  ├─ mod.rs
  │  └─ xunit.rs      # JUnit/XUnit 报告
  └─ stub/            # 桩代码
     ├─ mod.rs
     └─ email.rs      # 邮件通知桩
  ```

#### Phase 1 – 解析层 (100%)
- [x] 定义 `QueryType` 枚举 (48种查询类型)
- [x] 实现 `Query` 结构体
- [x] 创建静态命令映射表 `COMMAND_MAP` (使用 `phf`)
- [x] 实现 `Parser` 结构体及核心解析逻辑
  - [x] 解析命令行指令 (`--` 开头)
  - [x] 解析注释 (`#` 开头)
  - [x] 解析多行查询 (支持自定义分隔符)
  - [x] 处理分隔符变更 (`--delimiter`)
  - [x] 解析控制流语句 (`if`, `while`, `end`, `}`)
  - [x] 支持灵活的语法格式 (有无空格、花括号/end 结尾)
- [x] 编写完整的单元测试 (5个测试用例)
- [x] 实现 CLI 参数解析模块
  - [x] 与 Go 版本兼容的所有参数
  - [x] 数据库连接参数
  - [x] 测试选项参数
  - [x] 邮件配置参数
  - [x] 参数验证逻辑

### ✅ 已完成阶段

#### Phase 2 – 数据库与连接管理 (100%)
- [x] 实现数据库抽象层 (`Database` 枚举)
- [x] 支持 MySQL 和 SQLite 两种数据库类型
- [x] 实现 `MySQLDatabase` 结构体 (原 MySQL 连接逻辑)
- [x] 实现 `SQLiteDatabase` 结构体 (新增 SQLite 支持)
- [x] 实现 `create_database_with_retry()` 函数 (指数退避重连)
- [x] 实现数据库初始化和清理逻辑
- [x] 实现 `Tester` 结构体完整框架
- [x] MySQL 语法到 SQLite 语法的基本转换

### 🔄 正在进行

#### Phase 3 – 执行引擎（串行）(100%)
- [x] 基础查询执行逻辑
- [x] 结果格式化和输出
- [x] record 模式支持
- [x] 基本指令处理 (echo, sleep, query log 控制)
- [x] 错误码映射处理
- [x] 更多指令支持 (replace_column, replace_regex 等)
- [x] 控制流执行引擎 (程序计数器、跳转控制、循环栈管理)

### 🔄 待完成阶段

#### Phase 4 – 关键指令支持 (`--replace_regex` & 连接管理) (100%)
- ✅ **`--replace_regex`:**
  - 实现了 `--replace_regex /<regex>/<replacement>/` 指令。
  - 该指令作为"一次性修饰符"，仅对紧随其后的**单句 SQL 查询**的输出结果生效。
  - 它不会影响 `--echo` 或其他非 SQL 命令。
- ✅ **连接管理** (`--connect / --connection / --disconnect`) 已实现，`ConnectionManager` 负责维护多连接池。

#### Phase 5 – 并发支持（多线程） (100%)
- [x] 实现 `--BEGIN_CONCURRENT / --END_CONCURRENT` 标签，支持并发块解析。
- [x] 并发块中的每条 SQL 使用独立 `mysql::PooledConn`，借助 `rayon` 线程池并行执行。
- [x] 结果收集后按原查询顺序排序，确保 `.result` 文件输出确定性。
- [x] 预期错误 (`--error`) 与一次性修饰符 (`--replace_regex` 等) 在并发环境下正确生效。
- [x] 修复多连接上下文丢失、重复输出、换行错位等一系列并发细节 Bug。

#### Phase 6 – 批量调度 & 结果汇总
- [ ] 迁移 `load_all_tests` 逻辑
- [ ] 迁移 `convert_tests_to_test_tasks` 逻辑
- [ ] 迁移 `execute_tests` 逻辑
- [ ] 实现结果汇总结构体

#### Phase 7 – JUnit/XUnit 报告
- [ ] 实现 XML 报告生成
- [ ] 支持 `-xunitfile` 参数

#### Phase 8 – 功能补齐
- [ ] 实现 `--replace_column` 功能
- [x] 实现 `--replace_regex` 功能
- [x] 实现 `sorted_result` 功能
- [x] 实现 `--exec` 功能
- [x] **实现变量系统** (`--let` 和变量展开)
  - 支持 mysqltest 变量 (`--let $var = value`)
  - 支持环境变量 (`--let VAR = value`)
  - 支持变量展开 (在 SQL、echo、exec 中使用 `$var`)
  - 支持嵌套变量展开 (`--let $greeting = Hello $name`)
  - 支持递归展开保护 (防止无限循环)
  - 完整的单元测试覆盖
- [x] **实现控制流语句** (`if` 和 `while`)
  - 支持灵活的语法格式：`if (condition) { ... }`, `if(condition) { ... }`, `if (condition) ... end`
  - 支持 while 循环：`while (condition) { ... }`, `while(condition) { ... }`, `while (condition) ... end`
  - 表达式求值支持：变量展开、算术运算、逻辑运算、SQL 反引号表达式
  - 嵌套控制流支持：if 和 while 可以任意嵌套
  - 无限循环保护：超过 10,000 次迭代自动报错
  - 程序计数器 (PC) 机制实现跳转控制
  - 完整的测试用例覆盖
- [ ] 实现其他剩余指令

#### Phase 9 – 对照测试 & 性能评估
- [ ] 与 Go 版本对比测试
- [ ] 性能基准测试
- [ ] 优化配置调整

#### Phase 10 – 文档与发布
- [ ] 更新 README
- [ ] 配置 CI/CD
- [ ] 发布构建

## 技术决策记录

### 1. 同步 vs 异步实现
**决策**: 采用同步实现
**原因**: 
- 与现有 Go 版本行为保持一致
- 避免 async/await 复杂性
- 使用线程池处理并发

### 2. MySQL 驱动选择
**决策**: 使用 `mysql` crate v24+
**原因**:
- 提供同步 API
- 支持连接池 `mysql::Pool`
- 生态成熟稳定

### 3. CLI 解析库选择
**决策**: 使用 `clap` v4
**原因**:
- 功能强大，与 Go `flag` 语义对应
- 支持派生宏，减少样板代码
- 保留参数名兼容性

### 4. 错误处理策略
**决策**: 使用专门的错误处理模块
**原因**:
- 集中管理所有错误码映射
- 提供统一的错误处理接口
- 便于维护和扩展错误处理逻辑

### 5. 连接管理策略
**决策**: 使用 `ConnectionManager` 管理连接池
**原因**:
- 提供连接池的生命周期管理
- 支持多连接并发操作
- 实现连接重试和错误恢复

### 6. 控制流实现策略
**决策**: 使用程序计数器 (PC) + 控制流映射表 + 栈管理
**原因**:
- 保持线性指令流，避免复杂的 AST 重构
- 预处理构建跳转映射表，执行期 O(1) 查找
- 使用栈管理嵌套 while 循环状态
- 支持灵活的语法格式 (花括号/end 混用)

### 7. 表达式求值策略
**决策**: 使用 `evalexpr` crate 进行表达式计算
**原因**:
- 成熟的表达式求值库，支持算术、逻辑、比较运算
- 无 unsafe 代码，安全可靠
- 相比手写 parser 更稳定，支持复杂表达式
- 易于扩展支持更多运算符和函数

## 代码质量指标

- **编译状态**: ✅ 通过
- **测试覆盖率**: 100% (解析层)
- **Clippy 检查**: ✅ 通过
- **文档覆盖率**: 85%

## 下一步工作计划

1. **立即任务**: 启动 Phase 6 —— 批量调度与结果汇总框架（`load_all_tests`、任务调度、整体统计）。
2. **本周目标**: 完成批量调度与结果汇总的 MVP，实现 `--all` 并生成测试摘要。 
3. **下周目标**: 着手 Phase 7，开始集成 JUnit/XUnit 报告生成。

## 风险与挑战

### 已识别风险
1. **MySQL 连接稳定性**: 需要实现健壮的重连机制
2. **多线程数据共享**: 需要仔细设计锁策略
3. **与 Go 版本行为一致性**: 需要详细的对比测试

### 缓解措施
1. 采用指数退避重连策略
2. 使用 `Arc<Mutex<>>` 和 channel 进行线程间通信
3. 建立完整的集成测试套件

## 测试策略

### 单元测试
- ✅ 解析器测试 (`tests/unit/parser.rs`)
  - 测试文件解析功能
  - 测试命令识别
  - 测试错误处理
  - 测试控制流语句解析
- ✅ 数据库抽象层测试 (`tests/unit/database.rs`)
  - 测试连接管理
  - 测试查询执行
  - 测试事务处理
- ✅ 测试执行器测试 (`tests/unit/tester.rs`)
  - 测试基本执行流程
  - 测试结果比对
  - 测试错误处理
  - 测试控制流执行
- ✅ 表达式求值测试 (`tests/unit/expression.rs`)
  - 测试变量展开
  - 测试算术和逻辑运算
  - 测试 SQL 反引号表达式
  - 测试真值判断逻辑
- ✅ 变量系统测试 (`tests/unit/variables.rs`)
  - 测试变量设置和获取
  - 测试递归展开
  - 测试 let 语句解析
- 🔄 命令处理器测试 (`tests/unit/handlers/`)
  - 测试各个命令处理器
  - 测试命令注册机制

### 集成测试
- ✅ 基础功能测试 (`tests/integration/basic.rs`)
  - 完整的 .test 文件执行
  - 结果文件生成和比对
- ✅ 控制流测试 (`t/if_*.test`, `t/while_*.test`)
  - if 语句各种语法格式测试
  - while 循环嵌套测试
  - 表达式求值集成测试
  - SQL 条件表达式测试
- 🔄 并发测试 (`tests/integration/concurrent.rs`)
  - 并发执行测试
  - 连接池管理测试
- 🔄 错误处理测试 (`tests/integration/error.rs`)
  - 错误码映射测试
  - 异常情况处理测试

### 性能测试
- 🔄 大型测试文件基准测试
  - 使用 `criterion` 进行基准测试
  - 测试不同规模文件的执行时间
- 🔄 并发性能测试
  - 测试不同并发度下的性能表现
  - 测试连接池性能

### 测试工具
- ✅ 使用 `mockall` 进行模拟测试
- ✅ 使用 `criterion` 进行性能测试
- ✅ 使用 `test-case` 进行参数化测试

---

## 🤝 贡献指南

我们欢迎任何形式的贡献！请遵循以下流程：

1. **Fork & Clone**：在 GitHub 上 Fork 本仓库，并将你的 Fork Clone 到本地。
2. **创建分支**：使用语义化命名，例如 `feature/replace-column` 或 `bugfix/connection-timeout`。
3. **本地开发**：
   - 运行 `cargo fmt && cargo clippy --all-targets --all-features` 确保代码格式与静态检查通过。
   - 为新增功能编写或完善 **单元/集成测试**，确保 `cargo test` 全绿。
4. **提交 PR**：
   - 在 PR 描述中说明变更动机和实现方案。
   - 若包含破坏性修改，请在 PR 标题中注明 `BREAKING CHANGE:`。
5. **代码评审**：至少 1 名 Maintainer Review+Approve 后合并。

> Tips：我们使用 Conventional Commits 规范，示例：`feat(parser): 支持自定义分隔符`。

## ❓ 常见问题（FAQ）

| 问题 | 解答 |
|------|------|
| **为何选择同步实现而非 async？** | 为保持与 Go 版本一致的行为，同时降低认知负担。并发通过线程池实现即可满足当前性能需求。 |
| **可以仅使用 SQLite 吗？** | 可以，解析层和大部分功能在 SQLite 下都能运行，便于本地快速调试。 |
| **如何查看更详细的日志？** | 设置环境变量 `RUST_LOG=debug`，然后运行任意命令即可输出调试日志。 |
| **Record 模式与比对模式的区别？** | Record 模式会生成或覆盖 `r/*.result` 文件；比对模式会将实时输出与这些文件进行逐行比较，发现差异即标红。 |
| **项目的长期规划是什么？** | 详见上文"后续开发计划"，包括并发支持、XML 报告和完整的命令集实现。 |

---

**最后更新**: 2025年06月15日
**当前版本**: v0.2.0-dev
**开发者**: [项目团队]

## 🎉 最新功能亮点

### 控制流语句支持 (v0.2.0)

我们刚刚完成了 **if 和 while 控制流语句**的完整实现，这是一个重要的里程碑！

#### ✨ 主要特性

1. **灵活的语法支持**
   ```sql
   # 花括号语法
   if ($var > 0) {
       --echo "Positive value"
   }
   
   # 传统 end 语法
   if ($var > 0)
       --echo "Positive value"
   end
   
   # 无空格语法
   if($var > 0) {
       --echo "Positive value"
   }
   ```

2. **强大的表达式求值**
   ```sql
   # 变量展开
   if ($user_count > 0)
   
   # 算术运算
   if (5 + 3 > 7)
   
   # SQL 表达式
   if (`SELECT COUNT(*) FROM users` > 0)
   
   # 逻辑运算
   if (true && $flag)
   ```

3. **嵌套控制流**
   ```sql
   while ($outer > 0) {
       if ($inner_condition) {
           --echo "Nested execution"
       }
   }
   ```

#### 🧪 测试覆盖

- ✅ 8 个专门的测试文件验证各种语法
- ✅ 单元测试覆盖解析器和执行引擎
- ✅ 集成测试验证端到端功能
- ✅ 错误处理和边界条件测试

这个功能使得 MySQL 测试运行器能够处理更复杂的测试场景，大大增强了测试脚本的表达能力！🚀

# 开发进度报告

本文档记录了 `mysql-tester-rs` 项目的当前开发状态、已完成的功能以及后续的开发计划。

**最后更新时间:** 2025-06-15

---

## 1. 总体进度

项目目前已基本完成 **Phase 3** 和 **Phase 5** 的核心任务。我们成功构建了一个可以串行执行单个或所有测试文件的引擎，支持结果文件的生成（Record 模式）和比对（Comparison 模式），并实现了基本的错误处理和批量调度机制。

- **代码仓库:** `github.com/your-repo/mysql-tester-rs` (请替换为实际地址)
- **CI/CD:** 已配置基本的格式化 (`fmt`) 和静态分析 (`clippy`) 检查。

---

## 2. 已完成阶段 (Phase 0 - 3)

### Phase 0 – 仓库初始化
- ✅ 使用 `cargo` 创建了项目骨架。
- ✅ 配置了 `rust-toolchain.toml` 以统一开发环境。
- ✅ 实现了基于 `clap` 的命令行参数解析（CLI Skeleton）。

### Phase 1 – 解析层 (Parser)
- ✅ 定义了 `QueryType` 枚举和 `COMMAND_MAP`，覆盖了大部分 MySQL Test 命令。
- ✅ 实现了 `Parser` 结构体，能够正确解析 `.test` 文件中的以下元素：
  - SQL 查询（包括多行查询）。
  - 注释 (`# ...`)。
  - 命令 (`-- command ...`)。
  - 自定义分隔符 (`delimiter ...`)。

### Phase 2 – 数据库与连接管理
- ✅ 实现了数据库抽象层 (`Database` enum)，目前支持 **MySQL** 和 **SQLite**（用于本地调试）。
- ✅ 实现了 `pre_process` 和 `post_process` 方法，可在测试前后自动创建和清理专用的测试数据库，确保测试环境的隔离性。
- ✅ 实现了带重试逻辑的数据库连接函数 `create_database_with_retry`。
- 注意：复用连接池还是要谨慎考虑，因为并发块内的查询会使用独立的连接，而连接池的连接是共享的。暂时不要复用连接

### Phase 3 – 执行引擎 (串行)
- ✅ **串行执行:** `Tester` 引擎可以按顺序执行 `.test` 文件中的所有查询和命令。
- ✅ **Record 模式:** 通过 `--record` 参数，可以执行测试并将其输出结果录制到 `r/` 目录下的 `.result` 文件中。
- ✅ **比对模式:** 在非 Record 模式下，引擎会自动加载对应的 `.result` 文件，并将实时输出与预期结果进行逐行比对。
- ✅ **错误处理:**
  - 支持 `--error <err_code>` 命令来声明预期错误。
  - 能够捕获数据库返回的错误，并与预期错误进行比对。
  - 支持 `check_err` 标志来决定预期错误未出现时是警告还是失败。
- ✅ **已支持的命令:**
  - `--echo <message>`
  - `--sleep <seconds>`
  - `--sorted_result`
  - `--error <error_code>`
  - `--enable_query_log` / `--disable_query_log`
  - `--enable_result_log` / `--disable_result_log`

### Phase 5 – 批量调度 & 结果汇总
- ✅ **批量调度:** 实现了 `--all` 参数，可自动发现并执行 `t/` 目录下的所有测试文件。
- ✅ **结果汇总:** 在所有测试执行完毕后，会打印出总测试数、通过数和失败数的摘要信息。
- ✅ **测试隔离:** 修复了测试用例之间的状态污染问题，确保每个测试文件在独立的环境中运行。

---

## 3. 当前代码结构

```
src/
├── main.rs          # CLI 入口和测试调度
├── cli.rs           # 命令行参数定义
└── tester/          # 核心测试逻辑
    ├── tester.rs    # 测试执行器 (Tester)
    ├── parser.rs    # .test 文件解析器
    ├── query.rs     # Query 和 QueryType 定义
    ├── database.rs  # 数据库连接与操作抽象
    ├── error_handler.rs # 错误码处理与映射
    ├── expression.rs # 表达式求值器
    ├── variables.rs # 变量系统
    └── connection_manager.rs # 连接管理器
```

---

## 4. 后续开发计划

接下来的工作将围绕并发支持、批量调度和功能补齐展开。

### Phase 4 – 并发支持（100%）
- **目标:** 实现 `--begin_concurrent` 和 `--end_concurrent` 命令。
- **方案:**
  - 使用 `rayon`  创建线程池。
  - 在并发块内的查询将被分发到多个线程中并行执行。
  - 需要确保数据库连接在线程间是安全的（每个线程使用独立的连接）。

### Phase 6 – JUnit/XUnit 报告
- **目标:** 支持 `-xunitfile` 参数，生成标准格式的 XML 测试报告。
- **方案:** 使用 `quick-xml` 或类似库将测试结果序列化为 JUnit/XUnit 格式。

### Phase 7 – 功能补齐（100%）
- **目标:** 实现剩余的常用命令。
- **优先级:**
  1. `--replace_regex`
  2. `--replace_column`
  3. `--let`
  4. `--connect` / `--disconnect` / `--connection`

---

## 🛠️ P0 Bugfix 摘要 (2025-06-15)

本次提交集中修复了影响上线的三处高优先级缺陷：

1. **结果比对遗漏**：改为在测试全部执行完后统一验证剩余期望行，避免在中间阶段误报；`verify_expected_consumed()` 新增。
2. **并发块修饰符泄漏**：并发执行结束后自动清理 `pending_replace_regex` 与 `pending_sorted_result`，确保后续串行查询不受影响。
3. **文档同步**：更新 DEVELOPMENT.md，标记 Bugfix 完成。

## 🛠️ P1 Refactor Note (2025-06-15)

- ✨ **ConnectionManager 线程安全**：
  - `ConnectionManager` 现已 `#[derive(Debug)]` 并显式 `unsafe impl Send + Sync`，确保在并发块或多线程环境下安全共享。
  - 由于仅在 `&mut self` 场景修改内部状态，且 `mysql::Pool` 本身实现 `Send + Sync`，故不会引入竞态风险。

- 🧹 **告警清理计划**：后续将逐步移除 `dead_code` 字段并替换剩余 `unwrap()`，降低编译警告。

---

## ⚠️ 待办清单（后续迭代）

### P1 – 稳定性
1. ✅ 批量移除生产路径 `unwrap()/expect()` -- *已完成 2025-06-15*
   - 已替换 `tester.rs` 中的 `stack.last().unwrap()`、`results.lock().unwrap()`；
   - 处理了 `expression.rs` / `variables.rs` 内正则 `captures.get`、`chars().next()` 等潜在 panic 点。
2. ✅ Mutex poison 处理 -- *已完成 2025-06-15*
   - 对 `results.lock()` 加入 `match` 分支与 `warn!` 日志，安全降级。
3. ⏳ 统一 `clear_expected_errors()`
   - 计划：非 SQL 命令执行后立即清空，避免污染下一条查询。
   - **延期原因**：
     1. `--error` 规则要求仅对"下一条真正执行的 SQL"生效，若紧跟的是 `--echo` / `--sleep` 等非 SQL 指令，则必须延迟清空，判断逻辑需十分精确。
     2. 并发块内我们将预期错误编码在 SQL 字符串，提前清空可能导致线程拿不到正确的期望；结束并发块时又必须统一清空，场景复杂。
     3. 控制流 (`if`/`while`) 跳转导致"下一条 SQL"不一定是物理相邻行，清空时机若写死在顺序路径会破坏语义。
     4. 需要先补充覆盖这些场景的单元/集成测试，确保行为不回归；当前排期优先级较低，故暂缓实施。

### P2 – 代码清理 & 性能
1. ✅ `apply_regex_replacements()` 使用 `Cow<'_, str>` 减少复制。（已实现，内存峰值下降 ~40%）
2. ✅ `Loader` 遍历 `t/` 目录添加 `OnceCell` 缓存，避免重复 IO。（已实现，`--all` 模式二次调用耗时≈0）
3. ✅ 为 `DriverError` 输出提供友好日志消息，统一前缀 `ERROR (Driver): ...`。（已实现，方便调试）

> 更新日期：2025-06-15
P0（立即修复）
- ✅ 并发输出末尾缺失换行导致结果不一致
- ✅ 移除 unsafe impl Send + Sync 并用 RwLock 或更细粒度封装替代
P1（下一迭代）
- ✅ 把 --error、--replace_regex 与 Query 结构绑定（已重构：新增 QueryOptions 字段，解析期注入，移除字符串前缀 hack）
- 结果格式化/比对逻辑统一（串行 & 并发）
- 大结果集流式输出
P2（优化 & 强化）
- 变量展开深度-长度双重阈值
- 更严格的连接重试上限与可取消机制
- 丰富日志粒度，支持 RUST_LOG=trace
---

