#!/bin/bash

# 文件级并发性能分析脚本
# 使用 perf 工具和 flamegraph 生成性能火焰图

set -e

# 项目根目录
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
PERF_DIR="$PROJECT_ROOT/performance"
FLAMEGRAPH_DIR="$PERF_DIR/flamegraphs"
SCRIPTS_DIR="$PERF_DIR/scripts"

# 数据库连接配置
HOST=${HOST:-127.0.0.1}
PORT=${PORT:-3306}
USER=${USER:-root}
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

# 检查必要的工具
check_tools() {
    local tools=("perf" "flamegraph")
    local missing=()
    
    for tool in "${tools[@]}"; do
        if ! command -v "$tool" &> /dev/null; then
            missing+=("$tool")
        fi
    done
    
    if [ ${#missing[@]} -gt 0 ]; then
        log_error "缺少必要工具: ${missing[*]}"
        log_info "请安装缺少的工具:"
        for tool in "${missing[@]}"; do
            case $tool in
                perf)
                    log_info "  - perf: sudo apt-get install linux-perf (Ubuntu) 或 brew install perf (macOS)"
                    ;;
                flamegraph)
                    log_info "  - flamegraph: cargo install flamegraph"
                    ;;
            esac
        done
        exit 1
    fi
}

# 构建项目
build_project() {
    log_info "构建项目..."
    cd "$PROJECT_ROOT"
    cargo build --release
}

# 运行性能分析
run_perf_test() {
    local test_name=$1
    local test_dir=$2
    local parallel_count=$3
    local description=$4
    
    log_info "运行性能测试: $description"
    log_info "  测试目录: $test_dir"
    log_info "  并发数: $parallel_count"
    
    cd "$PROJECT_ROOT"
    
    # 记录基线结果 (如果不存在)
    if [ ! -d "$PERF_DIR/results/${test_name}" ] || [ -z "$(ls -A "$PERF_DIR/results/${test_name}")" ]; then
        log_info "记录基线结果..."
        RUST_LOG=info cargo run --release -- \
            --extension "$EXTENSION" \
            --record \
            --result-dir "$PERF_DIR/results" \
            --host "$HOST" \
            --port "$PORT" \
            --user "$USER" \
            --passwd "$PASSWD" \
            "$test_dir"
    fi
    
    # 运行性能分析
    log_info "生成火焰图..."
    perf record -g --call-graph dwarf -- \
        cargo run --release -- \
            --extension "$EXTENSION" \
            --parallel "$parallel_count" \
            --result-dir "$PERF_DIR/results" \
            --host "$HOST" \
            --port "$PORT" \
            --user "$USER" \
            --passwd "$PASSWD" \
            "$test_dir"
    
    # 生成火焰图
    perf script | flamegraph > "$FLAMEGRAPH_DIR/flamegraph_${test_name}.svg"
    
    # 清理临时文件
    rm -f perf.data
    
    log_info "火焰图已生成: $FLAMEGRAPH_DIR/flamegraph_${test_name}.svg"
}

# 主函数
main() {
    local test_type=${1:-all}
    
    log_info "开始并发性能分析..."
    log_info "项目目录: $PROJECT_ROOT"
    log_info "性能分析目录: $PERF_DIR"
    
    check_tools
    build_project
    
    case $test_type in
        basic|low)
            run_perf_test "concurrent_basic" "$PERF_DIR/test_cases/concurrent_basic" 4 "低并发测试 (4线程)"
            ;;
        medium)
            run_perf_test "concurrent_medium" "$PERF_DIR/test_cases/concurrent_medium" 8 "中并发测试 (8线程)"
            ;;
        high)
            run_perf_test "concurrent_high" "$PERF_DIR/test_cases/concurrent_high" 16 "高并发测试 (16线程)"
            ;;
        all)
            run_perf_test "concurrent_basic" "$PERF_DIR/test_cases/concurrent_basic" 4 "低并发测试 (4线程)"
            run_perf_test "concurrent_medium" "$PERF_DIR/test_cases/concurrent_medium" 8 "中并发测试 (8线程)"
            run_perf_test "concurrent_high" "$PERF_DIR/test_cases/concurrent_high" 16 "高并发测试 (16线程)"
            ;;
        *)
            log_error "未知的测试类型: $test_type"
            log_info "用法: $0 [basic|medium|high|all]"
            exit 1
            ;;
    esac
    
    log_info "性能分析完成!"
    log_info "火焰图位置: $FLAMEGRAPH_DIR/"
}

# 显示帮助信息
show_help() {
    cat << EOF
文件级并发性能分析脚本

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
    $0                                    # 运行所有测试
    $0 basic                              # 只运行低并发测试
    HOST=192.168.1.100 $0 medium          # 使用自定义数据库运行中并发测试
    
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