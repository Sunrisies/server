# Rust Server

这是一个使用 Rust 和 Actix-web 框架编写的博客后端服务。

## 技术栈

- `web框架`: Actix-web 框架编写的Rust后端服务
- `序列化`: Serde - 高效的序列化/反序列化
- `检验`: validator - 用于校验用户输入的库

## 功能特性

- 用户认证与授权
- 博客文章的增删改查
- 评论系统
- 标签管理
- RESTful API 设计

## 安装与运行

### 前置要求

- Rust (推荐最新稳定版)
- Cargo (Rust 包管理器，通常随 Rust 一起安装)

### 安装步骤

1. 克隆仓库
   ```bash
   git clone [仓库地址]
   cd blog/server
   ```

2. 安装依赖
   ```bash
   cargo build
   ```

3. 运行服务器
   ```bash
   cargo run
   ```

### 开发模式

为了在开发时自动重启服务器，可以使用 watchexec：

```powershell
# Windows PowerShell
$env:CRUD_MACRO_DEBUG=1
echo $env:CRUD_MACRO_DEBUG
watchexec -w src -w route-macros -r cargo run
```

```bash
# Linux/macOS
export CRUD_MACRO_DEBUG=1
echo $CRUD_MACRO_DEBUG
watchexec -w src -w route-macros -r cargo run
```

## API 文档

服务器启动后，可以通过以下地址访问 API 文档：

- Swagger UI: http://localhost:8080/swagger-ui/
- OpenAPI JSON: http://localhost:8080/api-docs/openapi.json

## 项目结构

```
server/
├── src/              # 源代码目录
├── route-macros/     # 自定义路由宏
├── Cargo.toml        # 项目配置文件
└── README.md         # 项目说明文档
```

## 贡献指南

欢迎提交 Issue 和 Pull Request 来帮助改进项目。

## 许可证

本项目采用 MIT 许可证。
