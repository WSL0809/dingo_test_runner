#!/bin/bash

# 分类测试运行脚本
# 用于验证重构后的测试结构是否正常工作

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 显示帮助信息
show_help() {
    echo "测试运行脚本"
    echo ""
    echo "用法: $0 [选项] [分类]"
    echo ""
    echo "选项:"
    echo "  -h, --help     显示帮助信息"
    echo "  -a, --all      运行所有分类的测试"
    echo "  -l, --list     列出所有可用的测试分类"
    echo "  -v, --verify   验证新结构与旧结构的兼容性"
    echo "  --dry-run      只显示将要运行的测试，不实际执行"
    echo ""
    echo "分类:"
    echo "  basic          基础功能测试"
    echo "  variables      变量和表达式测试"  
    echo "  control_flow   控制流测试"
    echo "  connection     连接管理测试"
    echo "  advanced       高级功能测试"
    echo "  error_handling 错误处理测试"
    echo "  source         Source/Include功能测试"
    echo "  performance    性能和边界测试"
    echo "  concurrent     并发执行测试"
    echo ""
    echo "示例:"
    echo "  $0 basic                    # 运行基础功能测试"
    echo "  $0 --all                    # 运行所有测试"
    echo "  $0 --list                   # 列出所有分类"
    echo "  $0 variables control_flow   # 运行多个分类"
}

