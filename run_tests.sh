#!/bin/bash

# 测试运行脚本
# 使用方法: ./run_tests.sh [test_type]
# test_type: unit (单元测试), integration (集成测试), all (所有测试)

set -e

# 设置测试环境变量
export RUST_LOG=debug
export DATABASE_URL=sqlite::memory:

# 根据参数决定运行哪种测试
TEST_TYPE=${1:-all}

echo "正在运行博客系统测试..."
echo "测试类型: $TEST_TYPE"
echo "========================================"

case $TEST_TYPE in
  unit)
    echo "运行单元测试..."
    cargo test --lib -- --test-threads=1
    ;;
  integration)
    echo "运行集成测试..."
    cargo test --test '*' -- --test-threads=1
    ;;
  all)
    echo "运行所有测试..."
    cargo test -- --test-threads=1
    ;;
  *)
    echo "未知测试类型: $TEST_TYPE"
    echo "支持的测试类型: unit, integration, all"
    exit 1
    ;;
esac

echo "========================================"
echo "测试完成!"
