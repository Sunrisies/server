# 后端发布脚本使用文档（deploy-production.sh）

本文档描述 `scripts/deploy-production.sh` 的实际用法、执行流程与回滚方式。
配套脚本：`scripts/apply-operations-schema.sh`（数据库增量）。
流程性说明另见 `docs/git-and-production-release.md`。

## 一、这个脚本做什么

在**生产机原生构建**（不在本机交叉编译），先起候选容器验收，再切换正式端口：

```
本机 cargo 检查 → 打包源码 → scp → 生产机 docker build
  → 候选容器（随机本地端口）验收 → 切换正式容器 22345 → 复验
```

- 正式容器名：`web-server`，宿主端口 `22345`
- 对外地址：`https://api.sunrise1024.top`
- 运行时密钥只来自生产机上的环境文件，**不进仓库、不进镜像**

## 二、前置条件

| 项 | 说明 |
|---|---|
| 本机 Rust 工具链 | 脚本会先跑 `cargo fmt --check` 和 `cargo check -q`，缺了直接失败 |
| 本机 git | 用于记录 `gitHash` / `gitBranch` 写进 `version.json` |
| 生产机 Docker | 构建和运行都在 `api.sunrise1024.top` 上完成 |
| 生产机 python3 | 用于改写 `.docker/version.json` |
| 网络 `my_app_network` | 候选与正式容器都接这个 Docker 网络 |
| 环境文件 | `/home/docker/server/releases/20261017-backend/production.runtime.env` |
| SSH 免密 | 脚本直接 `ssh root@api.sunrise1024.top` |

数据库连接、`JWT_SECRET` 等**全部**来自上面那个 `production.runtime.env`，
仓库里的 `.env` 只用于本地开发。

## 三、用法

```bash
cd /d/project/user/blog/server

# 推荐：显式传发布标签
export RELEASE=0.1.22-$(date +%Y%m%d-%H%M%S)
bash scripts/deploy-production.sh "$RELEASE"
```

不传参数时标签会退化成 `0.1.20-<时间戳>`——**注意这个 `0.1.20` 是写死的旧值**，
不会自动跟随当前版本，所以日常发布请自己传 `$RELEASE`。

标签必须匹配 `^[A-Za-z0-9._-]+$`，否则 `exit 2`。

### 涉及数据库结构时，先跑增量

```bash
bash scripts/apply-operations-schema.sh
```

`sql/operations_schema.sql` 全部使用 `IF NOT EXISTS`，可重复执行。
**必须先于新容器执行**，否则实体里新增的列会让所有查询报
`column ... does not exist`。脚本内部会：

1. `scp sql/operations_schema.sql` 到生产机 `/tmp`
2. `docker exec web-server printenv DATABASE_URL` 取连接串
3. 从中截取密码（避免明文出现在 `docker inspect` 里）
4. `docker cp` 进 postgres 容器，用 `/opt/bitnami/postgresql/bin/psql -v ON_ERROR_STOP=1` 执行
5. 删除临时文件

若取不到 `postgres://` 开头的连接串会直接 `exit 1`。

## 四、脚本执行的 9 个步骤

1. **校验标签**：正则不匹配 → `exit 2`
2. **本机检查**：`cargo fmt --check` + `cargo check -q`（任一失败即中止，未动生产）
3. **打包源码**：`target/web-server-$RELEASE-src.tar.gz`（**相对路径**，见"常见问题 1"）
   - 排除：`.git` `target` `.env` `.env.*` `temp_uploads` `data` `logs` `.codegraph` `.reasonix` `src/bin`
4. **密钥扫描**：包里若含 `.env*` / `reset_pwd.rs` / `gen_hash.rs`
   → 打印 `Refusing to ship secrets` 并 `exit 3`，同时删掉归档
5. **上传**：`scp` 到 `root@api.sunrise1024.top:/home/docker/server/releases/20261017-backend/`
6. **远端构建**：解压 → python3 改写 `.docker/version.json`
   （写入 `version` / `build` / `buildDate` / `buildTimestamp` / `gitHash` / `gitBranch`）→ `docker build`
7. **候选验收**：起 `web-server-candidate-$RELEASE`
   - `--network my_app_network`，`-p 127.0.0.1::2345`（随机本地端口）
   - `--env-file production.runtime.env`
   - 挂载 `/home/docker/server/web-server-candidate-logs` → `/app/logs`
   - 挂载 `/home/docker/server/dropbox` → `/app/data/dropbox`
   - 最多轮询 30 次 × 1 秒，两个条件都过才算通过：
     - `GET /api/v1/version` 的 `data.build` 等于本次 `$RELEASE`
     - `GET /api/v1/posts?page=1&limit=1` 成功
   - 超时则打印候选容器最后 80 行日志并 `exit 1`，**正式容器未被触碰**
