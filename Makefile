# Makefile for dingo_test_runner
# 
# Common development tasks for MySQL test runner

# Default variables
RUST_LOG ?= info
HOST ?= 127.0.0.1
PORT ?= 3306
USER ?= root
PASSWD ?= 123456
EXTENSION ?= dev
PARALLEL ?= 4

# Build targets
.PHONY: build release clean test check fmt clippy

build:
	cargo build

release:
	cargo build --release

clean:
	cargo clean

# Rust development
test:
	cargo test

check:
	cargo check

fmt:
	cargo fmt

clippy:
	cargo clippy

# Database connection string
DB_ARGS = --host $(HOST) --port $(PORT) --user $(USER) --passwd $(PASSWD)

# Development testing targets
.PHONY: dev-test dev-record dev-all integration-test integration-record

# Development testing with custom extension
dev-test:
	RUST_LOG=$(RUST_LOG) cargo run -- --extension $(EXTENSION) $(DB_ARGS) $(ARGS)

# Record development baselines
dev-record:
	RUST_LOG=$(RUST_LOG) cargo run -- --extension $(EXTENSION) --record $(DB_ARGS) $(ARGS)

# Run all development tests with parallel execution
dev-all:
	RUST_LOG=$(RUST_LOG) cargo run -- --extension $(EXTENSION) --parallel $(PARALLEL) $(DB_ARGS) tests/integration/

# Integration testing
integration-test:
	RUST_LOG=$(RUST_LOG) cargo run -- --extension integration $(DB_ARGS) tests/integration/

integration-record:
	RUST_LOG=$(RUST_LOG) cargo run -- --extension integration --record $(DB_ARGS) tests/integration/

# User testing targets (default extension)
.PHONY: user-test user-record user-examples

user-test:
	RUST_LOG=$(RUST_LOG) cargo run -- $(DB_ARGS) $(ARGS)

user-record:
	RUST_LOG=$(RUST_LOG) cargo run -- --record $(DB_ARGS) $(ARGS)

# Run example tests for users
user-examples:
	RUST_LOG=$(RUST_LOG) cargo run -- $(DB_ARGS) t/examples/

# Parallel execution targets
.PHONY: parallel-test parallel-all

parallel-test:
	RUST_LOG=$(RUST_LOG) cargo run -- --extension $(EXTENSION) --parallel $(PARALLEL) $(DB_ARGS) $(ARGS)

parallel-all:
	RUST_LOG=$(RUST_LOG) cargo run -- --extension $(EXTENSION) --parallel $(PARALLEL) $(DB_ARGS) --all

# Report generation targets
.PHONY: html-report junit-report allure-report

html-report:
	RUST_LOG=$(RUST_LOG) cargo run -- --extension $(EXTENSION) --report-format html $(DB_ARGS) $(ARGS)

junit-report:
	RUST_LOG=$(RUST_LOG) cargo run -- --extension $(EXTENSION) --report-format xunit --xunit-file report.xml $(DB_ARGS) $(ARGS)

allure-report:
	RUST_LOG=$(RUST_LOG) cargo run -- --extension $(EXTENSION) --report-format allure --allure-dir ./allure-results $(DB_ARGS) $(ARGS)

# Debug targets
.PHONY: debug trace

debug:
	RUST_LOG=debug cargo run -- --extension $(EXTENSION) $(DB_ARGS) $(ARGS)

trace:
	RUST_LOG=trace cargo run -- --extension $(EXTENSION) $(DB_ARGS) $(ARGS)

# Specific component debugging
debug-parser:
	RUST_LOG=dingo_test_runner::tester::pest_parser=debug cargo run -- --extension $(EXTENSION) $(DB_ARGS) $(ARGS)

# Quick test targets for common scenarios
.PHONY: quick-basic quick-variables quick-concurrent quick-examples

quick-basic:
	RUST_LOG=$(RUST_LOG) cargo run -- --extension $(EXTENSION) $(DB_ARGS) tests/integration/basic/

quick-variables:
	RUST_LOG=$(RUST_LOG) cargo run -- --extension $(EXTENSION) $(DB_ARGS) tests/integration/variables/

