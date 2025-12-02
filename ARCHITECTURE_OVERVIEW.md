# 博客系统架构总览

本文档提供了博客系统的架构概览，包含所有核心组件和它们之间的关系。

## 📚 文档导航

1. **[README.md](./README.md)** - 项目介绍和快速开始指南
2. **[architecture_diagram.md](./architecture_diagram.md)** - 详细的架构图和流程图
3. **[arch.md](./arch.md)** - 架构设计文档和最佳实践
4. **本文档** - 架构总览和核心概念

## 🎯 系统概览

博客系统是一个基于 Rust 和 Actix-Web 框架构建的现代化Web应用，采用分层架构设计，提供：

- 🔐 完整的认证授权系统 (JWT + RBAC)
- 📝 博客内容管理 (文章、分类、标签)
- 👥 用户管理和权限控制
- 💬 实时通信 (WebSocket + SSE)
- 📁 文件上传和云存储
- 📧 邮件服务

## 🏗️ 核心架构层次

### 1. 客户端层
- Web前端应用
- 移动端应用
- 管理后台

### 2. API网关层 (Actix-Web)
```
┌─────────────────────────────────┐
│     Actix-Web 服务器             │
├─────────────────────────────────┤
│  ▸ CORS 中间件                   │
│  ▸ 认证中间件 (JWT验证)           │
│  ▸ 权限中间件 (RBAC检查)          │
│  ▸ 日志中间件                     │
│  ▸ 错误处理中间件                 │
└─────────────────────────────────┘
```

### 3. 应用层

#### 路由层 (Routes)
- 请求路由分发
- URL模式匹配
- 参数提取

#### 处理器层 (Handlers)
```
handlers/
├── auth.rs          # 认证处理器 (登录/注册)
├── users.rs         # 用户管理
├── posts.rs         # 文章管理
├── category.rs      # 分类管理
├── tags.rs          # 标签管理
├── rooms.rs         # 聊天房间
├── room_messages.rs # 聊天消息
├── upload.rs        # 文件上传
└── email.rs         # 邮件处理
```

#### 服务层 (Services)
```
services/
├── auth.rs          # JWT生成/验证、密码加密
├── users.rs         # 用户业务逻辑
├── posts.rs         # 文章CRUD、搜索、过滤
├── category.rs      # 分类业务逻辑
├── email.rs         # 邮件发送服务
├── upload.rs        # 文件上传、云存储
├── ws.rs            # WebSocket聊天
└── sse.rs           # SSE推送
```

### 4. 数据层

#### 数据传输对象 (DTO)
```
dto/
├── common.rs        # 通用DTO (分页、响应)
├── user.rs          # 用户请求/响应DTO
├── posts.rs         # 文章请求/响应DTO
├── category.rs      # 分类DTO
└── tag.rs           # 标签DTO
```

#### 数据模型 (Models - SeaORM)
```
models/
├── users.rs              # 用户表
├── posts.rs              # 文章表
├── categories.rs         # 分类表
├── tags.rs               # 标签表
├── post_tags.rs          # 文章-标签关联
├── roles.rs              # 角色表
├── permissions.rs        # 权限表
├── user_roles.rs         # 用户-角色关联
├── role_permissions.rs   # 角色-权限关联
├── user_permissions.rs   # 用户-权限关联
├── rooms.rs              # 聊天房间
└── room_messages.rs      # 聊天消息
```

### 5. 基础设施层

#### 工具模块 (Utils)
```
utils/
├── jwt.rs            # JWT令牌生成/验证
├── crypto_pwd.rs     # Argon2密码加密
├── perm_cache.rs     # 权限缓存
├── websocket.rs      # WebSocket工具
├── sse.rs            # SSE工具
├── fmt_time.rs       # 时间格式化
├── file_size.rs      # 文件大小处理
├── db_error.rs       # 数据库错误处理
└── data_processing.rs # 数据处理
```

#### 配置模块 (Config)
```
config/
├── db.rs             # 数据库连接池配置
├── log.rs            # 日志配置
├── email.rs          # 邮件配置
├── error.rs          # 统一错误处理
└── api_doc.rs        # OpenAPI文档配置
```

