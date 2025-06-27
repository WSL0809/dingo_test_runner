#!/bin/bash

# 性能测试脚本 - 专注解析性能，不涉及数据库连接

echo "🔥 解析器性能测试（无数据库连接）"

# 测试单文件解析性能
echo "1. 测试单文件解析性能..."
# 使用 cargo build 测试编译性能
time cargo build --release

# 测试大批量文件解析（创建多个测试文件）
echo "2. 创建大批量测试文件..."
mkdir -p perf_tests

for i in {1..50}; do
  cp t/examples/simple_parse_test.test perf_tests/test_$i.test
done

echo "3. 使用内存性能分析..."
# 使用 Instruments 进行分析
CARGO_PROFILE_RELEASE_DEBUG=true cargo flamegraph --bin dingo_test_runner -- --help

echo "Performance analysis complete!"