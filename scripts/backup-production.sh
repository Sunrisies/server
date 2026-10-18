#!/usr/bin/env bash
# 在生产机执行：创建压缩数据库备份、校验可读性，并保留最近 14 天。
set -euo pipefail

BASE_DIR="${BASE_DIR:-/home/docker/server}"
POSTGRES_CONTAINER="${POSTGRES_CONTAINER:-postgres}"
SERVER_CONTAINER="${SERVER_CONTAINER:-web-server}"
BACKUP_DIR="${BACKUP_DIR:-$BASE_DIR/backups/postgres}"
KEEP_DAYS="${KEEP_DAYS:-14}"
STAMP="$(date +%Y%m%d-%H%M%S)"
ARCHIVE="$BACKUP_DIR/blog-$STAMP.dump.gz"

mkdir -p "$BACKUP_DIR"
DB_URL="$(docker exec "$SERVER_CONTAINER" printenv DATABASE_URL)"
docker exec -e DATABASE_URL="$DB_URL" "$POSTGRES_CONTAINER" sh -lc \
  '/opt/bitnami/postgresql/bin/pg_dump --format=custom --no-owner --no-privileges "$DATABASE_URL"' \
  | gzip -9 > "$ARCHIVE"

# 防止把空文件或截断文件当作成功备份。
test -s "$ARCHIVE"
gzip -t "$ARCHIVE"
gzip -dc "$ARCHIVE" | docker exec -i "$POSTGRES_CONTAINER" \
  /opt/bitnami/postgresql/bin/pg_restore --list >/dev/null

find "$BACKUP_DIR" -type f -name 'blog-*.dump.gz' -mtime "+$KEEP_DAYS" -delete
printf 'backup=%s bytes=%s\n' "$ARCHIVE" "$(wc -c < "$ARCHIVE")"
