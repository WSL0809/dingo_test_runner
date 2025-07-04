#!/bin/bash

# 集成测试环境设置脚本
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

echo "🚀 Setting up integration test environment..."

# 检查Docker和Docker Compose是否可用
check_docker() {
    if ! command -v docker &> /dev/null; then
        echo "❌ Docker is not installed or not in PATH"
        exit 1
    fi
    
    if ! command -v docker-compose &> /dev/null; then
        echo "❌ Docker Compose is not installed or not in PATH"
        exit 1
    fi
    
    echo "✅ Docker and Docker Compose are available"
}

# 构建测试环境
setup_test_environment() {
    echo "📦 Starting test containers..."
    
    cd "$PROJECT_ROOT"
    
    # 停止并清理现有容器
    docker-compose -f docker-compose.test.yml down -v --remove-orphans 2>/dev/null || true
    
    # 启动测试容器
    docker-compose -f docker-compose.test.yml up -d
    
    echo "⏳ Waiting for services to be ready..."
    
    # 等待MySQL就绪
    local retry_count=0
    local max_retries=30
    
    while [ $retry_count -lt $max_retries ]; do
        if docker exec dingo_test_mysql mysqladmin ping -h localhost -u root -p'test123456' --silent; then
            echo "✅ MySQL is ready"
            break
        fi
        
        retry_count=$((retry_count + 1))
        echo "⏳ Waiting for MySQL... ($retry_count/$max_retries)"
        sleep 2
    done
    
    if [ $retry_count -eq $max_retries ]; then
        echo "❌ MySQL failed to start within timeout"
        docker-compose -f docker-compose.test.yml logs mysql-test
        exit 1
    fi
    
    # 测试连接
    echo "🔍 Testing database connections..."
    
    # 测试MySQL连接
    if docker exec dingo_test_mysql mysql -h localhost -u root -p'test123456' -e "SELECT 1;" >/dev/null 2>&1; then
        echo "✅ MySQL connection successful"
    else
        echo "❌ MySQL connection failed"
        docker exec dingo_test_mysql mysql -h localhost -u root -p'test123456' -e "SELECT 1;" || true
        exit 1
    fi
    
    echo "🎉 Test environment is ready!"
    echo ""
    echo "📊 Service Information:"
    echo "  MySQL: localhost:13306"
    echo "    - User: root"
    echo "    - Password: test123456"
    echo "    - Database: test_db"
    echo ""
    echo "  DingoDB: localhost:14000"
    echo "    - Compatible with MySQL protocol"
    echo ""
}

# 验证测试环境
verify_environment() {
    echo "🔍 Verifying test environment..."
    
    # 检查Rust工具链
    if ! command -v cargo &> /dev/null; then
        echo "❌ Rust/Cargo is not installed"
        exit 1
    fi
    
    echo "✅ Rust toolchain is available"
    
    # 编译项目
    echo "🔨 Building project..."
    cd "$PROJECT_ROOT"
    if cargo build --release; then
        echo "✅ Project build successful"
    else
        echo "❌ Project build failed"
        exit 1
    fi
}

# 主函数
main() {
    echo "🧪 Dingo Test Runner - Integration Test Environment Setup"
    echo "========================================================"
    
    check_docker
    setup_test_environment
    verify_environment
    
    echo ""
    echo "🎉 Integration test environment setup complete!"
    echo ""
    echo "Next steps:"
    echo "  1. Run smoke tests: ./integration_tests/smoke/run_smoke.sh"
    echo "  2. Run all tests: ./integration_tests/scripts/run_tests.sh --all"
    echo "  3. Cleanup: ./integration_tests/scripts/cleanup.sh"
}

# 执行主函数
main "$@"