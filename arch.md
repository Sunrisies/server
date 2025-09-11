# 项目架构文档

## 1. 项目概述
本项目是一个基于 Actix-Web 框架的博客服务器，提供博客文章、分类、用户认证等功能。

## 2. 模块划分
- **main.rs**: 入口文件，初始化服务器并配置路由。
- **lib.rs**: 定义核心模块和公共接口。
- **config**: 包含数据库连接池创建和错误处理。
- **routes**: 定义 API 路由配置。
- **models**: 数据模型定义。
- **dto**: 数据传输对象。
- **middleware**: 中间件实现。
- **utils**: 工具类模块（如 SSE 支持）。
- **services**: 业务逻辑实现。
- **handlers**: 请求处理逻辑。

## 3. 依赖关系
- **main.rs** 依赖 `lib.rs` 中的 `config_routes` 和 `create_db_pool`。
- **lib.rs** 导出所有核心模块供其他模块使用。
- **routes** 依赖 `handlers` 和 `services` 处理请求。

## 4. 数据流
1. 请求通过 `routes` 模块路由到对应的 `handlers`。
2. `handlers` 调用 `services` 执行业务逻辑。
3. `services` 通过 `models` 与数据库交互。
4. 结果通过 `dto` 返回给客户端。

## 5. 关键配置
- **数据库连接**: 通过 `config::create_db_pool` 初始化。
- **CORS 配置**: 在 `main.rs` 中定义允许的源和方法。
- **SSE 支持**: 通过 `utils::sse` 实现。
