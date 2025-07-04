# 集成测试框架

## 目录结构

```
integration_tests/
├── README.md                    # 本文件
├── docker/                     # Docker相关配置
│   ├── init/                   # 数据库初始化脚本
│   └── Dockerfile.test         # 测试环境镜像
├── smoke/                      # 冒烟测试套件
│   ├── tests/                  # 测试文件(.test)
│   ├── expected/               # 期望结果(.result)
│   └── run_smoke.sh           # 冒烟测试执行脚本
├── functional/                 # 功能测试套件
│   ├── parser/                 # 解析器功能测试
│   ├── executor/               # 执行器功能测试
│   ├── database/               # 数据库功能测试
│   └── cli/                    # CLI功能测试
├── regression/                 # 回归测试套件
│   ├── bugs/                   # 历史bug防护测试
│   └── compatibility/          # 兼容性回归测试
├── performance/                # 性能测试套件
│   ├── concurrency/            # 并发性能测试
│   └── load/                   # 负载测试
├── scripts/                    # 测试脚本
│   ├── setup_env.sh           # 环境设置脚本
│   ├── run_tests.sh           # 测试执行脚本
│   ├── cleanup.sh             # 清理脚本
│   └── generate_report.sh     # 报告生成脚本
└── config/                     # 测试配置
    ├── mysql.conf             # MySQL测试配置
    ├── dingo.conf             # DingoDB测试配置
    └── test_matrix.yaml       # 测试矩阵配置
```

## 设计原则

### 1. 完全隔离
- 与生产测试目录 `t/` 和 `r/` 完全分离
- 使用独立的数据库实例进行测试
- 测试环境容器化，可重复部署

### 2. 分层测试
- **冒烟测试**: 15分钟内验证核心功能
- **功能测试**: 1小时内验证所有功能模块
- **回归测试**: 防止功能倒退
- **性能测试**: 验证性能指标

### 3. 多环境支持
- MySQL 8.0 标准环境
- DingoDB 兼容性环境
- 并发测试专用环境

## 快速开始

### 1. 启动测试环境
```bash
# 启动Docker测试环境
docker-compose -f docker-compose.test.yml up -d

# 等待服务就绪
./integration_tests/scripts/wait_for_services.sh
```

### 2. 运行冒烟测试
```bash
# 15分钟核心功能验证
./integration_tests/smoke/run_smoke.sh
```

### 3. 运行完整测试套件
```bash
# 运行所有集成测试
./integration_tests/scripts/run_tests.sh --all

# 运行特定测试套件
./integration_tests/scripts/run_tests.sh --smoke
./integration_tests/scripts/run_tests.sh --functional
./integration_tests/scripts/run_tests.sh --regression
```

### 4. 清理环境
```bash
# 停止并清理测试环境
docker-compose -f docker-compose.test.yml down -v
```

## 测试环境配置

### MySQL测试环境
- 端口: 13306
- 用户: root / test123456
- 数据库: test_db

### DingoDB测试环境
- 端口: 14000
- 兼容MySQL协议

## 扩展名约定

为避免与生产测试结果冲突，集成测试使用以下扩展名：

```bash
# 冒烟测试
--extension smoke

# 功能测试
--extension func

# 回归测试
--extension regress

# 性能测试
--extension perf

# MySQL环境测试
--extension mysql

# DingoDB环境测试
--extension dingo
```

## CI/CD 集成

集成测试支持在CI/CD流水线中运行：

```yaml
# GitHub Actions 示例
name: Integration Tests
on: [push, pull_request]

jobs:
  smoke-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - name: Run Smoke Tests
        run: |
          docker-compose -f docker-compose.test.yml up -d
          ./integration_tests/scripts/wait_for_services.sh
          ./integration_tests/smoke/run_smoke.sh
          docker-compose -f docker-compose.test.yml down -v
```