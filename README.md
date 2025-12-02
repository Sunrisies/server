# Rust 博客系统后端服务

<div align="center">

[![Rust](https://img.shields.io/badge/Rust-1.70+-orange.svg)](https://www.rust-lang.org/)
[![Actix-Web](https://img.shields.io/badge/Actix--Web-4.x-blue.svg)](https://actix.rs/)
[![SeaORM](https://img.shields.io/badge/SeaORM-latest-green.svg)](https://www.sea-ql.org/SeaORM/)
[![PostgreSQL](https://img.shields.io/badge/PostgreSQL-14+-blue.svg)](https://www.postgresql.org/)
[![License](https://img.shields.io/badge/license-MIT-green.svg)](LICENSE)

这是一个使用 Rust 和 Actix-web 框架构建的现代化博客后端服务，提供完整的博客管理、用户认证、权限控制、实时通信等功能。

[功能特性](#功能特性) •
[快速开始](#快速开始) •
[API文档](#api-文档) •
[架构设计](#架构设计) •
[配置说明](#配置说明)

</div>

---

## 📋 目录

- [技术栈](#技术栈)
- [功能特性](#功能特性)
- [快速开始](#快速开始)
- [项目结构](#项目结构)
- [API文档](#api-文档)
- [架构设计](#架构设计)
- [配置说明](#配置说明)
- [开发指南](#开发指南)
- [部署](#部署)
- [测试](#测试)
- [贡献指南](#贡献指南)
- [许可证](#许可证)

> 💡 **提示**: 查看 [📖 文档索引](./DOCUMENTATION_INDEX.md) 获取完整的文档导航

## 🚀 技术栈

### 核心框架
- **Web框架**: [Actix-Web 4.x](https://actix.rs/) - 高性能异步Web框架
- **ORM**: [SeaORM](https://www.sea-ql.org/SeaORM/) - 类型安全的异步ORM
- **数据库**: [PostgreSQL](https://www.postgresql.org/) - 强大的关系型数据库
- **运行时**: [Tokio](https://tokio.rs/) - 异步运行时

### 认证与安全
- **JWT**: [jsonwebtoken](https://github.com/Keats/jsonwebtoken) - JWT令牌生成和验证
- **密码加密**: [Argon2](https://github.com/RustCrypto/password-hashes) - 安全的密码哈希算法
- **权限控制**: RBAC (基于角色的访问控制)

### 其他特性
- **序列化**: [Serde](https://serde.rs/) - 高效的序列化/反序列化
- **验证**: [Validator](https://github.com/Keats/validator) - 数据验证
- **日志**: [log4rs](https://github.com/estk/log4rs) - 强大的日志系统
- **API文档**: [utoipa](https://github.com/juhaku/utoipa) - OpenAPI文档生成
- **文件存储**: 七牛云存储
- **实时通信**: WebSocket + SSE

## ✨ 功能特性

### 用户管理
- ✅ 用户注册/登录 (支持用户名/邮箱/手机)
- ✅ JWT令牌认证
- ✅ 密码加密存储 (Argon2)
- ✅ 用户信息管理
- ✅ 头像上传
- ✅ 邮箱验证

### 权限系统
- ✅ 基于RBAC的权限控制
- ✅ 灵活的角色管理
- ✅ 细粒度权限定义
- ✅ 用户特殊权限
- ✅ 权限缓存优化

### 内容管理
- ✅ 文章CRUD操作
- ✅ 富文本/Markdown支持
- ✅ 分类管理
- ✅ 标签系统 (多对多)
- ✅ 文章搜索/过滤
- ✅ 分页查询
- ✅ 草稿/发布状态
- ✅ 精选文章
- ✅ 浏览计数

### 实时通信
- ✅ WebSocket聊天室
- ✅ SSE服务端推送
- ✅ 在线用户管理
- ✅ 消息持久化
- ✅ 文件消息支持

### 文件管理
- ✅ 图片上传 (支持压缩)
- ✅ 视频上传
- ✅ 文件类型验证
- ✅ 七牛云存储集成
- ✅ 文件大小限制

### API特性
- ✅ RESTful API设计
- ✅ OpenAPI/Swagger文档
- ✅ 统一错误处理
- ✅ 请求参数验证
- ✅ CORS支持
- ✅ 日志记录

## 🎯 快速开始

### 前置要求

- **Rust** 1.70+ ([安装指南](https://www.rust-lang.org/tools/install))
- **PostgreSQL** 14+ ([安装指南](https://www.postgresql.org/download/))
- **Cargo** (随Rust一起安装)

### 安装步骤

1. **克隆仓库**
   ```bash
   git clone [仓库地址]
   cd blog/server
   ```

2. **配置数据库**
   ```bash
   # 创建数据库
   createdb blog_db

   # 或使用 psql
   psql -U postgres
   CREATE DATABASE blog_db;
   ```

3. **配置环境变量**
   ```bash
   # 复制环境变量模板
   cp .env.example .env

   # 编辑 .env 文件，配置数据库连接等信息
   ```

   > 💡 **配置帮助**:
   > - ⚡ 快速配置（2-5分钟）: [QUICK_CONFIG.md](./QUICK_CONFIG.md)
   > - 📝 详细配置说明: [CONFIGURATION_GUIDE.md](./CONFIGURATION_GUIDE.md)
   > - 📊 参数速查表: [PARAMETERS_CHEATSHEET.md](./PARAMETERS_CHEATSHEET.md)

   `.env` 文件示例：
   ```env
   DATABASE_URL=postgresql://user:password@localhost/blog_db
   JWT_SECRET=your-secret-key-change-in-production
   RUST_LOG=info

   # 七牛云配置 (可选)
   QINIU_ACCESS_KEY=your-access-key
   QINIU_SECRET_KEY=your-secret-key
   QINIU_BUCKET=your-bucket-name
   QINIU_DOMAIN=your-domain.com

   # 邮件配置 (可选)
   SMTP_HOST=smtp.example.com
   SMTP_PORT=587
   SMTP_USERNAME=your-email@example.com
   SMTP_PASSWORD=your-password
   ```

4. **运行数据库迁移**
   ```bash
   # 安装 SeaORM CLI
   cargo install sea-orm-cli

   # 运行迁移
   sea-orm-cli migrate up
   ```

5. **构建项目**
   ```bash
   cargo build
   ```

6. **运行服务器**
   ```bash
   cargo run
   ```

   服务器将在 `http://localhost:8080` 启动

### 开发模式

使用文件监听自动重启服务器：

```bash
# 安装 watchexec
cargo install watchexec-cli

# Windows PowerShell
$env:CRUD_MACRO_DEBUG=1
watchexec -w src -w route-macros -r cargo run

# Linux/macOS
export CRUD_MACRO_DEBUG=1
watchexec -w src -w route-macros -r cargo run
```

或使用提供的开发脚本：

```bash
# Linux/macOS
./dev.sh

# Windows
.\dev.sh
```

## 📁 项目结构

```
server/
├── src/                        # 源代码目录
│   ├── main.rs                # 应用程序入口
│   ├── lib.rs                 # 核心模块导出
│   ├── config/                # 配置模块
│   │   ├── db.rs             # 数据库连接配置
│   │   ├── email.rs          # 邮件配置
│   │   ├── error.rs          # 错误处理
│   │   ├── log.rs            # 日志配置
│   │   └── api_doc.rs        # API文档配置
│   ├── models/                # 数据模型 (ORM实体)
│   │   ├── users.rs          # 用户模型
│   │   ├── posts.rs          # 文章模型
│   │   ├── categories.rs     # 分类模型
│   │   ├── tags.rs           # 标签模型
│   │   ├── roles.rs          # 角色模型
│   │   └── permissions.rs    # 权限模型
│   ├── dto/                   # 数据传输对象
│   │   ├── user.rs           # 用户DTO
│   │   ├── posts.rs          # 文章DTO
│   │   └── common.rs         # 通用DTO
│   ├── handlers/              # HTTP请求处理器
│   │   ├── auth.rs           # 认证处理器
│   │   ├── users.rs          # 用户处理器
│   │   ├── posts.rs          # 文章处理器
│   │   └── upload.rs         # 上传处理器
│   ├── services/              # 业务逻辑服务
│   │   ├── auth.rs           # 认证服务
│   │   ├── posts.rs          # 文章服务
│   │   ├── email.rs          # 邮件服务
│   │   └── upload.rs         # 上传服务
│   ├── middleware/            # 中间件
│   │   ├── auth.rs           # JWT认证中间件
│   │   └── helpers.rs        # 辅助中间件
│   ├── routes/                # 路由配置
│   │   └── routes_module.rs  # 路由定义
│   └── utils/                 # 工具模块
│       ├── jwt.rs            # JWT工具
│       ├── crypto_pwd.rs     # 密码加密
│       └── perm_cache.rs     # 权限缓存
├── migration/                 # 数据库迁移文件
├── route-macros/              # 自定义路由宏
├── logs/                      # 日志文件目录
├── temp_uploads/              # 临时上传文件目录
├── Cargo.toml                 # 项目配置
├── .env.example              # 环境变量模板
├── Dockerfile                # Docker配置
├── architecture_diagram.md   # 架构图
├── arch.md                   # 架构文档
└── README.md                 # 项目说明

详细目录结构请参考 [arch.md](./arch.md)
```

## 📚 API 文档

服务器启动后，可以通过以下地址访问 API 文档：

- **Swagger UI**: http://localhost:8080/swagger-ui/
- **OpenAPI JSON**: http://localhost:8080/api-doc/openapi.json

### 主要API端点

#### 认证相关
```
POST /api/v1/auth/register    # 用户注册
POST /api/v1/auth/login       # 用户登录
POST /api/v1/auth/email       # 邮箱认证
POST /api/v1/auth/refresh     # 刷新令牌
```

#### 用户管理
```
GET    /api/v1/users          # 获取用户列表
GET    /api/v1/users/:uuid    # 获取用户详情
PUT    /api/v1/users/:uuid    # 更新用户信息
DELETE /api/v1/users/:uuid    # 删除用户
```

#### 文章管理
```
GET    /api/v1/posts          # 获取文章列表 (支持分页/搜索/过滤)
POST   /api/v1/posts          # 创建文章 (需要权限)
GET    /api/v1/posts/:uuid    # 获取文章详情
PUT    /api/v1/posts/:uuid    # 更新文章 (需要权限)
DELETE /api/v1/posts/:uuid    # 删除文章 (需要权限)
```

#### 分类与标签
```
GET    /api/v1/categories     # 获取分类列表
POST   /api/v1/categories     # 创建分类 (需要权限)
GET    /api/v1/tags           # 获取标签列表
POST   /api/v1/tags           # 创建标签 (需要权限)
```

#### 文件上传
```
POST /api/v1/upload           # 上传文件
POST /api/v1/upload/avatar    # 上传头像
POST /api/v1/upload/cover     # 上传封面
```

#### 实时通信
```
WS  /api/v1/ws                # WebSocket连接
GET /api/v1/sse/stream        # SSE事件流
```

## 🏗️ 架构设计

本项目采用分层架构设计，详细的架构图请参考：

- [architecture_diagram.md](./architecture_diagram.md) - 完整的Mermaid架构图
- [arch.md](./arch.md) - 详细的架构文档

### 核心架构

```
┌─────────────────────────────────────────────┐
│              客户端层                        │
│   (Web前端 / 移动端 / 管理后台)               │
└──────────────────┬──────────────────────────┘
                   │ HTTP/WebSocket
┌──────────────────▼──────────────────────────┐
│          中间件层                             │
│  CORS │ 认证 │ 权限 │ 日志 │ 错误处理          │
└──────────────────┬──────────────────────────┘
                   │
┌──────────────────▼──────────────────────────┐
│          路由层 (Routes)                     │
│      请求路由分发与参数解析                    │
└──────────────────┬──────────────────────────┘
                   │
┌──────────────────▼──────────────────────────┐
│        处理器层 (Handlers)                   │
│   请求参数验证 │ DTO转换 │ 响应构造            │
└──────────────────┬──────────────────────────┘
                   │
┌──────────────────▼──────────────────────────┐
│        服务层 (Services)                     │
│     核心业务逻辑 │ 事务处理                    │
└──────────────────┬──────────────────────────┘
                   │
┌──────────────────▼──────────────────────────┐
│      数据访问层 (Models - SeaORM)            │
│        ORM操作 │ 查询构建                     │
└──────────────────┬──────────────────────────┘
                   │
┌──────────────────▼──────────────────────────┐
│           数据库层                            │
│          PostgreSQL                          │
└─────────────────────────────────────────────┘
```

### 权限系统

采用 RBAC (基于角色的访问控制) 模型：

- **权限 (Permissions)**: 定义具体的操作权限
- **角色 (Roles)**: 权限的集合
- **用户-角色关联**: 用户可拥有多个角色
- **用户特殊权限**: 为用户单独授予/撤销权限
- **权限缓存**: 减少数据库查询，提高性能

## ⚙️ 配置说明

### 数据库配置

```env
DATABASE_URL=postgresql://username:password@localhost:5432/database_name
```

配置项 (在 `config/db.rs` 中):
- `max_connections`: 100 (最大连接数)
- `min_connections`: 5 (最小连接数)
- `connect_timeout`: 10s
- `idle_timeout`: 600s
- `max_lifetime`: 1800s

### JWT配置

```env
JWT_SECRET=your-secret-key-at-least-32-characters
```

- 令牌过期时间: 24小时
- 算法: HS256
- 包含信息: 用户ID, 过期时间

### CORS配置

开发环境 (在 `main.rs` 中):
```rust
.allowed_origin("*")
.allowed_methods(vec!["GET", "POST", "PUT", "DELETE"])
.supports_credentials()
```

生产环境建议配置具体的允许源。

### 日志配置

```env
RUST_LOG=info  # 日志级别: error, warn, info, debug, trace
```

日志输出:
- 控制台: INFO级别
- 文件: `logs/app.log`
- 轮转: 按天
- 保留: 30天

## 🛠️ 开发指南

### 添加新的API端点

1. **定义路由** (`routes/routes_module.rs`):
   ```rust
   web::resource("/api/v1/example")
       .route(web::get().to(handlers::example::get_example))
   ```

2. **创建处理器** (`handlers/example.rs`):
   ```rust
   #[route_permission("example:read")]
   pub async fn get_example(
       pool: web::Data<DatabaseConnection>,
   ) -> Result<HttpResponse> {
       // 实现逻辑
   }
   ```

3. **创建服务** (`services/example.rs`):
   ```rust
   pub async fn get_example_data(db: &DatabaseConnection) -> Result<Vec<Model>> {
       Example::find().all(db).await
   }
   ```

4. **定义DTO** (`dto/example.rs`):
   ```rust
   #[derive(Serialize, Deserialize)]
   pub struct ExampleResponse {
       pub id: i32,
       pub name: String,
   }
   ```

### 代码风格

项目使用 `rustfmt` 格式化代码：

```bash
cargo fmt
```

代码检查：

```bash
cargo clippy
```

### 提交规范

使用 [Conventional Commits](https://www.conventionalcommits.org/) 规范：

```
feat: 添加新功能
fix: 修复bug
docs: 文档更新
style: 代码格式调整
refactor: 代码重构
test: 测试相关
chore: 构建/工具链更新
```

## 🚢 部署

### Docker部署

1. **构建镜像**:
   ```bash
   docker build -t blog-server .
   ```

2. **运行容器**:
   ```bash
   docker run -d \
     -p 8080:8080 \
     -e DATABASE_URL=postgresql://... \
     -e JWT_SECRET=... \
     --name blog-server \
     blog-server
   ```

### 生产环境配置

1. **构建发布版本**:
   ```bash
   cargo build --release
   ```

2. **配置环境变量**:
   - 设置强密钥的 `JWT_SECRET`
   - 配置正确的 `DATABASE_URL`
   - 限制 CORS 允许的源
   - 配置 HTTPS

3. **使用进程管理器** (如 systemd):
   ```ini
   [Unit]
   Description=Blog Server
   After=network.target

   [Service]
   Type=simple
   User=blog
   WorkingDirectory=/opt/blog-server
   Environment="DATABASE_URL=..."
   Environment="JWT_SECRET=..."
   ExecStart=/opt/blog-server/target/release/blog-server
   Restart=always

   [Install]
   WantedBy=multi-user.target
   ```

## 🧪 测试

运行所有测试：

```bash
cargo test
```

运行特定模块测试：

```bash
cargo test services::auth
```

生成测试覆盖率报告：

```bash
cargo install cargo-tarpaulin
cargo tarpaulin --out Html
```

## 📝 贡献指南

欢迎提交 Issue 和 Pull Request！

### 贡献步骤

1. Fork 本仓库
2. 创建特性分支 (`git checkout -b feature/AmazingFeature`)
3. 提交更改 (`git commit -m 'feat: Add some AmazingFeature'`)
4. 推送到分支 (`git push origin feature/AmazingFeature`)
5. 开启 Pull Request

### 开发规范

- 遵循项目的代码风格 (使用 `cargo fmt`)
- 运行 `cargo clippy` 确保代码质量
- 为新功能添加测试
- 更新相关文档

## 📄 许可证

本项目采用 MIT 许可证 - 详见 [LICENSE](LICENSE) 文件

## 🙏 致谢

- [Actix-Web](https://actix.rs/) - 强大的 Rust Web 框架
- [SeaORM](https://www.sea-ql.org/SeaORM/) - 优秀的 ORM 库
- [Tokio](https://tokio.rs/) - 异步运行时

## 📞 联系方式

如有问题或建议，欢迎通过以下方式联系：

- 提交 [Issue](../../issues)
- 发送邮件到: [your-email@example.com]

---

<div align="center">
Made with ❤️ using Rust
</div>