### 6. 数据存储层
- **PostgreSQL** - 主数据库
- **七牛云存储** - 文件存储

## 🔄 典型请求流程

```
1. 客户端发送 HTTP 请求
   ↓
2. CORS 中间件检查跨域
   ↓
3. 日志中间件记录请求
   ↓
4. 认证中间件验证 JWT 令牌
   ↓
5. 权限中间件检查用户权限
   ↓
6. 路由层分发请求到对应 Handler
   ↓
7. Handler 验证请求参数
   ↓
8. Handler 调用 Service 执行业务逻辑
   ↓
9. Service 通过 Model 访问数据库
   ↓
10. Service 返回结果给 Handler
    ↓
11. Handler 构造 DTO 响应
    ↓
12. 错误处理中间件统一处理异常
    ↓
13. 日志中间件记录响应
    ↓
14. 返回 HTTP 响应给客户端
```

## 🔐 认证授权系统

### JWT 认证流程

```
登录请求
   ↓
验证用户名密码 (Argon2)
   ↓
生成 JWT 令牌 (包含: user_id, exp)
   ↓
返回令牌给客户端
   ↓
后续请求携带令牌在 Authorization 头
   ↓
认证中间件验证令牌
   ↓
提取用户信息
   ↓
继续处理请求
```

### RBAC 权限系统

```
权限系统组成:
┌────────────────────────────────────┐
│  Permissions (权限)                 │
│  ├─ posts:create                   │
│  ├─ posts:edit                     │
│  ├─ posts:delete                   │
│  └─ users:manage                   │
└────────────────────────────────────┘
            ↓ 组合成
┌────────────────────────────────────┐
│  Roles (角色)                       │
│  ├─ admin (所有权限)                │
│  ├─ editor (编辑权限)               │
│  └─ author (作者权限)               │
└────────────────────────────────────┘
            ↓ 分配给
┌────────────────────────────────────┐
│  Users (用户)                       │
│  ├─ 角色权限 (通过 user_roles)       │
│  └─ 特殊权限 (通过 user_permissions) │
└────────────────────────────────────┘

权限验证:
用户最终权限 = 角色权限 ∪ 用户特殊权限
```

### 权限检查流程

```
1. 请求到达需要权限的路由
   ↓
2. 检查路由标记的权限 (通过 route_permission! 宏)
   ↓
3. 查询权限缓存
   ↓
4. 如缓存未命中:
   - 查询用户角色
   - 查询角色权限
   - 查询用户特殊权限
   - 合并计算最终权限
   - 更新缓存
   ↓
5. 验证用户是否有所需权限
   ↓
6. 通过 → 继续处理 | 拒绝 → 返回 403
```

## 📊 数据库设计要点

### 核心表关系

```
users (用户)
  ├─→ posts (1:N) - 一个用户可以有多篇文章
  ├─→ user_roles (1:N) - 一个用户可以有多个角色
  └─→ user_permissions (1:N) - 一个用户可以有多个特殊权限

posts (文章)
  ├─→ categories (N:1) - 多篇文章属于一个分类
  └─→ post_tags (N:M) - 文章和标签多对多关系

roles (角色)
  ├─→ user_roles (1:N) - 一个角色可被多个用户拥有
  └─→ role_permissions (1:N) - 一个角色可以有多个权限

permissions (权限)
  ├─→ role_permissions (1:N) - 一个权限可以属于多个角色
  └─→ user_permissions (1:N) - 一个权限可以被授予多个用户

rooms (聊天房间)
  └─→ room_messages (1:N) - 一个房间有多条消息
```

### 关键索引

- users: uuid, user_name, email, phone (唯一索引)
- posts: uuid (唯一), author_id, category_id, status (普通索引)
- post_tags: (post_id, tag_id) 联合主键
- user_roles: (user_id, role_id) 联合索引
- role_permissions: (role_id, permission_id) 联合索引

## 🌐 API 设计原则

### RESTful 规范

