# 性能测试和火焰图分析

本目录包含了 dingo_test_runner 的性能测试工具和火焰图生成脚本。

## 目录结构

```
performance/
├── README.md              # 本文档
├── flamegraphs/           # 火焰图输出目录
│   ├── flamegraph_concurrent_basic.svg
│   ├── flamegraph_concurrent_medium.svg
│   └── flamegraph_concurrent_high.svg
├── scripts/               # 性能测试脚本
│   ├── run_perf_analysis.sh      # 完整的性能分析脚本 (需要 perf 工具)
│   └── generate_flamegraph.sh    # 简化的火焰图生成脚本
├── test_cases/           # 性能测试用例 (自定义测试目录)
│   ├── concurrent_basic/     # 低并发测试 (4线程, 10个测试文件)
│   ├── concurrent_medium/    # 中并发测试 (8线程, 20个测试文件)
│   └── concurrent_high/      # 高并发测试 (16线程, 20个测试文件)
└── results/              # 性能测试结果 (自定义结果目录)
    ├── concurrent_basic/
    ├── concurrent_medium/
    └── concurrent_high/
```

## 快速开始

### 前提条件

1. **数据库连接**: 确保 MySQL 服务正在运行
2. **工具安装**: 
   - `flamegraph`: `cargo install flamegraph`
   - `perf` (可选): Linux 系统需要安装 perf 工具

### 生成火焰图

#### 方法1: 使用简化脚本 (推荐)

```bash
# 生成所有火焰图
./performance/scripts/generate_flamegraph.sh

# 只生成低并发火焰图
./performance/scripts/generate_flamegraph.sh basic

# 只生成中并发火焰图
./performance/scripts/generate_flamegraph.sh medium

# 只生成高并发火焰图
./performance/scripts/generate_flamegraph.sh high
```

#### 方法2: 使用完整性能分析脚本

```bash
# 需要安装 perf 工具
./performance/scripts/run_perf_analysis.sh

# 指定特定测试类型
./performance/scripts/run_perf_analysis.sh medium
```

### 自定义配置

通过环境变量配置数据库连接:

```bash
# 自定义数据库配置
HOST=192.168.1.100 \
PORT=3306 \
USER=testuser \
PASSWD=testpass \
./performance/scripts/generate_flamegraph.sh
```

## 测试场景说明

### 低并发测试 (concurrent_basic)
- **并发数**: 4 线程
- **测试文件**: 10 个
- **适用场景**: 验证基本并发功能，分析单线程性能
- **关注点**: 解析器性能、变量处理、控制流

### 中并发测试 (concurrent_medium)
- **并发数**: 8 线程
- **测试文件**: 20 个
- **适用场景**: 典型的并发使用场景
- **关注点**: 连接池管理、线程同步、资源竞争

### 高并发测试 (concurrent_high)
- **并发数**: 16 线程
- **测试文件**: 20 个
- **适用场景**: 高压力测试，识别并发瓶颈
- **关注点**: 连接池压力、内存分配、系统资源利用

## 火焰图分析指南

### 查看火焰图

生成的火焰图位于 `performance/flamegraphs/` 目录下:

```bash
# 在浏览器中查看
open performance/flamegraphs/flamegraph_concurrent_basic.svg

# 或者使用任何 SVG 查看器
```

### 分析重点

1. **CPU 热点识别**
   - 查找调用栈中占用时间最多的函数
   - 关注数据库连接、解析器、变量处理等关键路径

2. **并发瓶颈分析**
   - 识别锁争用情况
   - 分析线程同步开销
   - 检查连接池的使用效率

3. **内存分配模式**
   - 查找频繁的内存分配
   - 分析内存池的效果
   - 识别可能的内存泄漏

## 性能测试命令

### 使用 Makefile (推荐)

```bash
# 运行性能测试
make parallel-test PARALLEL=4 ARGS="performance/test_cases/concurrent_basic/"
make parallel-test PARALLEL=8 ARGS="performance/test_cases/concurrent_medium/"
make parallel-test PARALLEL=16 ARGS="performance/test_cases/concurrent_high/"

# 生成基线结果
make dev-record EXTENSION=perf ARGS="performance/test_cases/concurrent_basic/"
```

### 直接使用 cargo

```bash
# 记录基线结果
cargo run --release -- \
    --extension perf \
    --record \
    --result-dir performance/results \
    --host 127.0.0.1 \
    --port 3306 \
    --user root \
    --passwd 123456 \
    performance/test_cases/concurrent_basic/

# 运行并发测试
cargo run --release -- \
    --extension perf \
    --parallel 4 \
    --result-dir performance/results \
    --host 127.0.0.1 \
    --port 3306 \
    --user root \
    --passwd 123456 \
    performance/test_cases/concurrent_basic/
```

## 故障排除

### 常见问题

1. **flamegraph 工具未安装**
   ```bash
   cargo install flamegraph
   ```

2. **数据库连接失败**
   - 检查 MySQL 服务是否运行
   - 验证连接参数是否正确
   - 确认数据库权限

3. **结果文件不存在**
   - 首次运行需要使用 `--record` 参数生成基线结果
   - 或者运行脚本会自动生成基线结果

### 调试模式

```bash
# 启用调试日志
RUST_LOG=debug ./performance/scripts/generate_flamegraph.sh basic

# 查看详细的解析器日志
RUST_LOG=dingo_test_runner::tester::pest_parser=debug \
./performance/scripts/generate_flamegraph.sh basic
```

## 高级用法

### 自定义并发度

修改脚本中的并发参数或直接使用 cargo 命令:

```bash
# 自定义并发度
cargo run --release -- \
    --extension perf \
    --parallel 32 \
    --result-dir performance/results \
    performance/test_cases/concurrent_high/
```

### 组合不同测试类型

```bash
# 同时运行多种测试
./performance/scripts/generate_flamegraph.sh basic
./performance/scripts/generate_flamegraph.sh medium
./performance/scripts/generate_flamegraph.sh high

# 比较不同场景下的性能差异
```

## 贡献指南

如果需要添加新的性能测试场景:

1. 在 `test_cases/` 下创建新的测试目录
2. 在 `results/` 下创建对应的结果目录
3. 修改脚本以支持新的测试类型
4. 更新本文档