quick-concurrent:
	RUST_LOG=$(RUST_LOG) cargo run -- --extension $(EXTENSION) $(DB_ARGS) tests/integration/concurrent/

quick-examples:
	RUST_LOG=$(RUST_LOG) cargo run -- $(DB_ARGS) t/examples/

# Development workflow targets
.PHONY: dev-setup dev-baseline dev-verify

# Setup development environment (record all baselines)
dev-setup:
	@echo "Setting up development baselines..."
	$(MAKE) dev-record ARGS="tests/integration/"
	@echo "Development setup complete!"

# Create baseline for specific test
dev-baseline:
	@echo "Recording baseline for: $(ARGS)"
	$(MAKE) dev-record ARGS="$(ARGS)"

# Verify against baseline
dev-verify:
	@echo "Verifying against baseline: $(ARGS)"
	$(MAKE) dev-test ARGS="$(ARGS)"

# CI/CD targets
.PHONY: ci ci-test ci-build

ci: ci-build ci-test

ci-build:
	cargo build --release

ci-test:
	cargo test
	RUST_LOG=$(RUST_LOG) cargo run -- --extension ci --parallel $(PARALLEL) $(DB_ARGS) tests/integration/

# Help target
.PHONY: help

help:
	@echo "Available targets:"
	@echo ""
	@echo "Build targets:"
	@echo "  build         - Build debug version"
	@echo "  release       - Build release version"
	@echo "  clean         - Clean build artifacts"
	@echo "  test          - Run Rust unit tests"
	@echo "  check         - Run cargo check"
	@echo "  fmt           - Format code"
	@echo "  clippy        - Run clippy lints"
	@echo ""
	@echo "Development testing:"
	@echo "  dev-test      - Run tests with dev extension"
	@echo "  dev-record    - Record dev baselines"
	@echo "  dev-all       - Run all integration tests (parallel)"
	@echo "  dev-setup     - Setup development baselines"
	@echo ""
	@echo "User testing:"
	@echo "  user-test     - Run tests with default extension"
	@echo "  user-record   - Record user baselines"
	@echo "  user-examples - Run example tests"
	@echo ""
	@echo "Integration testing:"
	@echo "  integration-test   - Run integration tests"
	@echo "  integration-record - Record integration baselines"
	@echo ""
	@echo "Parallel execution:"
	@echo "  parallel-test - Run tests in parallel"
	@echo "  parallel-all  - Run all tests in parallel"
	@echo ""
	@echo "Quick tests:"
	@echo "  quick-basic      - Test basic functionality"
	@echo "  quick-variables  - Test variable system"
	@echo "  quick-concurrent - Test concurrent features"
	@echo "  quick-examples   - Test user examples"
	@echo ""
	@echo "Reports:"
	@echo "  html-report   - Generate HTML report"
	@echo "  junit-report  - Generate JUnit XML report"
	@echo "  allure-report - Generate Allure report"
	@echo ""
	@echo "Debugging:"
	@echo "  debug         - Run with debug logging"
	@echo "  trace         - Run with trace logging"
	@echo "  debug-parser  - Debug parser specifically"
	@echo ""
	@echo "CI/CD:"
	@echo "  ci           - Full CI pipeline"
	@echo "  ci-build     - CI build step"
	@echo "  ci-test      - CI test step"
	@echo ""
	@echo "Variables (can be overridden):"
	@echo "  HOST=$(HOST)       - Database host"
	@echo "  PORT=$(PORT)       - Database port"
	@echo "  USER=$(USER)       - Database user"
	@echo "  PASSWD=$(PASSWD)   - Database password"
	@echo "  EXTENSION=$(EXTENSION) - Test extension"
	@echo "  PARALLEL=$(PARALLEL)   - Parallel threads"
	@echo "  RUST_LOG=$(RUST_LOG)   - Log level"
	@echo ""
	@echo "Examples:"
	@echo "  make dev-test ARGS='t/examples/basic_example.test'"
	@echo "  make dev-record ARGS='tests/integration/basic/'"
	@echo "  make parallel-test PARALLEL=8 ARGS='tests/integration/'"
	@echo "  make debug ARGS='t/examples/basic_example.test'"
	@echo "  make user-test HOST=192.168.1.100 PASSWD=mypass ARGS='t/demo_tests/'"