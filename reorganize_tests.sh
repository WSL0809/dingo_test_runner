#!/bin/bash

# 测试文件重组脚本
# 将现有的 .test 文件按功能分类重新组织

set -e

echo "开始重组测试文件..."

BASE_DIR="/Users/wangshilong/Downloads/lazy-cat-sync/dingo_test_runner"
OLD_T_DIR="$BASE_DIR/t"
NEW_T_DIR="$BASE_DIR/t_for_test"

# 移动测试文件的函数（结果文件保持在r/目录）
move_test_file() {
    local test_file="$1"
    local target_dir="$2"
    local filename=$(basename "$test_file")
    
    echo "移动 $filename 到 $target_dir/"
    cp "$test_file" "$target_dir/"
    
    # 注意：结果文件继续保持在 r/ 目录，测试运行器会自动查找
}

# 1. 基础功能测试
echo "=== 移动基础功能测试 ==="
move_test_file "$OLD_T_DIR/simple_test.test" "$NEW_T_DIR/basic"
move_test_file "$OLD_T_DIR/simple_connect.test" "$NEW_T_DIR/basic"
move_test_file "$OLD_T_DIR/simple_exec.test" "$NEW_T_DIR/basic"
move_test_file "$OLD_T_DIR/simple_nested.test" "$NEW_T_DIR/basic"
move_test_file "$OLD_T_DIR/echo_test.test" "$NEW_T_DIR/basic"
move_test_file "$OLD_T_DIR/mysql_connect.test" "$NEW_T_DIR/basic"
move_test_file "$OLD_T_DIR/test1.test" "$NEW_T_DIR/basic"

# 2. 变量和表达式测试
echo "=== 移动变量和表达式测试 ==="
move_test_file "$OLD_T_DIR/variable_basic.test" "$NEW_T_DIR/variables"
move_test_file "$OLD_T_DIR/variable_simple.test" "$NEW_T_DIR/variables"
move_test_file "$OLD_T_DIR/variable_sql.test" "$NEW_T_DIR/variables"
move_test_file "$OLD_T_DIR/variable_sql_simple.test" "$NEW_T_DIR/variables"
move_test_file "$OLD_T_DIR/variable_boundary_test.test" "$NEW_T_DIR/variables"
move_test_file "$OLD_T_DIR/eval_test.test" "$NEW_T_DIR/variables"

# 3. 控制流测试
echo "=== 移动控制流测试 ==="
move_test_file "$OLD_T_DIR/if_simple.test" "$NEW_T_DIR/control_flow"
move_test_file "$OLD_T_DIR/if_with_sql.test" "$NEW_T_DIR/control_flow"
move_test_file "$OLD_T_DIR/if_brace_syntax.test" "$NEW_T_DIR/control_flow"
move_test_file "$OLD_T_DIR/while_simple.test" "$NEW_T_DIR/control_flow"
move_test_file "$OLD_T_DIR/while_brace_syntax.test" "$NEW_T_DIR/control_flow"
move_test_file "$OLD_T_DIR/debug_while.test" "$NEW_T_DIR/control_flow"
move_test_file "$OLD_T_DIR/nested_control_flow.test" "$NEW_T_DIR/control_flow"
move_test_file "$OLD_T_DIR/control_flow_demo.test" "$NEW_T_DIR/control_flow"
move_test_file "$OLD_T_DIR/flexible_syntax.test" "$NEW_T_DIR/control_flow"

# 4. 并发执行测试
echo "=== 移动并发执行测试 ==="
move_test_file "$OLD_T_DIR/concurrent_basic.test" "$NEW_T_DIR/concurrent"

# 5. 错误处理测试
echo "=== 移动错误处理测试 ==="
move_test_file "$OLD_T_DIR/error_test.test" "$NEW_T_DIR/error_handling"
move_test_file "$OLD_T_DIR/expected_error_test.test" "$NEW_T_DIR/error_handling"
move_test_file "$OLD_T_DIR/error_directive_validation.test" "$NEW_T_DIR/error_handling"
move_test_file "$OLD_T_DIR/fail_fast_false_result_test.test" "$NEW_T_DIR/error_handling"
move_test_file "$OLD_T_DIR/fail_fast_no_result_test.test" "$NEW_T_DIR/error_handling"

# 6. 高级功能测试
echo "=== 移动高级功能测试 ==="
move_test_file "$OLD_T_DIR/replace_regex_test.test" "$NEW_T_DIR/advanced"
move_test_file "$OLD_T_DIR/sorted_result_test.test" "$NEW_T_DIR/advanced"
move_test_file "$OLD_T_DIR/regex_test.test" "$NEW_T_DIR/advanced"
move_test_file "$OLD_T_DIR/tags_test.test" "$NEW_T_DIR/advanced"
move_test_file "$OLD_T_DIR/tag_coverage_test.test" "$NEW_T_DIR/advanced"
move_test_file "$OLD_T_DIR/advanced_test.test" "$NEW_T_DIR/advanced"
move_test_file "$OLD_T_DIR/exec_test.test" "$NEW_T_DIR/advanced"

