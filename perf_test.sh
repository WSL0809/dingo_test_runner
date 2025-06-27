#!/bin/bash

# Performance testing script for dingo_test_runner

set -e

echo "🔥 Performance Testing for dingo_test_runner"

# Test scenarios
scenarios=(
    "单文件解析性能测试"
    "并发执行性能测试" 
    "数据库连接性能测试"
    "全量测试性能基线"
)

# 1. 单文件解析性能测试
echo "1. 单文件解析性能测试"
cargo flamegraph --bin dingo_test_runner -- t/examples/basic_example.test --record
mv flamegraph.svg flamegraph_single_parse.svg

# 2. 并发执行性能测试
echo "2. 并发执行性能测试"
cargo flamegraph --bin dingo_test_runner -- --parallel 4 t/examples/
mv flamegraph.svg flamegraph_concurrent.svg

# 3. 数据库连接性能测试
echo "3. 数据库连接性能测试"
cargo flamegraph --bin dingo_test_runner -- --max-connections 10 t/examples/
mv flamegraph.svg flamegraph_db_connections.svg

# 4. 全量测试性能基线
echo "4. 全量测试性能基线"
cargo flamegraph --bin dingo_test_runner -- --all --parallel 8
mv flamegraph.svg flamegraph_full_baseline.svg

echo "🎯 Performance analysis complete!"
echo "Generated flame graphs:"
echo "  - flamegraph_single_parse.svg"
echo "  - flamegraph_concurrent.svg" 
echo "  - flamegraph_db_connections.svg"
echo "  - flamegraph_full_baseline.svg"