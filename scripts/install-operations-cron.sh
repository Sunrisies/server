#!/usr/bin/env bash
# 在生产机执行一次。可选：先 export ALERT_WEBHOOK=你的告警地址。
set -euo pipefail
BASE_DIR="${BASE_DIR:-/home/docker/server}"
SCRIPT_DIR="$BASE_DIR/scripts"
mkdir -p "$SCRIPT_DIR"

cat > "$SCRIPT_DIR/blog-operations.cron" <<EOF
# 每 5 分钟检查前后端和容器状态
*/5 * * * * $SCRIPT_DIR/monitor-production.sh >> $BASE_DIR/monitor-cron.log 2>&1
# 每天北京时间 03:20 备份并验证数据库
20 3 * * * $SCRIPT_DIR/backup-production.sh >> $BASE_DIR/backup-cron.log 2>&1
EOF
# 保留机器上已有的定时任务，仅替换本项目的两条任务。
{ crontab -l 2>/dev/null || true; } \
  | grep -v "$SCRIPT_DIR/monitor-production.sh" \
  | grep -v "$SCRIPT_DIR/backup-production.sh" \
  > "$SCRIPT_DIR/existing.cron"
cat "$SCRIPT_DIR/existing.cron" "$SCRIPT_DIR/blog-operations.cron" | crontab -
rm -f "$SCRIPT_DIR/existing.cron"
crontab -l