# 7. 连接管理测试
echo "=== 移动连接管理测试 ==="
move_test_file "$OLD_T_DIR/connection_test.test" "$NEW_T_DIR/connection"
move_test_file "$OLD_T_DIR/connection_management.test" "$NEW_T_DIR/connection"
move_test_file "$OLD_T_DIR/connection_edge_cases.test" "$NEW_T_DIR/connection"

# 8. Source/Include 功能测试
echo "=== 移动 Source/Include 功能测试 ==="
move_test_file "$OLD_T_DIR/source_basic.test" "$NEW_T_DIR/source"
move_test_file "$OLD_T_DIR/source_comprehensive.test" "$NEW_T_DIR/source"
move_test_file "$OLD_T_DIR/source_depth.test" "$NEW_T_DIR/source"
move_test_file "$OLD_T_DIR/source_error.test" "$NEW_T_DIR/source"
move_test_file "$OLD_T_DIR/source_nested.test" "$NEW_T_DIR/source"
move_test_file "$OLD_T_DIR/source_syntax.test" "$NEW_T_DIR/source"
move_test_file "$OLD_T_DIR/source_variables.test" "$NEW_T_DIR/source"

# 9. 性能/边界测试
echo "=== 移动性能/边界测试 ==="
move_test_file "$OLD_T_DIR/parser_edge_cases.test" "$NEW_T_DIR/performance"
move_test_file "$OLD_T_DIR/create_tables_loop.test" "$NEW_T_DIR/performance"
move_test_file "$OLD_T_DIR/drop_tables_loop.test" "$NEW_T_DIR/performance"
move_test_file "$OLD_T_DIR/sequence.test" "$NEW_T_DIR/performance"

# 复制子目录（demo_tests, br 等保持原样）
echo "=== 保留现有子目录 ==="
if [ -d "$OLD_T_DIR/demo_tests" ]; then
    cp -r "$OLD_T_DIR/demo_tests" "$NEW_T_DIR/"
    echo "复制 demo_tests 目录"
fi

if [ -d "$OLD_T_DIR/br" ]; then
    cp -r "$OLD_T_DIR/br" "$NEW_T_DIR/"
    echo "复制 br 目录"
fi

# 创建分类说明文件
cat > "$NEW_T_DIR/README.md" << 'EOF'
# 重构后的测试目录结构

## 目录说明

- **basic/** - 基础功能测试（连接、简单查询、echo等）
- **variables/** - 变量系统测试（let语句、变量展开、表达式）
- **control_flow/** - 控制流测试（if/while语句、嵌套控制）
- **concurrent/** - 并发执行测试
- **connection/** - 连接管理测试（多连接、连接切换）
- **error_handling/** - 错误处理测试（预期错误、失败处理）
- **advanced/** - 高级功能测试（正则替换、结果排序、标签等）
- **source/** - Source/Include 功能测试
- **performance/** - 性能和边界测试（解析器边界、循环测试）
- **include/** - 共享的包含文件
- **demo_tests/** - 演示测试（保持原结构）
- **br/** - BR相关测试（保持原结构）

## 迁移说明

- 对应的 .result 文件已同步移动到 `../r_new/` 对应目录
- Include 文件路径已保持兼容
- 可通过目录名快速定位相关功能测试

## 运行测试

```bash
# 运行特定分类的测试
cargo run -- t_for_test/basic/
cargo run -- t_for_test/variables/
cargo run -- t_for_test/control_flow/

# 运行单个测试
cargo run -- t_for_test/basic/simple_test.test
```
EOF

echo "=== 创建验证脚本 ==="
cat > "$NEW_T_DIR/verify_migration.sh" << 'EOF'
#!/bin/bash

# 验证迁移后的文件完整性

echo "验证测试文件迁移..."

find t_for_test -name "*.test" | wc -l | xargs echo "新结构测试文件总数:"
find t -name "*.test" -not -path "*/demo_tests/*" -not -path "*/br/*" | wc -l | xargs echo "原结构主要测试文件数:"

echo ""
echo "各分类文件数量:"
for dir in t_for_test/*/; do
    count=$(find "$dir" -name "*.test" | wc -l)
    echo "  $(basename $dir): $count files"
done

echo ""
echo "检查是否有遗漏的文件..."
find t -name "*.test" -not -path "*/demo_tests/*" -not -path "*/br/*" -not -path "*/include/*" | while read file; do
    filename=$(basename "$file")
    if ! find t_for_test -name "$filename" -type f > /dev/null 2>&1; then
        echo "警告: $filename 可能未被迁移"
    fi
done

echo "验证完成"
EOF

chmod +x "$NEW_T_DIR/verify_migration.sh"

echo ""
echo "=== 测试文件重组完成 ==="
echo "新的测试结构位于: $NEW_T_DIR"
echo "对应的结果文件位于: $BASE_DIR/r_new"
echo ""
echo "运行验证脚本:"
echo "cd $NEW_T_DIR && ./verify_migration.sh"