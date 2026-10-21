# 后端 Git 提交与生产发布流程

后端生产 API 为 `https://api.sunrise1024.top`，正式容器名为 `web-server`，主机端口为 `22345`。运行时密钥和数据库连接仅来自服务器上的 `/home/docker/server/releases/20261017-backend/production.runtime.env`，不得复制到仓库或镜像中。

## 1. 提交前检查

在后端项目根目录执行：

```bash
cargo fmt --check
cargo check -q
git diff --check
git status --short
```

如本次涉及数据库结构，先确认迁移是可重复执行的增量操作，并对生产数据库做备份后再切换容器。

## 2. 构建、候选验收与生产切换

生产机是 x86_64，因此由生产机原生 Docker 构建，避免在 Apple 芯片上模拟 x86。发布脚本会：打包不含 `.env` 的源码、上传到独立发布目录、写入本次发布标签、时间和 Git 提交号、构建镜像、启动随机本地端口的候选容器并检查版本和公开文章接口，最后切换 `22345` 正式端口。运行时密钥始终来自受保护的环境文件。

临时文件箱的文本、图片、音频和视频文件保存在服务器 `/home/docker/server/dropbox/`，并挂载到容器 `/app/data/dropbox/`。候选和正式容器共用该持久目录；文件 24 小时后由后端自动清理。免登录上传仅允许文本，媒体上传要求有效登录会话。

```bash
export RELEASE=0.1.20-$(date +%Y%m%d-%H%M%S)
bash scripts/deploy-production.sh "$RELEASE"
curl --fail https://api.sunrise1024.top/api/v1/version
```

脚本将旧容器保留为 `web-server-rollback-$RELEASE`。若新版本验收失败，停止当前 `web-server`，将该容器改回 `web-server` 后启动；同时保存失败容器日志以便排查。

若新版本未通过验收，停止当前 `web-server`，将 `web-server-rollback-$RELEASE` 改回 `web-server` 后启动；同时保存失败容器日志以便排查。

## 4. Git 提交与推送

当前开发分支为 `codex/fix-auth-p0`。完成线上验收后提交本次代码与文档：

```bash
git add -A
git commit -m "feat: add publishing analytics and version history"
git push origin codex/fix-auth-p0
```

推送后创建合并请求或按团队流程合并到主分支。最后用 `git status --short` 确认工作区干净，并记录发布镜像标签、提交哈希与验收结果。
