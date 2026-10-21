#!/usr/bin/env bash

# 在生产主机原生构建，先启动候选容器，再切换正式端口。
# 使用方式：bash scripts/deploy-production.sh [release-tag]
set -euo pipefail

readonly ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
readonly HOST="api.sunrise1024.top"
readonly GIT_HASH="$(git rev-parse --short HEAD 2>/dev/null || echo unknown)"
readonly GIT_BRANCH="$(git branch --show-current 2>/dev/null || echo unknown)"
readonly REMOTE_BASE="/home/docker/server/releases/20261017-backend"
readonly RUNTIME_ENV="$REMOTE_BASE/production.runtime.env"
readonly RELEASE="${1:-0.1.20-$(date +%Y%m%d-%H%M%S)}"
readonly IMAGE="web-server:$RELEASE"
readonly ARCHIVE="/tmp/web-server-$RELEASE-src.tar.gz"
readonly REMOTE_RELEASE="$REMOTE_BASE/$RELEASE"
readonly CANDIDATE="web-server-candidate-$RELEASE"
readonly ROLLBACK="web-server-rollback-$RELEASE"

if [[ ! "$RELEASE" =~ ^[A-Za-z0-9._-]+$ ]]; then
  echo "Invalid release tag: $RELEASE" >&2
  exit 2
fi

cd "$ROOT_DIR"
cargo fmt --check
cargo check -q

tar --exclude=.git --exclude=target --exclude=.env --exclude=temp_uploads \
  -czf "$ARCHIVE" .
scp "$ARCHIVE" "root@$HOST:$REMOTE_BASE/"

ssh "root@$HOST" "RELEASE='$RELEASE' IMAGE='$IMAGE' REMOTE_BASE='$REMOTE_BASE' RUNTIME_ENV='$RUNTIME_ENV' CANDIDATE='$CANDIDATE' ROLLBACK='$ROLLBACK' GIT_HASH='$GIT_HASH' GIT_BRANCH='$GIT_BRANCH' bash -s" <<'REMOTE'
set -euo pipefail
REMOTE_RELEASE="$REMOTE_BASE/$RELEASE"
mkdir -p "$REMOTE_RELEASE" /home/docker/server/web-server-candidate-logs /home/docker/server/dropbox
tar -xzf "$REMOTE_BASE/web-server-$RELEASE-src.tar.gz" -C "$REMOTE_RELEASE"
python3 - "$REMOTE_RELEASE/.docker/version.json" "$RELEASE" "$GIT_HASH" "$GIT_BRANCH" <<'PY'
import json
import sys
from datetime import datetime, timezone
from pathlib import Path

path = Path(sys.argv[1])
data = json.loads(path.read_text())
now = datetime.now(timezone.utc).astimezone()
data.update({
    "previousVersion": data.get("version", "unknown"),
    "version": sys.argv[2].split("-", 1)[0],
    "build": sys.argv[2],
    "buildDate": now.isoformat(timespec="seconds"),
    "buildTimestamp": now.strftime("%Y%m%d%H%M%S"),
    "gitHash": sys.argv[3],
    "gitBranch": sys.argv[4],
    "environment": "production",
})
temporary = path.with_suffix(".tmp")
temporary.write_text(json.dumps(data, ensure_ascii=False, indent=2) + "\n")
temporary.replace(path)
PY
docker build -t "$IMAGE" "$REMOTE_RELEASE"

docker rm -f "$CANDIDATE" >/dev/null 2>&1 || true
docker run -d --init --name "$CANDIDATE" --restart no --network my_app_network \
  -p 127.0.0.1::2345 --env-file "$RUNTIME_ENV" \
  -v /home/docker/server/web-server-candidate-logs:/app/logs \
  -v /home/docker/server/dropbox:/app/data/dropbox "$IMAGE" >/dev/null

for attempt in $(seq 1 30); do
  port="$(docker port "$CANDIDATE" 2345/tcp | sed 's/.*://')"
  if curl -fsS "http://127.0.0.1:$port/api/v1/version" | python3 -c 'import json,sys; sys.exit(0 if json.load(sys.stdin)["data"]["build"] == sys.argv[1] else 1)' "$RELEASE" && \
     curl -fsS "http://127.0.0.1:$port/api/v1/posts?page=1&limit=1" >/dev/null; then
    break
  fi
  if [ "$attempt" = 30 ]; then
    docker logs "$CANDIDATE" --tail 80
    exit 1
  fi
  sleep 1
done

docker stop web-server >/dev/null
docker rename web-server "$ROLLBACK"
docker run -d --init --name web-server --restart always --network my_app_network \
  -p 22345:2345 --env-file "$RUNTIME_ENV" \
  -v /home/docker/server/web-server:/app/logs \
  -v /home/docker/server/dropbox:/app/data/dropbox "$IMAGE" >/dev/null
sleep 10
curl -fsS http://127.0.0.1:22345/api/v1/version >/dev/null
docker rm -f "$CANDIDATE" >/dev/null 2>&1 || true

curl -fsS http://127.0.0.1:22345/api/v1/version | python3 -c 'import json,sys; data=json.load(sys.stdin)["data"]; assert data["build"] == sys.argv[1], data; print(json.dumps(data, ensure_ascii=False))' "$RELEASE"
REMOTE

ssh "root@$HOST" "curl -fsS http://127.0.0.1:22345/api/v1/version"
