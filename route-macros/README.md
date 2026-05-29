# route-macros

Procedural macros for auto-generating Actix-web CRUD handlers, routes, and OpenAPI documentation from SeaORM entities.

## Features

- **`crud_entity!`** — 生成 CRUD handler（create / read / update / delete / list）+ OpenAPI 文档
- **`route_permission`** — 路由权限标记宏
- 与 `route-macros-types` 配合使用，支持自定义错误类型

## Quick Start

```toml
[dependencies]
route-macros = "0.1"
route-macros-types = "0.1"
```

### 1. 定义 SeaORM 实体和请求 DTO

```rust
// models/my_entity.rs
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "my_entities")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub name: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
```

```rust
// dto/my_entity.rs
use serde::Deserialize;
use validator::Validate;

#[derive(Deserialize, Validate)]
pub struct CreateMyEntityRequest {
    #[validate(length(min = 1, max = 100))]
    pub name: String,
}
```

### 2. 使用宏生成 handler

```rust
// handlers/my_entity.rs
use route_macros::crud_entity;
use route_macros_types::{ApiResponse, EmptyResponse, HttpResult};

crud_entity!({
    entity: my_entity,
    route_prefix: "/api/v1/my-entities",
    permission_prefix: "my_entity",
    id_type: "id",
    operations: ["create", "read", "list", "delete"],
    create_request_type: CreateMyEntityRequest,
    unique_field: Name,
    openapi_summary: "My Entity",
});
```

### 3. 注册路由

```rust
// routes/my_entity.rs
use crate::handlers::my_entity::my_entity_routes::{
    create_my_entity_handler, delete_my_entity_handler,
    get_my_entity_all_handler, get_my_entity_handler,
};
use actix_web::web;

pub fn config_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/v1/my-entities")
            .route("", web::post().to(create_my_entity_handler))
            .route("", web::get().to(get_my_entity_all_handler))
            .route("/{id}", web::get().to(get_my_entity_handler))
            .route("/{id}", web::delete().to(delete_my_entity_handler)),
    );
}
```

## 配置参考

### `crud_entity!` 参数

| 参数 | 必填 | 类型 | 说明 |
|------|------|------|------|
| `entity` | ✅ | Ident | SeaORM 实体名（对应 `models::xxx`） |
| `route_prefix` | ✅ | 字符串 | API 路径前缀，如 `"/api/v1/items"` |
| `permission_prefix` | ✅ | 字符串 | 权限前缀 |
| `id_type` | ❌ | `"uuid"` 或 `"id"` | ID 类型，默认 `uuid` |
| `operations` | ❌ | 字符串数组 | 可选：`"create"`, `"read"`, `"list"`, `"delete"`，默认全开 |
| `create_request_type` | create 时需要 | Ident | 创建请求的 DTO 类型 |
| `update_request_type` | update 时需要 | Ident | 更新请求的 DTO 类型（预留） |
| `unique_field` | ❌ | Ident | 创建时做唯一性检查的字段名，如 `Name` |
| `error_type` | ❌ | 类型路径 | 错误类型，默认 `crate::AppError` |
| `openapi_summary` | ❌ | 字符串 | OpenAPI 摘要 |
| `openapi_create` / `openapi_read` / `openapi_list` / `openapi_delete` | ❌ | 对象 | 各操作的 OpenAPI 配置（summary / description / tag / deprecated / hidden） |

### `error_type` 说明

宏生成的代码会引用以下错误变体（需在自定义类型中定义）：

| 变体 | 用途 |
|------|------|
| `DatabaseError(String)` | 数据库查询错误 |
| `DatabaseConnectionError(String)` | 数据库连接 / 唯一性冲突错误 |
| `NotFound(String)` | 资源不存在 |
| `ValidationError(ValidationErrorJson)` | 参数校验失败 |
| `BadRequest(String)` | 请求参数错误 |

如果不指定 `error_type`，默认使用 `crate::AppError`，也可用 `route_macros_types::AppError`。

## 本地开发

```bash
# 克隆项目
git clone <repo>
cd server

# 使用本地路径依赖
cargo add route-macros --path ../route-macros
cargo add route-macros-types --path ../route-macros-types

# 编译
cargo check -p route-macros
```
