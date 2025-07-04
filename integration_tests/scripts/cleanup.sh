#!/bin/bash

# 集成测试环境清理脚本
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

echo "🧹 Cleaning up integration test environment..."

cd "$PROJECT_ROOT"

# 停止并清理Docker容器
echo "🛑 Stopping test containers..."
docker-compose -f docker-compose.test.yml down -v --remove-orphans 2>/dev/null || true

# 清理Docker资源
echo "🗑️  Removing test volumes and networks..."
docker volume prune -f 2>/dev/null || true
docker network prune -f 2>/dev/null || true

# 清理测试结果文件
echo "📄 Cleaning up test result files..."
find r/ -name "*.smoke" -delete 2>/dev/null || true
find integration_tests/ -name "*.result" -delete 2>/dev/null || true
find integration_tests/ -name "*_report.txt" -delete 2>/dev/null || true

echo "✅ Cleanup completed!"