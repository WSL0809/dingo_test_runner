#!/bin/bash

# 标签功能测试套件运行脚本
# 用于批量运行所有标签功能相关的测试用例

set -e

echo "🏷️  开始运行标签功能测试套件"
echo "================================"

# 设置测试环境
export RUST_LOG=info
TEST_DIR="tests/integration/advanced"
EXTENSION="dev"
HOST="127.0.0.1"
PORT="3306"
USER="root"
PASSWORD="123456"

# 检查MySQL连接
echo "🔧 检查MySQL连接..."
if ! mysql -h "$HOST" -P "$PORT" -u "$USER" -p"$PASSWORD" -e "SELECT 1" > /dev/null 2>&1; then
    echo "❌ 无法连接到MySQL数据库 ($HOST:$PORT)"
    echo "请确保MySQL服务正在运行，并且连接配置正确"
    exit 1
fi
echo "✅ MySQL连接正常"

# 定义测试文件列表
declare -a TEST_FILES=(
    "tags_test.test"
    "tags_basic_features.test"
    "tags_advanced_scenarios.test"
    "tags_concurrent_scenarios.test"
    "tags_edge_cases.test"
    "tags_performance_test.test"
    "tags_simple_integration.test"
)

# 计数器
total_tests=${#TEST_FILES[@]}
passed_tests=0
failed_tests=0

echo "📋 计划运行 $total_tests 个标签功能测试"
echo ""

# 创建测试结果基线（如果不存在）
echo "🔄 生成测试结果基线..."
for test_file in "${TEST_FILES[@]}"; do
    test_path="$TEST_DIR/$test_file"
    if [ -f "$test_path" ]; then
        echo "  生成基线: $test_file"
        cargo run -- --extension "$EXTENSION" --record --host "$HOST" --port "$PORT" --user "$USER" --passwd "$PASSWORD" "$test_path" > /dev/null 2>&1 || true
    fi
done

echo ""
echo "🧪 开始运行标签功能测试..."
echo ""

# 运行每个测试文件
for i in "${!TEST_FILES[@]}"; do
    test_file="${TEST_FILES[$i]}"
    test_path="$TEST_DIR/$test_file"
    test_num=$((i + 1))
    
    echo "[$test_num/$total_tests] 运行测试: $test_file"
    
    if [ -f "$test_path" ]; then
        # 运行测试
        if cargo run -- --extension "$EXTENSION" --host "$HOST" --port "$PORT" --user "$USER" --passwd "$PASSWORD" "$test_path" > /dev/null 2>&1; then
            echo "  ✅ 通过: $test_file"
            ((passed_tests++))
        else
            echo "  ❌ 失败: $test_file"
            ((failed_tests++))
            
            # 显示详细错误信息
            echo "  错误详情:"
            cargo run -- --extension "$EXTENSION" --host "$HOST" --port "$PORT" --user "$USER" --passwd "$PASSWORD" "$test_path" 2>&1 | tail -10 | sed 's/^/    /'
        fi
    else
        echo "  ⚠️  文件不存在: $test_path"
        ((failed_tests++))
    fi
    
    echo ""
done

# 运行并发测试
echo "🔄 运行并发标签功能测试..."
concurrent_tests=(
    "tags_concurrent_scenarios.test"
    "tags_performance_test.test"
)

for test_file in "${concurrent_tests[@]}"; do
    test_path="$TEST_DIR/$test_file"
    if [ -f "$test_path" ]; then
        echo "  并发测试: $test_file"
        if cargo run -- --extension "$EXTENSION" --parallel 2 --host "$HOST" --port "$PORT" --user "$USER" --passwd "$PASSWORD" "$test_path" > /dev/null 2>&1; then
            echo "  ✅ 并发测试通过: $test_file"
        else
            echo "  ❌ 并发测试失败: $test_file"
        fi
    fi
done

echo ""
echo "📊 测试结果统计"
echo "==================="
echo "总测试数: $total_tests"
echo "通过: $passed_tests"
echo "失败: $failed_tests"
echo "成功率: $(( passed_tests * 100 / total_tests ))%"

if [ $failed_tests -eq 0 ]; then
    echo ""
    echo "🎉 所有标签功能测试通过！"
    exit 0
else
    echo ""
    echo "⚠️  有 $failed_tests 个测试失败"
    exit 1
fi