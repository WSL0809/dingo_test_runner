#!/bin/bash

# 验证迁移后的文件完整性

echo "验证测试文件迁移..."

find . -name "*.test" | wc -l | xargs echo "新结构测试文件总数:"
find ../t -name "*.test" -not -path "*/demo_tests/*" -not -path "*/br/*" | wc -l | xargs echo "原结构主要测试文件数:"

echo ""
echo "各分类文件数量:"
for dir in */; do
    if [ -d "$dir" ] && [ "$dir" != "r_new/" ] && [ "$dir" != "include/" ]; then
        count=$(find "$dir" -name "*.test" | wc -l)
        echo "  $(basename $dir): $count files"
    fi
done

echo ""
echo "检查是否有遗漏的文件..."
find ../t -name "*.test" -not -path "*/demo_tests/*" -not -path "*/br/*" -not -path "*/include/*" | while read file; do
    filename=$(basename "$file")
    if ! find . -name "$filename" -type f > /dev/null 2>&1; then
        echo "警告: $filename 可能未被迁移"
    fi
done

echo "验证完成"
