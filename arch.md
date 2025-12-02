# 项目架构文档

## 1. 项目概述
本项目是一个基于 Rust 和 Actix-Web 框架构建的现代化博客系统后端服务，提供了完整的博客管理、用户认证、权限控制、实时通信等功能。

### 核心技术栈
- **Web框架**: Actix-Web 4.x
- **ORM**: SeaORM
- **数据库**: PostgreSQL
- **认证**: JWT (jsonwebtoken)
- **密码加密**: Argon2
- **文件存储**: 七牛云存储
- **实时通信**: WebSocket + SSE
- **日志**: log4rs
- **API文档**: OpenAPI (utoipa)

## 2. 项目结构

### 2.1 目录结构
```
src/
├── main.rs              # 应用程序入口点
├── lib.rs               # 核心模块导出和全局配置
│
├── config/              # 配置模块
│   ├── mod.rs          # 配置模块导出
│   ├── db.rs           # 数据库连接池配置
│   ├── email.rs        # 邮件服务配置
│   ├── error.rs        # 统一错误处理定义
│   ├── log.rs          # 日志配置
│   └── api_doc.rs      # OpenAPI文档配置
│
├── models/              # 数据模型定义 (SeaORM实体)
│   ├── mod.rs          # 模型模块导出
│   ├── prelude.rs      # 常用模型预导入
│   ├── users.rs        # 用户模型
│   ├── posts.rs        # 文章模型
│   ├── categories.rs   # 分类模型
│   ├── tags.rs         # 标签模型
│   ├── post_tags.rs    # 文章-标签关联
│   ├── roles.rs        # 角色模型
│   ├── permissions.rs  # 权限模型
│   ├── user_roles.rs   # 用户-角色关联
│   ├── role_permissions.rs  # 角色-权限关联
│   ├── user_permissions.rs  # 用户-权限关联
│   ├── rooms.rs        # 聊天房间模型
│   └── room_messages.rs     # 房间消息模型
│
├── dto/                 # 数据传输对象
│   ├── mod.rs          # DTO模块导出
│   ├── common.rs       # 通用DTO (分页、响应等)
│   ├── user.rs         # 用户相关DTO
│   ├── posts.rs        # 文章相关DTO
│   ├── category.rs     # 分类相关DTO
│   └── tag.rs          # 标签相关DTO
│
├── handlers/            # HTTP请求处理器
│   ├── mod.rs          # 处理器模块导出
│   ├── auth.rs         # 认证处理器 (登录/注册)
│   ├── users.rs        # 用户管理处理器
│   ├── posts.rs        # 文章管理处理器
│   ├── category.rs     # 分类管理处理器
│   ├── tags.rs         # 标签管理处理器
│   ├── rooms.rs        # 房间管理处理器
│   ├── room_messages.rs # 消息处理器
│   ├── upload.rs       # 文件上传处理器
│   └── email.rs        # 邮件处理器
│
├── services/            # 业务逻辑服务
│   ├── mod.rs          # 服务模块导出
│   ├── auth.rs         # 认证服务 (JWT生成/验证)
│   ├── users.rs        # 用户服务
│   ├── posts.rs        # 文章服务 (CRUD/搜索)
│   ├── category.rs     # 分类服务
│   ├── email.rs        # 邮件服务
│   ├── upload.rs       # 文件上传服务
│   ├── ws.rs           # WebSocket聊天服务
│   └── sse.rs          # SSE推送服务
│
├── middleware/          # 中间件
│   ├── mod.rs          # 中间件模块导出
│   ├── auth.rs         # JWT认证中间件
│   └── helpers.rs      # 辅助中间件函数
│
├── routes/              # 路由配置
│   ├── mod.rs          # 路由模块导出
│   └── routes_module.rs # 路由定义与注册
│
├── utils/               # 工具模块
│   ├── mod.rs          # 工具模块导出
│   ├── jwt.rs          # JWT工具
│   ├── crypto_pwd.rs   # 密码加密工具
│   ├── perm_cache.rs   # 权限缓存工具
│   ├── websocket.rs    # WebSocket工具
│   ├── sse.rs          # SSE工具
│   ├── fmt_time.rs     # 时间格式化
│   ├── file_size.rs    # 文件大小处理
│   ├── db_error.rs     # 数据库错误处理
│   └── data_processing.rs # 数据处理工具
│
├── sql/                 # SQL脚本
│   ├── user.sql        # 用户表SQL
│   ├── posts.sql       # 文章表SQL
│   └── room.sql        # 房间表SQL
│
└── schema/              # Schema定义
    └── user.rs

migration/               # 数据库迁移
route-macros/            # 自定义路由宏
```

## 3. 架构设计

### 3.1 分层架构

项目采用经典的分层架构设计：

