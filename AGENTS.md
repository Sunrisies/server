# AGENTS.md — 博客后端（server）

个人博客后端：Rust 2024 edition + Actix-web 4 + SeaORM + PostgreSQL。
仓库：`github.com/Sunrisies/server`，主分支 `main`。

## 快速开始

```bash
cp .env.example .env     # 填 DATABASE_URL 和 JWT_SECRET（必填，>=32 字节）
cargo run                # 监听 SERVER_HOST:SERVER_PORT
```

- `JWT_SECRET` 为空或过短会直接启动失败；轮换该值会让所有已签发 token（含频道 token）失效。
- 公开注册已关闭（`/api/v1/auth/register` 固定 403），新用户需在后台由管理员创建。

## 测试

测试会连接真实 PostgreSQL，**库名必须以 `_test` 结尾且 host 为 localhost/127.0.0.1**，
启动时会 drop/recreate 表，务必使用独立测试库。

```bash
export DATABASE_URL_TEST='postgres://postgres:local-ci-only@127.0.0.1:55440/server_acceptance_test'
export JWT_SECRET='test-only-signing-key-never-use-in-production'
cargo test --locked -- --test-threads=1
```

本机环境：Windows 的 5432 被 navicat 隧道占用，测试库跑在 **WSL Ubuntu-22.04 的 PostgreSQL 14**，
端口 `55440`，用户 `postgres` / 密码 `local-ci-only`，库 `server_acceptance_test`。
（原端口 `55432` 的 IPv4 段被一个无属主的进程占用，导致 postgres 只能绑 IPv6、TCP 认证失败 `28P01`，故换到 55440。）
启动方式：`wsl -d Ubuntu-22.04 -- sudo service postgresql start`。
编译产物建议放 Linux 文件系统（`CARGO_TARGET_DIR=/root/target`），避免 `/mnt/d` 上编译过慢。

## 提交前检查

`.git/hooks/pre-commit`（pre-commit.com）会运行 fmt、clippy、cargo-deny、typos 等，必须全部通过：

```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
```

CI：`.github/workflows/auth-security.yml`（ubuntu + postgres:17-alpine，`cargo test --locked -- --test-threads=1`）。

## 目录结构

- `src/handlers/`：HTTP 处理器，按资源分文件
- `src/services/`：业务逻辑与事务
- `src/models/`：SeaORM 实体
- `src/dto/`：请求/响应结构（含 validator 校验）
- `src/middleware/auth.rs`：统一鉴权入口
- `src/sql/`：需要手工执行或由初始化代码 embed 的 SQL

## 鉴权与权限约定

- 登录态是 httpOnly cookie `access_token`；公开路径白名单集中在 `src/middleware/auth.rs` 的 `public_request()`。
- 管理员权限只信数据库 `user_roles`/`roles`，**不信任 JWT 里的 `role_id`**。
- 注销后的 token 记录在 `revoked_tokens`（存 SHA-256 指纹）。
- 公开写接口：`/auth/login|logout`、`/email/send-verification-code`、`/feedback`、`/subscriptions`、`/drop/*`。
  其中 feedback/subscriptions 目前**没有频率限制**，是已知遗留项。
- `/api/v1/clipboard` 整体公开，但 `/clipboard/channel` 与 `/clipboard/my-channel` 在 handler 内要求登录。

## 文章与时间约定（重要）

- `posts.status`：`0` 草稿、`1` 发布、`2` 下线；非管理员只能看到 `status = 1`。
- `published_at` 非空：普通创建写当前时间；`CreatePostRequest.publish_time` / `UpdatePostRequest.publish_time`
  可显式指定（RFC3339），用于 CSDN 等外部同步。
- 首次发布（`0 → 1`）时，只有 `published_at == created_at`（从未显式指定过）才改写为当前时间，
  否则保留原值。因此同步来的草稿发布后仍显示原文发布时间；普通草稿行为不变。
- 时间统一存 UTC，响应通过 `fmt_beijing` 格式化为北京时间。
- 文章列表/详情/前后篇/时间轴/搜索均强制 `status = 1`（管理员除外），改动查询时注意保持。

## 数据库注意

- 多处 DDL 是 PostgreSQL 专用（`JSONB`、`TIMESTAMPTZ`、`BIGSERIAL`、`ON CONFLICT`），
  虽然 `src/config/db.rs` 支持 `sqlite://`，但启动初始化会在 SQLite 上失败，不要假设可切库。
- 部署前必须手工执行 `src/sql/performance_guards.sql`（`post_views` 唯一索引、`login_limits`）；
  缺索引会导致 `POST /api/v1/posts/{uuid}/view` 500。
- `src/sql/operations_schema.sql` 也是手工执行项，见 `docs/OPERATIONS.md`。

## 接口变更同步（必须遵守）

每次改动以下内容后，**立即**在 `../SYNC_LOG.md` 的「变更记录」区块顶部追加一条记录（按文件内模板格式）：
- `src/routes/` 新增或修改了路由路径
- `src/dto/` 请求/响应结构字段增删改
- `src/handlers/` 接口行为约定变更（状态码、鉴权要求等）
- `src/models/` 影响到对外接口的模型变更

追加后同步更新「last_read 指针」表中本会话的时间戳。
纯内部重构（不影响接口契约的）**不需要**记录。

## 安全 / 不要提交

- `src/bin/reset_pwd.rs` 里有明文生产库口令，**永远不要提交**（已在 `.gitignore` 排除）。
- 不要提交 `.env`、`.codegraph/`、`.reasonix/`。
- 上传目录 `data/images/`、`data/dropbox/`、日志目录均为运行时数据。

## 已知遗留（非阻塞，后续可做）

- feedback/subscriptions 无限流；订阅接口 `ON CONFLICT ... status='active'` 会把退订邮箱重新激活。
- 分页 `page` 只有下限没有上限。
- 部署/备份脚本存在中断风险（先停旧容器再验证新版本、空 dump 后清理备份等），详见 `docs/`。
- `HTTP_ACCESS_LOG_ENABLED=0` 仍会记录日志（只有字面量 `false` 才关闭）。
- 生产环境 `TRUSTED_PROXY_IPS` 未配置时，基于 IP 的限流会退化为全局计数。

## 最近变更

- 2026-11-14：认证 P0 加固合并（token 注销、登录限流、关闭公开注册、drop 上传类型/流式下载修复）。
- 2026-11-14：支持 `publish_time`，CSDN 同步草稿保留原始发布时间。
