# 生产运维

## 数据库迁移

部署前先运行：

```bash
DB_URL="$(docker exec web-server printenv DATABASE_URL)"
docker exec -e DATABASE_URL="$DB_URL" postgres sh -lc \
  '/opt/bitnami/postgresql/bin/psql "$DATABASE_URL" -v ON_ERROR_STOP=1' \
  < sql/operations_schema.sql
```

该 SQL 使用 `IF NOT EXISTS`，可以安全重复执行。新增表或索引必须先追加到此文件，再发布依赖它的代码。

## 备份、校验与恢复演练

将 `scripts/backup-production.sh`、`scripts/monitor-production.sh` 和
`scripts/install-operations-cron.sh` 上传到 `/home/docker/server/scripts/`，赋予可执行权限，
在生产机运行 `install-operations-cron.sh`。备份每天 03:20 执行，保留 14 天，并会通过
`pg_restore --list` 校验归档完整性。

恢复必须先在隔离数据库演练，确认后才允许覆盖生产库：

```bash
gzip -dc /home/docker/server/backups/postgres/blog-YYYYMMDD-HHMMSS.dump.gz | \
  docker exec -i postgres /opt/bitnami/postgresql/bin/pg_restore --clean --if-exists --no-owner -d "$DATABASE_URL"
```

## 监控与日志

每 5 分钟检查前端、后端与两个容器。设置 `ALERT_WEBHOOK` 后失败将推送告警。后端应用日志挂载到
`/home/docker/server/web-server/`，查询最近错误：

```bash
docker logs web-server --since 1h | grep -E 'ERROR|WARN'
```
