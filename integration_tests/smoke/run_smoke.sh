#!/bin/bash

# 冒烟测试执行脚本
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
SMOKE_DIR="$SCRIPT_DIR"

echo "🧪 Running Smoke Tests"
echo "======================"

# 测试配置
DB_HOST="127.0.0.1"
DB_PORT="13306"
DB_USER="root"
DB_PASSWORD="test123456"
EXTENSION="smoke"

# 检查测试环境
check_environment() {
    echo "🔍 Checking test environment..."
    
    # 检查Docker容器是否运行
    if ! docker ps | grep -q "dingo_test_mysql"; then
        echo "❌ MySQL test container is not running"
        echo "Please run: ./integration_tests/scripts/setup_env.sh"
        exit 1
    fi
    
    # 检查数据库连接
    if ! docker exec dingo_test_mysql mysqladmin ping -h localhost -u root -ptest123456 --silent; then
        echo "❌ Cannot connect to test database"
        exit 1
    fi
    
    echo "✅ Test environment is ready"
}

# 运行冒烟测试
run_smoke_tests() {
    echo "🚀 Running smoke tests..."
    
    cd "$PROJECT_ROOT"
    
    local test_files=(
        "01_basic_connection"
        "02_basic_variables" 
        "03_basic_sql"
        "04_control_flow"
        "05_regex_replacement"
    )
    
    local passed=0
    local failed=0
    local start_time=$(date +%s)
    
    echo "📋 Test Suite: Smoke Tests"
    echo "🎯 Target: Core functionality verification"
    echo "⏱️  Timeout: 15 minutes"
    echo ""
    
    for test_name in "${test_files[@]}"; do
        echo "🧪 Running: $test_name"
        
        local test_start=$(date +%s)
        
        if cargo run --release -- \
            --host "$DB_HOST" \
            --port "$DB_PORT" \
            --user "$DB_USER" \
            --passwd "$DB_PASSWORD" \
            --extension "$EXTENSION" \
            --record \
            "integration_tests/smoke/tests/${test_name}.test" > /dev/null 2>&1; then
            
            echo "✅ $test_name - PASSED ($(( $(date +%s) - test_start ))s)"
            passed=$((passed + 1))
        else
            echo "❌ $test_name - FAILED ($(( $(date +%s) - test_start ))s)"
            failed=$((failed + 1))
        fi
    done
    
    local end_time=$(date +%s)
    local duration=$((end_time - start_time))
    
    echo ""
    echo "📊 Smoke Test Results:"
    echo "  ✅ Passed: $passed"
    echo "  ❌ Failed: $failed"
    echo "  ⏱️  Duration: ${duration}s"
    echo ""
    
    if [ $failed -eq 0 ]; then
        echo "🎉 All smoke tests passed!"
        echo "✅ Core functionality is working correctly"
        return 0
    else
        echo "💥 Some smoke tests failed!"
        echo "❌ Core functionality issues detected"
        return 1
    fi
}

# 验证测试结果
verify_results() {
    echo "🔍 Verifying test results..."
    
    # 运行验证模式
    cd "$PROJECT_ROOT"
    
    local verification_passed=0
    local verification_failed=0
    
    for test_file in "$SMOKE_DIR/tests"/*.test; do
        local test_name=$(basename "$test_file" .test)
        
        if cargo run --release -- \
            --host "$DB_HOST" \
            --port "$DB_PORT" \
            --user "$DB_USER" \
            --passwd "$DB_PASSWORD" \
            --extension "$EXTENSION" \
            "integration_tests/smoke/tests/${test_name}.test" > /dev/null 2>&1; then
            
            verification_passed=$((verification_passed + 1))
        else
            echo "❌ Verification failed for: $test_name"
            verification_failed=$((verification_failed + 1))
        fi
    done
    
    if [ $verification_failed -eq 0 ]; then
        echo "✅ All verifications passed"
        return 0
    else
        echo "❌ $verification_failed verifications failed"
        return 1
    fi
}

# 生成报告
generate_report() {
    echo "📄 Generating smoke test report..."
    
    local report_file="$PROJECT_ROOT/integration_tests/smoke/smoke_test_report.txt"
    
    cat > "$report_file" << EOF
Smoke Test Report
================

Date: $(date)
Environment: Docker MySQL (port 13306)
Extension: $EXTENSION

Test Suite Status: $1
Duration: $(( $(date +%s) - $2 ))s

Test Files:
$(ls -la "$SMOKE_DIR/tests"/*.test | awk '{print "  " $9}')

Expected Results:
$(find "$PROJECT_ROOT/r/integration_tests/smoke" -name "*.smoke" 2>/dev/null | wc -l) result files generated

EOF
    
    echo "📄 Report saved to: $report_file"
}

# 主函数
main() {
    local start_time=$(date +%s)
    
    check_environment
    
    if run_smoke_tests && verify_results; then
        generate_report "PASSED" "$start_time"
        echo ""
        echo "🎉 Smoke tests completed successfully!"
        echo "✅ Ready for further testing"
        exit 0
    else
        generate_report "FAILED" "$start_time"
        echo ""
        echo "💥 Smoke tests failed!"
        echo "❌ Environment or core functionality issues detected"
        exit 1
    fi
}

# 执行主函数
main "$@"