```
客户端请求
    ↓
[中间件层] - CORS、认证、日志、错误处理
    ↓
[路由层] - 请求路由分发
    ↓
[处理器层 Handlers] - 请求参数验证、DTO转换
    ↓
[服务层 Services] - 业务逻辑实现
    ↓
[数据访问层 Models] - ORM操作
    ↓
[数据库层] - PostgreSQL
```

### 3.2 模块命名规则

1. **文件命名**: 使用小写字母和下划线分隔，如 `user_roles.rs`
2. **模块结构**: 每个功能目录包含 `mod.rs` 文件用于模块导出
3. **导出规则**: 公共接口通过 `lib.rs` 统一导出

### 3.3 依赖关系

- **Handlers 层**: 依赖 Services 层和 DTO 层，负责HTTP请求处理
- **Services 层**: 依赖 Models 层，实现核心业务逻辑
- **Models 层**: SeaORM实体定义，直接与数据库交互
- **DTO 层**: 独立层，用于请求/响应数据传输
- **Middleware 层**: 依赖 Utils 层，提供横切关注点
- **Utils 层**: 基础工具，被各层使用

## 4. 核心功能模块

### 4.1 认证授权系统

#### 认证流程
1. 用户提交登录凭证 (用户名/邮箱/手机 + 密码)
2. 验证密码 (Argon2哈希比对)
3. 生成JWT令牌 (包含用户ID、过期时间)
4. 返回令牌给客户端
5. 后续请求携带令牌在Authorization头部

#### 权限系统 (RBAC)
- **权限定义**: `permissions` 表定义所有权限
- **角色管理**: `roles` 表定义角色，通过 `role_permissions` 关联权限
- **用户角色**: 通过 `user_roles` 关联用户和角色
- **特殊权限**: 通过 `user_permissions` 为用户单独授予/撤销权限
- **权限验证**: 使用 `route_permission!` 宏标记需要的权限
- **权限缓存**: 使用 `perm_cache` 减少数据库查询

#### 中间件保护
- `middleware/auth.rs`: JWT验证和权限检查
- 自动从令牌中提取用户信息
- 验证用户是否有访问路由所需的权限

### 4.2 用户管理

#### 用户模型
```rust
users {
    id: i32 (PK)
    uuid: String (UK)
    user_name: String (UK)
    pass_word: String (哈希)
    email: String (UK)
    phone: String (UK)
    image: String
    binding: String
    created_at: DateTime
    updated_at: DateTime
}
```

#### 功能特性
- 用户注册/登录
- 多登录方式 (用户名/邮箱/手机)
- 密码加密存储
- 用户信息CRUD
- 角色分配
- 权限管理

### 4.3 文章管理

#### 文章模型
```rust
posts {
    id: i32 (PK)
    uuid: String (UK)
    author_id: i32 (FK -> users)
    category_id: i32 (FK -> categories)
    title: String
    summary: Text
    content: Text
    markdowncontent: Text
    cover_image: String
    status: i16
    featured: bool
    view_count: i32
    published_at: DateTime
    created_at: DateTime
    updated_at: DateTime
    size: i32
}
```

#### 功能特性
- 文章CRUD操作
- 富文本/Markdown支持
- 分类管理
- 标签系统 (多对多关系)
- 文章搜索/过滤
- 分页查询
- 草稿/发布状态
- 精选文章
- 浏览计数

### 4.4 实时通信

#### WebSocket聊天
- 房间系统
- 实时消息推送
- 在线用户管理
- 消息持久化
- 文件消息支持
- 消息过期机制

#### SSE推送
- 服务端主动推送
- 用于通知、实时更新
- 长连接管理

### 4.5 文件上传

#### 上传流程
1. 客户端选择文件
2. 验证文件类型和大小
3. 上传到服务器临时目录
4. 处理文件 (图片压缩/视频转码)
5. 上传到七牛云存储
6. 删除临时文件
7. 返回云存储URL

#### 支持功能
- 图片上传 (支持压缩)
- 视频上传
- 文件类型验证
- 大小限制
- 云存储集成

## 5. 数据流

### 5.1 典型请求流程

```
1. 客户端发送请求
   ↓
2. CORS中间件检查跨域
   ↓
3. 日志中间件记录请求
   ↓
4. 认证中间件验证JWT
   ↓
5. 权限中间件检查权限
   ↓
6. 路由到对应Handler
   ↓
7. Handler验证请求参数
   ↓
8. 调用Service执行业务逻辑
   ↓
9. Service通过Model访问数据库
   ↓
10. 返回结果并构造DTO
   ↓
11. Handler构造HTTP响应
   ↓
12. 错误处理中间件捕获异常
   ↓
13. 日志中间件记录响应
   ↓
14. 返回给客户端
```

### 5.2 权限验证流程

