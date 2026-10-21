#!/usr/bin/env bash
# 供 cron 调用。失败会写入日志；设置 ALERT_WEBHOOK 可把故障发到兼容 webhook。
set -euo pipefail

API_URL="${API_URL:-https://api.sunrise1024.top/api/v1/version}"
WEB_URL="${WEB_URL:-https://blog.sunrise1024.top/api/ready}"
LOG_FILE="${LOG_FILE:-/home/docker/server/monitor.log}"
ALERT_WEBHOOK="${ALERT_WEBHOOK:-}"

fail() {
  local message="$1"
  printf '%s ERROR %s\n' "$(date -Is)" "$message" >> "$LOG_FILE"
  if [ -n "$ALERT_WEBHOOK" ]; then
    curl -fsS --max-time 10 -H 'Content-Type: application/json' \
      -d "{\"text\":\"Blog production alert: $message\"}" "$ALERT_WEBHOOK" >/dev/null || true
  fi
  exit 1
}

curl -fsS --max-time 10 "$API_URL" >/dev/null || fail 'backend health check failed'
curl -fsS --max-time 10 "$WEB_URL" >/dev/null || fail 'frontend health check failed'
docker inspect -f '{{.State.Running}}' web-server | grep -qx true || fail 'web-server is not running'
docker inspect -f '{{.State.Running}}' web_nuxt | grep -qx true || fail 'web_nuxt is not running'
printf '%s OK\n' "$(date -Is)" >> "$LOG_FILE"