8. **切换正式端口**（`trap rollback ERR` 兜底）：
   - `docker stop web-server` → 改名为 `web-server-rollback-$RELEASE`
   - 起新的 `web-server`：`-p 22345:2345 --restart always`，同样接 `my_app_network`、同样挂载
   - `sleep 10` 后 `curl http://127.0.0.1:22345/api/v1/version`
   - **任一步失败自动回滚**：删掉半成品、把 rollback 改回 `web-server` 并启动
9. **收尾**：删掉候选容器，输出最终 `version.json`，再从宿主机 `curl` 一次确认

## 五、验收

部署脚本自己会打印最终版本 JSON，另外手动确认：

```bash
curl -s https://api.sunrise1024.top/api/v1/version
curl -s "https://api.sunrise1024.top/api/v1/posts?page=1&limit=1"
```

预期 `data.build` 等于你传入的 `$RELEASE`，`data.gitHash` 等于本机 `git rev-parse --short HEAD`。

## 六、回滚

脚本把上一版保留为 `web-server-rollback-<标签>`，需要时手动切回：

```bash
ssh root@api.sunrise1024.top '
  docker stop web-server
  docker rm -f web-server
  docker rename web-server-rollback-0.1.21-20261117-144607 web-server
  docker start web-server
  sleep 5
  curl -fsS http://127.0.0.1:22345/api/v1/version
'
```

先 `docker logs <失败容器> --tail 200` 留证据再回滚。

## 七、常见问题

### 1. `scp: stat local "/tmp/xxx.tar.gz": No such file or directory`

本机 PATH 里的是 **Windows OpenSSH 的 scp**，它不认 MSYS 风格的 `/tmp` 绝对路径。
脚本已改用仓库内相对路径 `target/...`（两边 scp 都按各自 cwd 解析），
所以**不要把归档路径改回 `/tmp`**。若你新增脚本请沿用同样做法。

### 2. 构建卡在 `apk add`（尤其 `Installing g++`）

生产机在境内，官方源 `dl-cdn.alpinelinux.org` 从**容器内**访问会停滞
（宿主机反而正常，实测容器内 25 秒只下了 1.8MB）。
`Dockerfile` 已内置切换腾讯云镜像：

```dockerfile
RUN sed -i 's#dl-cdn.alpinelinux.org#mirrors.cloud.tencent.com#g' /etc/apk/repositories && \
    apk add --no-cache build-base openssl perl
```

同理容器内访问阿里云 CDN 也不通，**别改回 aliyun**。
实测腾讯云 / 清华源在容器内可达。

### 3. `Refusing to ship secrets: remove .env / src/bin from the archive`

归档里混进了密钥。`src/bin/reset_pwd.rs` 含**明文生产库口令**，
`src/bin/` 和 `reset_pwd.rs`、`gen_hash.rs` 已在 `.gitignore` 与 tar 排除列表里，
请检查是否有别处把 `.env` 打进去了。

### 4. 候选容器验收超时

```bash
ssh root@api.sunrise1024.top '
  docker logs web-server-candidate-<标签> --tail 80
'
```

常见原因：数据库结构没先跑 `apply-operations-schema.sh`、
`production.runtime.env` 缺变量、`JWT_SECRET` 为空或短于 32 字节。

### 5. 测试连不上数据库

本机测试库在 **WSL Ubuntu-22.04 的 PostgreSQL 14，端口 55440**（不是 5432）：

```bash
wsl -d Ubuntu-22.04 -- sudo service postgresql start
export DATABASE_URL_TEST='postgres://postgres:local-ci-only@127.0.0.1:55440/server_acceptance_test'
export JWT_SECRET='test-only-signing-key-never-use-in-production'
cargo test --locked -- --test-threads=1
```

库名必须以 `_test` 结尾、host 必须是 `localhost`/`127.0.0.1`，
否则 `src/test_helpers.rs` 的断言会直接拒绝。

## 八、相关文件

| 文件 | 作用 |
|---|---|
| `scripts/deploy-production.sh` | 发布主脚本 |
| `scripts/apply-operations-schema.sh` | 数据库增量（发布前） |
| `sql/operations_schema.sql` | 增量 SQL，全部 `IF NOT EXISTS` |
| `src/sql/performance_guards.sql` | `post_views` 唯一索引 + `login_limits`，需手工执行过一次 |
| `docs/OPERATIONS.md` | 备份、监控、手工 SQL 说明 |
| `docs/git-and-production-release.md` | 提交与发布的完整流程 |