# 列出所有测试分类
list_categories() {
    echo -e "${BLUE}可用的测试分类:${NC}"
    echo ""
    for dir in t_for_test/*/; do
        if [ -d "$dir" ] && [ "$(basename "$dir")" != "r_new" ] && [ "$(basename "$dir")" != "include" ]; then
            category=$(basename "$dir")
            count=$(find "$dir" -name "*.test" | wc -l | tr -d ' ')
            case $category in
                "basic")
                    desc="基础功能测试 (连接、简单查询)"
                    ;;
                "variables")
                    desc="变量和表达式测试 (let语句、变量展开)"
                    ;;
                "control_flow")
                    desc="控制流测试 (if/while语句)"
                    ;;
                "connection")
                    desc="连接管理测试 (多连接、连接切换)"
                    ;;
                "advanced")
                    desc="高级功能测试 (正则替换、结果排序)"
                    ;;
                "error_handling")
                    desc="错误处理测试 (预期错误、失败处理)"
                    ;;
                "source")
                    desc="Source/Include功能测试"
                    ;;
                "performance")
                    desc="性能和边界测试 (解析器边界、循环)"
                    ;;
                "concurrent")
                    desc="并发执行测试"
                    ;;
                "demo_tests")
                    desc="演示测试 (保持原结构)"
                    ;;
                "br")
                    desc="BR相关测试 (保持原结构)"
                    ;;
                *)
                    desc="其他测试"
                    ;;
            esac
            printf "  %-15s (%2d files) - %s\n" "$category" "$count" "$desc"
        fi
    done
}

# 运行指定分类的测试
run_category() {
    local category="$1"
    local dry_run="$2"
    
    if [ ! -d "t_for_test/$category" ]; then
        echo -e "${RED}错误: 分类 '$category' 不存在${NC}"
        return 1
    fi
    
    local test_files=($(find "t_for_test/$category" -name "*.test" | sort))
    
    if [ ${#test_files[@]} -eq 0 ]; then
        echo -e "${YELLOW}警告: 分类 '$category' 中没有测试文件${NC}"
        return 0
    fi
    
    echo -e "${BLUE}=== 运行 $category 分类测试 (${#test_files[@]} 个文件) ===${NC}"
    
    local passed=0
    local failed=0
    
    for test_file in "${test_files[@]}"; do
        local test_name=$(basename "$test_file" .test)
        
        if [ "$dry_run" = "true" ]; then
            echo "  [DRY-RUN] $test_name"
            continue
        fi
        
        echo -n "  运行 $test_name ... "
        
        # 临时修改测试路径，使用新结构
        if timeout 30s cargo run --quiet -- "$test_file" > /dev/null 2>&1; then
            echo -e "${GREEN}PASSED${NC}"
            ((passed++))
        else
            echo -e "${RED}FAILED${NC}"
            ((failed++))
        fi
    done
    
    if [ "$dry_run" != "true" ]; then
        echo ""
        if [ $failed -eq 0 ]; then
            echo -e "${GREEN}$category 分类: 全部 $passed 个测试通过${NC}"
        else
            echo -e "${RED}$category 分类: $passed 个通过, $failed 个失败${NC}"
        fi
        echo ""
    fi
    
    return $failed
}

# 验证新旧结构兼容性
verify_compatibility() {
    echo -e "${BLUE}=== 验证新旧结构兼容性 ===${NC}"
    echo ""
    
    # 检查是否有必要的目录和文件
    if [ ! -d "t_for_test" ]; then
        echo -e "${RED}错误: 新测试结构目录 t_for_test 不存在${NC}"
        return 1
    fi
    
    if [ ! -d "t" ]; then
        echo -e "${RED}错误: 原测试目录 t 不存在${NC}"
        return 1
    fi
    
    # 比较文件数量
    local old_count=$(find t -name "*.test" -not -path "*/demo_tests/*" -not -path "*/br/*" | wc -l | tr -d ' ')
    local new_main_count=$(find t_for_test -name "*.test" -not -path "*/demo_tests/*" -not -path "*/br/*" | wc -l | tr -d ' ')
    
    echo "原结构主要测试文件数: $old_count"
    echo "新结构主要测试文件数: $new_main_count"
    
    if [ "$old_count" -eq "$new_main_count" ]; then
        echo -e "${GREEN}✓ 文件数量匹配${NC}"
    else
        echo -e "${YELLOW}⚠ 文件数量不匹配，可能需要检查${NC}"
    fi
    
    # 检查是否有遗漏的文件
    echo ""
    echo "检查遗漏的文件..."
    local missing_files=0
    
    find t -name "*.test" -not -path "*/demo_tests/*" -not -path "*/br/*" -not -path "*/include/*" | while read file; do
        filename=$(basename "$file")
        if ! find t_for_test -name "$filename" -type f > /dev/null 2>&1; then
            echo -e "${RED}⚠ 遗漏文件: $filename${NC}"
            ((missing_files++))
        fi
    done
    
    if [ $missing_files -eq 0 ]; then
        echo -e "${GREEN}✓ 没有遗漏的文件${NC}"
    fi
    
    echo ""
    echo -e "${GREEN}兼容性验证完成${NC}"
}

# 主函数
main() {
    local categories=()
    local run_all=false
    local dry_run=false
    local total_failed=0
    
    # 解析命令行参数
    while [[ $# -gt 0 ]]; do
        case $1 in
            -h|--help)
                show_help
                exit 0
                ;;
            -a|--all)
                run_all=true
                shift
                ;;
            -l|--list)
                list_categories
                exit 0
                ;;
            -v|--verify)
                verify_compatibility
                exit 0
                ;;
            --dry-run)
                dry_run=true
                shift
                ;;
            -*)
                echo -e "${RED}未知选项: $1${NC}"
                show_help
                exit 1
                ;;
            *)
                categories+=("$1")
                shift
                ;;
        esac
    done
    
    # 如果没有指定分类且不是运行全部，显示帮助
    if [ ${#categories[@]} -eq 0 ] && [ "$run_all" = false ]; then
        show_help
        exit 0
    fi
    
    # 确保构建了二进制文件
    if [ "$dry_run" != "true" ]; then
        echo -e "${BLUE}构建测试运行器...${NC}"
        if ! cargo build --quiet --bin dingo_test_runner; then
            echo -e "${RED}构建失败${NC}"
            exit 1
        fi
    fi
    
    # 运行所有分类
    if [ "$run_all" = true ]; then
        categories=(basic variables control_flow connection advanced error_handling source performance concurrent)
    fi
    
    # 运行指定的分类
    for category in "${categories[@]}"; do
        if ! run_category "$category" "$dry_run"; then
            ((total_failed++))
        fi
    done
    
    # 显示总结
    if [ "$dry_run" != "true" ] && [ ${#categories[@]} -gt 1 ]; then
        echo -e "${BLUE}=== 测试总结 ===${NC}"
        if [ $total_failed -eq 0 ]; then
            echo -e "${GREEN}所有分类测试通过！${NC}"
        else
            echo -e "${RED}有 $total_failed 个分类测试失败${NC}"
            exit 1
        fi
    fi
}

main "$@"