```
资源              GET (读取)      POST (创建)     PUT (更新)      DELETE (删除)
/posts           获取列表         创建文章        -               -
/posts/:uuid     获取详情         -               更新文章        删除文章
/categories      获取列表         创建分类        -               -
/categories/:id  获取详情         -               更新分类        删除分类
```

### 响应格式

成功响应:
```json
{
  "data": { ... },
  "pagination": {
    "page": 1,
    "page_size": 20,
    "total_items": 150,
    "total_pages": 8
  }
}
```

错误响应:
```json
{
  "error": {
    "code": "ERROR_CODE",
    "message": "错误描述",
    "details": { ... }
  }
}
```

## 🚀 性能优化策略

### 1. 数据库层
- **连接池**: 100个最大连接，5个最小连接
- **索引优化**: 为常用查询字段添加索引
- **查询优化**: 避免N+1问题，使用预加载
- **分页查询**: 所有列表接口支持分页

### 2. 应用层
- **权限缓存**: 减少权限查询的数据库访问
- **异步处理**: 使用Tokio异步运行时
- **连接复用**: 数据库连接池管理

### 3. 传输层
- **响应压缩**: Gzip压缩
- **字段选择**: 只返回必要字段
- **批量操作**: 支持批量请求

## 🔒 安全措施

### 认证安全
- ✅ JWT 签名验证
- ✅ 令牌过期控制
- ✅ Argon2 密码加密
- ✅ 防止密码日志泄露

### 授权安全
- ✅ RBAC 权限控制
- ✅ 细粒度权限定义
- ✅ 路由级权限保护
- ✅ 权限缓存机制

### 输入验证
- ✅ 类型验证
- ✅ 长度限制
- ✅ SQL 注入防护 (ORM)
- ✅ XSS 防护

### 文件上传
- ✅ MIME 类型验证
- ✅ 文件大小限制
- ✅ 文件名清理
- ✅ 临时文件清理

## 📈 监控与日志

### 日志级别
- **ERROR**: 系统错误
- **WARN**: 警告信息
- **INFO**: 一般操作
- **DEBUG**: 调试信息
- **TRACE**: 详细追踪

### 日志输出
- 控制台: INFO 级别
- 文件: logs/app.log
- 轮转: 按天
- 保留: 30天

### 关键日志点
- 用户登录/注册
- 权限验证失败
- 数据库操作错误
- API 请求/响应
- 文件上传操作

## 🔧 扩展点

### 易于扩展的部分

1. **新增 API 端点**
   - 添加路由定义
   - 创建 Handler
   - 实现 Service
   - 定义 DTO

2. **新增权限**
   - 在数据库添加权限记录
   - 使用 `route_permission!` 标记路由

3. **新增中间件**
   - 实现中间件逻辑
   - 在路由配置中注册

4. **新增数据模型**
   - 定义 SeaORM 实体
   - 创建迁移文件
   - 运行迁移

## 📖 相关文档

- **[API 设计模式](./arch.md#api-设计模式)** - RESTful API 设计规范
- **[数据库设计模式](./arch.md#数据库设计模式)** - 数据库操作最佳实践
- **[Rust 编码标准](./arch.md#rust-编码标准)** - 代码风格和规范
- **[安全指南](./arch.md#安全指南)** - 安全最佳实践
- **[测试指南](./arch.md#测试指南)** - 测试策略和方法

## 🎓 学习资源

### Rust 学习
- [The Rust Book](https://doc.rust-lang.org/book/)
- [Rust by Example](https://doc.rust-lang.org/rust-by-example/)

### Actix-Web
- [Actix-Web 官方文档](https://actix.rs/docs/)
- [Actix-Web API 文档](https://docs.rs/actix-web/)

### SeaORM
- [SeaORM 文档](https://www.sea-ql.org/SeaORM/docs/index/)
- [SeaORM 教程](https://www.sea-ql.org/SeaORM/docs/basic-crud/select/)

---

## 📞 获取帮助

如果您对架构有任何疑问，请：

1. 查看相关文档
2. 搜索已有的 Issues
3. 创建新的 Issue
4. 联系团队成员

---

<div align="center">

**[返回顶部](#博客系统架构总览)**

Made with ❤️ using Rust

</div>
