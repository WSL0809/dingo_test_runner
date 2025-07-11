#!/bin/bash

# 简化的火焰图生成脚本
# 直接使用 flamegraph 工具，无需 perf

set -e

# 项目根目录
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
PERF_DIR="$PROJECT_ROOT/performance"
FLAMEGRAPH_DIR="$PERF_DIR/flamegraphs"

# 数据库连接配置
HOST=${HOST:-127.0.0.1}
PORT=${PORT:-3306}
DB_USER=${DB_USER:-root}
PASSWD=${PASSWD:-123456}
EXTENSION=${EXTENSION:-perf}

# 颜色输出
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# 检查 flamegraph 工具
check_flamegraph() {
    if ! command -v flamegraph &> /dev/null; then
        log_error "flamegraph 工具未安装"
        log_info "请安装: cargo install flamegraph"
        exit 1
    fi
}

# 生成火焰图
generate_flamegraph() {
    local test_name=$1
    local test_dir=$2
    local parallel_count=$3
    local description=$4
    
    log_info "生成火焰图: $description"
    log_info "  测试目录: $test_dir"
    log_info "  并发数: $parallel_count"
    
    cd "$PROJECT_ROOT"
    
    # 记录基线结果 (如果不存在)
    if [ ! -d "$PERF_DIR/results/${test_name}" ] || [ -z "$(ls -A "$PERF_DIR/results/${test_name}" 2>/dev/null)" ]; then
        log_info "记录基线结果..."
        RUST_LOG=info cargo run --release -- \
            --extension "$EXTENSION" \
            --record \
            --result-dir "$PERF_DIR/results" \
            --host "$HOST" \
            --port "$PORT" \
            --user "$DB_USER" \
            --passwd "$PASSWD" \
            "$test_dir"
    fi
    
    # 生成火焰图
    local output_file="$FLAMEGRAPH_DIR/flamegraph_${test_name}.svg"
    log_info "生成火焰图..."
    
    # Use full path to cargo to avoid PATH issues
    CARGO_PATH=$(which cargo)
    flamegraph -o "$output_file" -- \
        "$CARGO_PATH" run --release -- \
            --extension "$EXTENSION" \
            --parallel "$parallel_count" \
            --result-dir "$PERF_DIR/results" \
            --host "$HOST" \
            --port "$PORT" \
            --user "$DB_USER" \
            --passwd "$PASSWD" \
            "$test_dir"
    
    log_info "火焰图已生成: $output_file"
}

# 主函数
main() {
    local test_type=${1:-all}
    
    log_info "开始生成并发性能火焰图..."
    log_info "项目目录: $PROJECT_ROOT"
    log_info "火焰图目录: $FLAMEGRAPH_DIR"
    
    check_flamegraph
    
    # 构建项目
    log_info "构建项目..."
    cargo build --release
    
    case $test_type in
        basic|low)
            generate_flamegraph "concurrent_basic" "$PERF_DIR/test_cases/concurrent_basic" 4 "低并发测试 (4线程)"
            ;;
        medium)
            generate_flamegraph "concurrent_medium" "$PERF_DIR/test_cases/concurrent_medium" 8 "中并发测试 (8线程)"
            ;;
        high)
            generate_flamegraph "concurrent_high" "$PERF_DIR/test_cases/concurrent_high" 16 "高并发测试 (16线程)"
            ;;
        all)
            generate_flamegraph "concurrent_basic" "$PERF_DIR/test_cases/concurrent_basic" 4 "低并发测试 (4线程)"
            generate_flamegraph "concurrent_medium" "$PERF_DIR/test_cases/concurrent_medium" 8 "中并发测试 (8线程)"
            generate_flamegraph "concurrent_high" "$PERF_DIR/test_cases/concurrent_high" 16 "高并发测试 (16线程)"
            ;;
        *)
            log_error "未知的测试类型: $test_type"
            log_info "用法: $0 [basic|medium|high|all]"
            exit 1
            ;;
    esac
    
    log_info "火焰图生成完成!"
    log_info "查看结果: ls -la $FLAMEGRAPH_DIR/"
}

# 显示帮助信息
show_help() {
    cat << EOF
简化的火焰图生成脚本

用法: $0 [选项] [测试类型]

测试类型:
    basic, low    - 低并发测试 (4线程)
    medium        - 中并发测试 (8线程)  
    high          - 高并发测试 (16线程)
    all           - 运行所有测试 (默认)

选项:
    -h, --help    - 显示帮助信息

环境变量:
    HOST          - 数据库主机 (默认: 127.0.0.1)
    PORT          - 数据库端口 (默认: 3306)
    USER          - 数据库用户 (默认: root)
    PASSWD        - 数据库密码 (默认: 123456)
    EXTENSION     - 测试扩展名 (默认: perf)

示例:
    $0                                    # 生成所有火焰图
    $0 basic                              # 只生成低并发火焰图
    HOST=192.168.1.100 $0 medium          # 使用自定义数据库生成中并发火焰图
    
输出:
    - 火焰图: performance/flamegraphs/flamegraph_*.svg
    - 结果文件: performance/results/*/
EOF
}

# 参数解析
if [ "$1" = "-h" ] || [ "$1" = "--help" ]; then
    show_help
    exit 0
fi

main "$@"