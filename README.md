# 博客系统后端

个人博客系统 REST API 后端，基于 **Actix-web** + **SeaORM** + **PostgreSQL** 构建。

**作者**：Sunrisies ([3266420686@qq.com](mailto:3266420686@qq.com))

---

## 功能特性

- 📝 **文章管理** — 创建 / 编辑 / 删除 / 分页列表 / 时间轴 / 上下篇导航
- 🏷️ **分类 & 标签** — 文章分类与标签关联、标签使用量统计
- 🔐 **用户认证** — 密码登录 / 邮箱验证码登录 / JWT 令牌
- 📎 **外部链接** — 友链管理
- 🖼️ **图片上传** — 七牛云存储
- 📧 **邮件服务** — 验证码发送（SMTP）
- 💬 **聊天室** — WebSocket 房间
- 📋 **云剪贴板** — 频道式文本/文件/图片跨设备同步
- 📄 **OpenAPI 文档** — 启动时自动生成 `openapi.json`

## 技术栈

| 领域 | 技术 |
|------|------|
| Web 框架 | Actix-web 4 |
| ORM | SeaORM 1.x |
| 数据库 | PostgreSQL |
| 认证 | JWT (jsonwebtoken) + Argon2id |
| 存储 | 七牛云 Kodo |
| API 文档 | Utoipa (OpenAPI 3.1) |
| 宏生成 | route-macros (自研，已发布到 crates.io) |

## 快速开始

### 环境要求

- Rust 1.85+
- PostgreSQL 16+
- 七牛云存储账号（可选，文件上传功能需要）

### 配置

复制 `.env.example` 为 `.env`，修改配置：

```env
DATABASE_URL=postgres://postgres:password@localhost:5432/database
JWT_SECRET=your_jwt_secret_key
SERVER_HOST=0.0.0.0
SERVER_PORT=2345

# 七牛云（可选）
QINIU_ACCESS_KEY=your_access_key
QINIU_SECRET_KEY=your_secret_key
QINIU_BUCKET=blog-sunrise
QINIU_DOMAIN=https://img.yourdomain.com

# SMTP（可选）
SMTP_HOST=smtp.qq.com
SMTP_PORT=465
SMTP_USERNAME=your_email@qq.com
SMTP_PASSWORD=your_smtp_password
```

### 运行

```bash
# 初始化数据库（建表）
psql -h localhost -U postgres -d database -f migration/src/public.sql

# 启动服务
cargo run -p web-server

# 服务运行在 http://0.0.0.0:2345
# API 文档生成在 openapi.json
```

## 项目结构

```
server/
├── src/
│   ├── config/          # 配置、日志、错误定义
│   ├── dto/             # 请求/响应 DTO
│   ├── handlers/        # 请求处理器
│   ├── middleware/       # 认证中间件
│   ├── models/          # SeaORM 实体
│   ├── routes/          # 路由注册
│   ├── services/        # 业务逻辑层
│   └── utils/           # 工具函数（JWT、密码、文件等）
├── migration/           # 数据库迁移 SQL
├── route-macros/        # proc-macro 库（CRUD 代码生成）
├── route-macros-types/  # 共享类型库
└── Cargo.toml           # workspace 配置
```

## Workspace

本项目由 workspace 管理三个 crate：

| crate | 类型 | 说明 | crates.io |
|-------|------|------|-----------|
| `web-server` | 二进制 | 博客后端服务 | — |
| `route-macros` | proc-macro | CRUD handler 代码生成器 | [![crates.io](https://img.shields.io/crates/v/route-macros)](https://crates.io/crates/route-macros) |
| `route-macros-types` | 库 | 共享类型（ApiResponse 等） | [![crates.io](https://img.shields.io/crates/v/route-macros-types)](https://crates.io/crates/route-macros-types) |

## API 概览

| 模块 | 路径前缀 | 主要接口 |
|------|----------|----------|
| 认证 | `/api/v1/auth` | 注册、登录（密码/邮箱/手机/OAuth） |
| 文章 | `/api/v1/posts` | CRUD、时间轴、上下篇、标签/分类筛选 |
| 分类 | `/api/v1/categories` | CRUD |
| 标签 | `/api/v1/tags` | CRUD、使用量统计、标签下文章列表 |
| 链接 | `/api/v1/links` | CRUD |
| 图片 | `/api/v1/images` | 上传/列表/详情/删除 |
| 上传 | `/api/v1/upload` | 文件上传到七牛云 |
| 邮件 | `/api/v1/email` | 发送验证码 |
| 聊天室 | `/api/v1/rooms` | 创建房间、获取消息 |
| 云剪贴板 | `/api/v1/clipboard` | 频道创建/登录、文本/文件上传、列表、删除 |
| 系统 | `/api/v1/version` | 版本信息 |

### 云剪贴板认证流程

1. 管理员创建频道 → `POST /clipboard/channel`（需管理员登录）
2. 用户登录频道 → `POST /clipboard/channel/auth` → 获取频道 token
3. 后续请求带 `Authorization: Bearer <token>`

## 许可证

MIT
