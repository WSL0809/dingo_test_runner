#!/bin/bash

# 快速设置性能测试基线脚本

set -e

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
PERF_DIR="$PROJECT_ROOT/performance"

# 数据库连接配置
HOST=${HOST:-127.0.0.1}
PORT=${PORT:-3306}
USER=${USER:-root}
PASSWD=${PASSWD:-123456}
EXTENSION=${EXTENSION:-perf}

# 颜色输出
GREEN='\033[0;32m'
NC='\033[0m'

log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

# 为所有测试文件创建简单的基线结果
create_baseline_results() {
    local test_category=$1
    local result_dir="$PERF_DIR/results/$test_category"
    
    log_info "为 $test_category 创建基线结果..."
    
    # 创建结果目录
    mkdir -p "$result_dir"
    
    # 为每个测试文件创建期望结果
    for test_file in "$PERF_DIR/test_cases/$test_category"/*.test; do
        local test_name=$(basename "$test_file" .test)
        local result_file="$result_dir/${test_name}.perf"
        
        if [ ! -f "$result_file" ]; then
            log_info "  创建 $result_file"
            cat > "$result_file" << 'EOF'
Variable 1: 100
Variable 2: hello world
Variable is greater than 50
Counter: 0
Counter: 1
Counter: 2
Test completed successfully
EOF
        fi
    done
}

main() {
    log_info "设置性能测试基线结果..."
    
    create_baseline_results "concurrent_basic"
    create_baseline_results "concurrent_medium" 
    create_baseline_results "concurrent_high"
    
    log_info "基线结果设置完成!"
    log_info "现在可以运行性能测试了:"
    log_info "  ./performance/scripts/generate_flamegraph.sh basic"
}

main "$@"