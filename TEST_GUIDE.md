# 博客系统测试指南

## 概述

本指南提供了如何运行博客系统测试的详细说明。测试分为两类：

1. **单元测试**: 测试单个模块或函数的功能
2. **集成测试**: 测试多个组件之间的交互

## 测试结构

```
tests/
├── common/
│   └── mod.rs              # 测试公共工具函数
└── user_registration_test.rs # 用户注册流程集成测试

src/services/
├── auth_test.rs            # 认证服务单元测试
└── posts_test.rs           # 文章服务单元测试 (已存在)
```

## 运行测试

### 方法1: 使用测试脚本

#### Linux/macOS
```bash
# 运行所有测试
./run_tests.sh

# 只运行单元测试
./run_tests.sh unit

# 只运行集成测试
./run_tests.sh integration
```

#### Windows
```cmd
REM 运行所有测试
run_tests.bat

REM 只运行单元测试
run_tests.bat unit

REM 只运行集成测试
run_tests.bat integration
```

### 方法2: 使用Cargo命令

```bash
# 运行所有测试
cargo test

# 只运行单元测试
cargo test --lib

# 只运行集成测试
cargo test --test '*'

# 运行特定测试
cargo test test_register_user_success

# 显示测试输出
cargo test -- --nocapture

# 运行测试并生成覆盖率报告
cargo tarpaulin --out Html
```

## 测试覆盖范围

### 用户管理测试

1. **用户注册**:
   - 成功创建用户
   - 用户名太短
   - 密码太短
   - 用户名重复
   - 验证密码是否正确哈希
   - 验证用户角色分配

2. **用户登录**:
   - 密码登录成功
   - 无效凭据
   - Cookie设置
   - 邮箱登录

3. **用户查询**:
   - 获取用户列表
   - 按ID获取用户
   - 按用户名获取用户
   - 分页查询

4. **用户管理**:
   - 更新用户信息
   - 删除用户

## 测试环境

测试使用SQLite内存数据库，确保测试之间相互隔离，不会影响实际数据。

### 环境变量

测试运行时会设置以下环境变量：

```bash
RUST_LOG=debug           # 日志级别
DATABASE_URL=sqlite::memory:  # 使用内存数据库
```

## 编写新测试

### 单元测试

单元测试应该放在对应模块的`*_test.rs`文件中。例如，测试认证服务应放在`src/services/auth_test.rs`。

基本结构：

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use // ... 其他导入

    #[tokio::test]
    async fn test_function_name() {
        // 测试设置
        // ...

        // 执行操作
        let result = function_under_test();

        // 验证结果
        assert!(result.is_ok());
    }
}
```

### 集成测试

集成测试应放在`tests/`目录中。

基本结构：

```rust
use actix_web::{http, test};
use serde_json::json;

mod common;
use common::{create_test_app, create_test_user};

#[actix_web::test]
async fn test_api_endpoint() {
    let app = create_test_app().await;

    // 准备请求数据
    let request_data = json!({
        "field1": "value1",
        "field2": "value2"
    });

    // 发送请求
    let req = test::TestRequest::post()
        .uri("/api/v1/endpoint")
        .set_json(request_data)
        .to_request();

    let resp = test::call_service(&app, req).await;

    // 验证响应
    assert_eq!(resp.status(), http::StatusCode::OK);
}
```

## 测试最佳实践

1. **隔离测试**: 每个测试应该独立，不依赖其他测试的状态
2. **使用描述性名称**: 测试名称应清楚描述测试的内容
3. **Arrange-Act-Assert模式**: 组织测试代码，先设置环境，再执行操作，最后验证结果
4. **测试边界情况**: 不仅测试正常情况，还要测试边界和异常情况
5. **使用模拟对象**: 对于外部依赖（如数据库、网络），使用模拟对象进行测试
6. **保持测试快速**: 单元测试应该快速运行，集成测试可以稍慢
7. **定期运行测试**: 在提交代码前运行测试，确保没有破坏现有功能

## 故障排除

### 测试失败

如果测试失败，可以：

1. 使用`--nocapture`标志查看测试输出：
   ```bash
   cargo test -- --nocapture
   ```

2. 只运行失败的测试：
   ```bash
   cargo test failed_test_name
   ```

3. 启用详细日志：
   ```bash
   RUST_LOG=debug cargo test -- --nocapture
   ```

### 数据库连接问题

如果遇到数据库连接问题，确保：

1. 安装了SQLite
2. 检查环境变量`DATABASE_URL`是否正确设置

### 测试性能问题

如果测试运行缓慢，可以考虑：

1. 使用`--test-threads=1`标志串行运行测试，避免数据库冲突
2. 使用共享测试数据库，避免为每个测试创建新的数据库连接

## 贡献测试

当添加新功能时，请确保：

1. 为新功能添加相应的单元测试
2. 如果新功能涉及API端点，添加集成测试
3. 确保所有测试通过后再提交代码
4. 保持高测试覆盖率

## 参考资料

- [Rust测试指南](https://doc.rust-lang.org/book/ch11-00-testing.html)
- [Actix-web测试指南](https://actix.rs/docs/testing)
- [Sea-ORM测试指南](https://www.sea-ql.org/SeaORM/docs/basic-crud/#unit-testing)