```
1. 请求到达认证中间件
   ↓
2. 提取Authorization头部的JWT
   ↓
3. 验证JWT签名和过期时间
   ↓
4. 提取用户ID
   ↓
5. 检查路由所需权限
   ↓
6. 查询权限缓存
   ↓
7. 如缓存未命中，查询数据库
   - 查询用户角色
   - 查询角色权限
   - 查询用户特殊权限
   - 合并计算最终权限
   ↓
8. 更新权限缓存
   ↓
9. 验证是否有所需权限
   ↓
10. 通过/拒绝
```

## 6. 关键配置

### 6.1 数据库连接池
```rust
// config/db.rs
max_connections: 100      // 最大连接数
min_connections: 5        // 最小连接数
connect_timeout: 10s      // 连接超时
idle_timeout: 600s        // 空闲超时
max_lifetime: 1800s       // 连接最大生命周期
```

### 6.2 CORS配置
```rust
// main.rs
allowed_origin: "*"       // 开发环境允许所有源
allowed_methods: [GET, POST, PUT, DELETE, PATCH]
allowed_headers: [Authorization, Content-Type]
supports_credentials: true
max_age: 3600
```

### 6.3 JWT配置
```rust
// 通过环境变量配置
JWT_SECRET: "your-secret-key"
TOKEN_EXPIRATION: 24h     // 令牌过期时间
```

## 7. 安全措施

### 7.1 认证安全
- JWT令牌签名验证
- 令牌过期时间控制
- 刷新令牌机制
- 密码Argon2加密存储
- 防止密码明文记录日志

### 7.2 授权安全
- 基于RBAC的权限控制
- 细粒度权限定义
- 权限缓存防止频繁查询
- 路由级权限保护

### 7.3 输入验证
- 请求参数类型验证
- 数据长度限制
- SQL注入防护 (ORM参数化)
- XSS防护 (内容转义)

### 7.4 文件上传安全
- MIME类型验证
- 文件大小限制
- 文件名清理
- 临时文件清理

## 8. 性能优化

### 8.1 数据库优化
- 连接池管理
- 索引优化
- 查询优化 (避免N+1)
- 分页查询

### 8.2 缓存策略
- 权限缓存
- 数据库连接池
- 静态资源缓存

### 8.3 并发处理
- Actix-Web异步处理
- Tokio运行时
- 数据库连接池并发

## 9. 错误处理

### 9.1 统一错误类型
```rust
// config/error.rs
pub enum AppError {
    DatabaseError(String),
    ValidationError(String),
    NotFound(String),
    Unauthorized(String),
    Forbidden(String),
    InternalError(String),
}
```

### 9.2 错误响应格式
```json
{
    "error": {
        "code": "VALIDATION_ERROR",
        "message": "请求参数验证失败",
        "details": {
            "field": ["error message"]
        }
    }
}
```

## 10. 日志系统

### 10.1 日志级别
- **ERROR**: 错误信息
- **WARN**: 警告信息
- **INFO**: 一般信息
- **DEBUG**: 调试信息
- **TRACE**: 追踪信息

### 10.2 日志配置
```rust
// config/log.rs
- 控制台输出: INFO级别
- 文件输出: logs/app.log
- 日志轮转: 按天
- 日志保留: 30天
```

## 11. API文档

### 11.1 OpenAPI支持
- 使用 `utoipa` crate
- 自动生成OpenAPI规范
- Swagger UI集成
- 路径: `/api-doc/openapi.json`

### 11.2 API版本控制
- URL版本控制: `/api/v1/...`
- 向后兼容原则

## 12. 测试策略

### 12.1 单元测试
- 对核心业务逻辑编写单元测试
- Service层测试
- Utils工具测试

### 12.2 集成测试
- API端点测试
- 数据库操作测试
- 中间件测试

## 13. 部署

### 13.1 构建
```bash
cargo build --release
```

### 13.2 Docker支持
```dockerfile
# Dockerfile已提供
FROM rust:latest
...
```

### 13.3 环境变量
```env
DATABASE_URL=postgresql://user:pass@host/db
JWT_SECRET=your-secret-key
QINIU_ACCESS_KEY=your-access-key
QINIU_SECRET_KEY=your-secret-key
SMTP_HOST=smtp.example.com
```

## 14. 扩展性

### 14.1 模块化设计
- 各模块独立
- 便于添加新功能
- 清晰的依赖关系

### 14.2 中间件扩展
- 易于添加新中间件
- 统一的中间件接口

### 14.3 路由扩展
- 使用宏简化路由定义
- 支持权限标记

## 15. 最佳实践

1. **代码组织**: 遵循分层架构，保持各层职责单一
2. **错误处理**: 使用统一的错误类型，提供清晰的错误信息
3. **异步编程**: 合理使用async/await，避免阻塞
4. **数据库操作**: 使用ORM，避免SQL注入
5. **安全意识**: 验证所有输入，保护敏感数据
6. **日志记录**: 记录关键操作，便于问题排查
7. **性能监控**: 关注数据库查询性能，优化慢查询
8. **文档维护**: 保持API文档和代码同步
