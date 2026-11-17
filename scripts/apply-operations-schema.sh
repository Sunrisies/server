#!/usr/bin/env bash
# 在生产库执行 sql/operations_schema.sql（幂等，可重复执行）。
# 用法：bash scripts/apply-operations-schema.sh
set -euo pipefail

readonly ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
readonly HOST="api.sunrise1024.top"
readonly APP_CONTAINER="web-server"
readonly DB_CONTAINER="postgres"
readonly REMOTE_SQL="/tmp/operations_schema.sql"

cd "$ROOT_DIR"

scp -q sql/operations_schema.sql "root@$HOST:$REMOTE_SQL"

DATABASE_URL="$(ssh "root@$HOST" "docker exec $APP_CONTAINER printenv DATABASE_URL")"
if [[ "$DATABASE_URL" != postgres://* ]]; then
  echo "Unexpected DATABASE_URL from $APP_CONTAINER" >&2
  exit 1
fi

PASSWORD="${DATABASE_URL#postgres://*:}"
PASSWORD="${PASSWORD%%@*}"

ssh "root@$HOST" "docker cp $REMOTE_SQL $DB_CONTAINER:$REMOTE_SQL && docker exec -e PGPASSWORD='$PASSWORD' $DB_CONTAINER /opt/bitnami/postgresql/bin/psql \
  -h 127.0.0.1 -U postgres -d database -v ON_ERROR_STOP=1 -f $REMOTE_SQL"
ssh "root@$HOST" "rm -f $REMOTE_SQL"
echo "operations_schema.sql applied"
