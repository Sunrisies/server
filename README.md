# Rust Web Server + route-macros

博客系统后端，包含一个 Actix-web REST API 项目和一个 proc-macro 库。

## Workspace 结构

```
├── server/                    # Actix-web REST API 服务
├── route-macros/              # proc-macro 库（生成 CRUD handler）
└── route-macros-types/        # 共享类型库（响应结构、错误类型等）
```

## 本地开发

```bash
# 编译全部
cargo check

# 只编译宏库
cargo check -p route-macros

# 只编译主项目
cargo check -p web-server
```

## 发布

```bash
# 先发布 types（route-macros 依赖它）
cargo publish -p route-macros-types

# 再发布 route-macros
cargo publish -p route-macros
```
