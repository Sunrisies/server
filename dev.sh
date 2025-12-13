#!/bin/bash

# 配置变量 - 将从 Cargo.toml 动态获取
PROJECT_NAME=""
IMAGE_NAME=""
LOCAL_TAR_FILE="web-server.tar"
REMOTE_USER="root"
REMOTE_HOST="api.chaoyang1024.top"
REMOTE_DIR="/home/docker/server"
CONTAINER_NAME="web-server"
PORT_MAPPING="22345:2345"
VOLUME_MAPPING="/home/docker/server/web_server:/home/app/logs"
KEEP_IMAGE_VERSIONS=2  # 本地保留最近2个版本

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# 版本管理
CURRENT_VERSION=""
NEW_VERSION=""
NEXT_VERSION=""
BUILD_TIMESTAMP=""

# 备份文件
BACKUP_CARGO_TOML=""

# 部署状态
DEPLOYMENT_SUCCESS=false

# 日志函数
log() {
  echo -e "${CYAN}[$(date +'%Y-%m-%d %H:%M:%S')] $1${NC}"
}

log_success() {
  echo -e "${GREEN}[$(date +'%Y-%m-%d %H:%M:%S')] $1${NC}"
}

log_error() {
  echo -e "${RED}[$(date +'%Y-%m-%d %H:%M:%S')] $1${NC}"
}

log_warning() {
  echo -e "${YELLOW}[$(date +'%Y-%m-%d %H:%M:%S')] $1${NC}"
}

# 检查命令是否执行成功
check_success() {
  if [ $? -ne 0 ]; then
    log_error "错误: $1 执行失败"
    return 1
  fi
  return 0
}

# 从 Cargo.toml 获取项目名称
get_project_name() {
  if [ -f "Cargo.toml" ]; then
    PROJECT_NAME=$(grep '^name =' Cargo.toml | head -1 | sed 's/name = "\(.*\)"/\1/' | tr -d '"' | tr -d ' ')
    # 使用项目名称作为镜像名称和容器名称的基础
    IMAGE_NAME="$PROJECT_NAME"
    CONTAINER_NAME="$PROJECT_NAME"
    LOCAL_TAR_FILE="$PROJECT_NAME.tar"
  else
    PROJECT_NAME="web-server"
    IMAGE_NAME="web-server"
    CONTAINER_NAME="web-server"
    LOCAL_TAR_FILE="web-server.tar"
    log_warning "未找到 Cargo.toml，使用默认名称: $PROJECT_NAME"
  fi
  echo "$PROJECT_NAME"
}

# 从 Cargo.toml 获取当前版本
get_current_version() {
  if [ -f "Cargo.toml" ]; then
    CURRENT_VERSION=$(grep '^version =' Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/' | tr -d '"' | tr -d ' ')
  else
    CURRENT_VERSION="unknown"
  fi
  echo "$CURRENT_VERSION"
}

# 递增版本号
increment_version() {
  local version=$1
  local increment_type=${2:-patch}  # major, minor, patch

  if [ "$version" = "unknown" ] || [ -z "$version" ]; then
    echo "0.0.1"
    return 0
  fi

  # 解析版本号
  local major=$(echo $version | cut -d. -f1)
  local minor=$(echo $version | cut -d. -f2)
  local patch=$(echo $version | cut -d. -f3)

  case $increment_type in
    "major")
      major=$((major + 1))
      minor=0
      patch=0
      ;;
    "minor")
      minor=$((minor + 1))
      patch=0
      ;;
    "patch"|*)
      patch=$((patch + 1))
      ;;
  esac

  echo "$major.$minor.$patch"
}

# 备份重要文件
backup_files() {
  log "备份重要文件..."

  # 备份 Cargo.toml
  if [ -f "Cargo.toml" ]; then
    BACKUP_CARGO_TOML=$(mktemp)
    cp Cargo.toml "$BACKUP_CARGO_TOML"
    log "已备份 Cargo.toml"
  fi
}

# 恢复备份文件
restore_backup_files() {
  log "恢复备份文件..."

  if [ -n "$BACKUP_CARGO_TOML" ] && [ -f "$BACKUP_CARGO_TOML" ]; then
    cp "$BACKUP_CARGO_TOML" Cargo.toml
    log "已恢复 Cargo.toml"
    rm -f "$BACKUP_CARGO_TOML"
  fi
}

# 清理备份文件
cleanup_backup_files() {
  if [ -n "$BACKUP_CARGO_TOML" ] && [ -f "$BACKUP_CARGO_TOML" ]; then
    rm -f "$BACKUP_CARGO_TOML"
  fi
}

# 更新版本号
update_version() {
  if [ -f "Cargo.toml" ]; then
    CURRENT_VERSION=$(get_current_version)

    # 递增版本号（默认递增patch版本）
    NEXT_VERSION=$(increment_version "$CURRENT_VERSION" "patch")

    # 更新 Cargo.toml 中的版本号
    sed -i "s/^version = \".*\"/version = \"$NEXT_VERSION\"/" Cargo.toml

    # 生成构建时间戳
    BUILD_TIMESTAMP=$(date +%Y%m%d%H%M%S)
    NEW_VERSION="${NEXT_VERSION}-${BUILD_TIMESTAMP}"

    log "版本号已更新: $CURRENT_VERSION → $NEXT_VERSION (构建标签: $NEW_VERSION)"
  else
    NEW_VERSION="$(date +%Y%m%d%H%M%S)"
    NEXT_VERSION="unknown"
    log "未找到 Cargo.toml，使用时间戳作为版本号: $NEW_VERSION"
  fi
}

# 回滚版本号（在部署失败时使用）
revert_version() {
  if [ -f "Cargo.toml" ] && [ -n "$CURRENT_VERSION" ] && [ "$CURRENT_VERSION" != "unknown" ]; then
    sed -i "s/^version = \".*\"/version = \"$CURRENT_VERSION\"/" Cargo.toml
    log_warning "版本号已回退到: $CURRENT_VERSION"
  fi
}

# 生成版本信息
generate_version_info() {
  local git_hash=$(git rev-parse --short HEAD 2>/dev/null || echo "unknown")
  local git_branch=$(git branch --show-current 2>/dev/null || echo "unknown")

  local version_info="{
  \"name\": \"$PROJECT_NAME\",
  \"version\": \"${NEXT_VERSION:-unknown}\",
  \"previousVersion\": \"${CURRENT_VERSION:-unknown}\",
  \"build\": \"${NEW_VERSION}\",
  \"buildDate\": \"$(date -Iseconds)\",
  \"buildTimestamp\": \"$BUILD_TIMESTAMP\",
  \"gitHash\": \"$git_hash\",
  \"gitBranch\": \"$git_branch\",
  \"environment\": \"production\",
  \"language\": \"rust\"
}"

  # 创建版本信息文件，在 Docker 构建时会复制到镜像中
  mkdir -p .docker
  echo "$version_info" > ".docker/version.json"
  log "版本信息已生成: $CURRENT_VERSION → $NEXT_VERSION (Git: $git_hash)"
}

# 提交版本更新到 Git（可选）
commit_version_update() {
  log "检查 Git 提交..."

  if [ -d ".git" ] && git status --porcelain Cargo.toml | grep -q "Cargo.toml"; then
    log "正在提交版本更新到 Git..."
    git add Cargo.toml
    git commit -m "chore: bump version to $NEXT_VERSION [deploy]"

    # 可以选择是否自动推送
    read -p "是否推送版本更新到远程仓库? (y/N): " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
      git push
      log_success "版本更新已推送到远程仓库"
    else
      log_warning "版本更新已提交但未推送，请手动执行: git push"
    fi
  else
    log "未检测到 Git 仓库或版本文件未更改，跳过 Git 提交"
  fi
}

# 检查 Docker 环境
check_docker_environment() {
  log "检查 Docker 环境..."

  if ! command -v docker &> /dev/null; then
    log_error "Docker 未安装"
    return 1
  fi

  if ! docker info &> /dev/null; then
    log_error "Docker 守护进程未运行或无权限访问"
    return 1
  fi

  log_success "Docker 环境检查通过"
}

# 运行测试（在宿主机上运行，确保代码质量）
run_tests() {
  log "正在运行 Rust 测试..."

  if command -v cargo &> /dev/null; then
    cargo test
    if [ $? -eq 0 ]; then
      log_success "所有测试通过"
    else
      log_warning "测试失败，但继续部署流程"
      # 根据需求决定是否在此处退出
      # return 1
    fi
  else
    log_warning "未找到 cargo 命令，跳过测试"
  fi
}

# 构建 Rust 项目
build_rust_project() {
  log "正在构建 Rust 项目..."

  if command -v cargo &> /dev/null; then
    cargo build --release
    check_success "Rust 项目构建"

    # 检查生成的可执行文件
    if [ -f "target/release/$PROJECT_NAME" ]; then
      log_success "Rust 项目构建成功，可执行文件大小: $(du -h target/release/$PROJECT_NAME | cut -f1)"
    else
      log_error "未找到构建的可执行文件: target/release/$PROJECT_NAME"
      return 1
    fi
  else
    log_error "未找到 cargo 命令"
    return 1
  fi
}

# 使用 Docker 构建项目
build_with_docker() {
  log "正在使用 Docker 构建 Rust 项目..."

  # 检查 Dockerfile 是否存在
  if [ ! -f "Dockerfile" ]; then
    log_error "未找到 Dockerfile"
    return 1
  fi

  # 构建镜像
  log "构建 Docker 镜像: $IMAGE_NAME:$NEW_VERSION"
  docker build \
    --platform linux/amd64 \
    -t $IMAGE_NAME:$NEW_VERSION \
    -t $IMAGE_NAME:latest \
    --build-arg BUILD_TIMESTAMP=$BUILD_TIMESTAMP \
    --build-arg GIT_HASH=$(git rev-parse --short HEAD 2>/dev/null || echo "unknown") \
    --build-arg PROJECT_VERSION=$NEXT_VERSION \
    .

  check_success "Docker 镜像构建"

  # 验证镜像
  log "验证构建的镜像..."
  if docker image inspect $IMAGE_NAME:$NEW_VERSION &> /dev/null; then
    local image_size=$(docker images $IMAGE_NAME:$NEW_VERSION --format "{{.Size}}")
    log_success "镜像构建成功，大小: $image_size"
    return 0
  else
    log_error "镜像验证失败"
    return 1
  fi
}

# 保存 Docker 镜像为 tar 文件
save_docker_image() {
  log "正在将 Docker 镜像保存为 tar 文件..."
  docker save -o $LOCAL_TAR_FILE $IMAGE_NAME:$NEW_VERSION
  check_success "Docker 镜像保存"

  local file_size=$(du -h $LOCAL_TAR_FILE | cut -f1)
  log_success "镜像保存成功，文件大小: $file_size"
}

# 上传文件到远程服务器
upload_to_remote() {
  log "正在上传 $LOCAL_TAR_FILE 到 $REMOTE_HOST..."
  scp $LOCAL_TAR_FILE $REMOTE_USER@$REMOTE_HOST:$REMOTE_DIR/
  check_success "文件上传"
}

# 清理本地 Docker 镜像（当前构建的）
cleanup_local_images() {
  log "正在清理本地 Docker 镜像（当前构建的）..."

  # 删除特定版本的镜像
  if docker image inspect $IMAGE_NAME:$NEW_VERSION &> /dev/null; then
    docker rmi $IMAGE_NAME:$NEW_VERSION
    log_success "已删除本地镜像: $IMAGE_NAME:$NEW_VERSION"
  else
    log "本地镜像 $IMAGE_NAME:$NEW_VERSION 不存在，无需删除"
  fi

  # 清理悬空镜像
  local dangling_images=$(docker images -f "dangling=true" -q)
  if [ -n "$dangling_images" ]; then
    docker rmi $dangling_images 2>/dev/null || log "无法删除部分悬空镜像"
    log "已清理悬空镜像"
  fi
}

# 清理本地旧镜像（保留最近2次）
cleanup_old_local_images() {
  log "正在清理本地旧镜像（保留最近 $KEEP_IMAGE_VERSIONS 次构建）..."

  # 获取所有镜像标签，按构建时间戳排序
  local all_images=$(docker images --filter "reference=$IMAGE_NAME" --format "{{.Tag}}" | grep -E '^[0-9]+\.[0-9]+\.[0-9]+-[0-9]+$' | sort -r)

  if [ -z "$all_images" ]; then
    log "没有找到需要清理的旧镜像"
    return 0
  fi

  # 计算需要保留的镜像数量
  local keep_count=$KEEP_IMAGE_VERSIONS
  local total_images=$(echo "$all_images" | wc -l)

  if [ $total_images -le $keep_count ]; then
    log "当前只有 $total_images 个镜像，未超过保留限制 $keep_count 个，无需清理"
    return 0
  fi

  log "发现 $total_images 个镜像，保留最新的 $keep_count 个"

  # 获取需要删除的旧镜像（跳过前 $keep_count 个）
  local images_to_delete=$(echo "$all_images" | tail -n +$((keep_count + 1)))

  if [ -z "$images_to_delete" ]; then
    log "没有需要删除的旧镜像"
    return 0
  fi

  local deleted_count=0
  while IFS= read -r image_tag; do
    if [ -n "$image_tag" ]; then
      log "删除旧镜像: $IMAGE_NAME:$image_tag"
      if docker rmi "$IMAGE_NAME:$image_tag" 2>/dev/null; then
        ((deleted_count++))
      else
        log_warning "无法删除镜像 $IMAGE_NAME:$image_tag，可能正在被使用"
      fi
    fi
  done <<< "$images_to_delete"

  log_success "已删除 $deleted_count 个旧镜像，保留了最新的 $keep_count 个镜像"

  # 显示当前保留的镜像
  local remaining_images=$(docker images --filter "reference=$IMAGE_NAME" --format "table {{.Repository}}:{{.Tag}}\t{{.Size}}\t{{.CreatedSince}}" | grep -E "$IMAGE_NAME:" | head -n $((keep_count + 1)))
  if [ -n "$remaining_images" ]; then
    log "当前保留的镜像:"
    echo "$remaining_images"
  fi
}

# 清理远程服务器的旧镜像
cleanup_remote_images() {
  log "正在清理远程服务器的旧镜像..."

  local cleanup_script=$(cat << 'EOF'
set -e

cd REMOTE_DIR_PLACEHOLDER

# 定义远程日志函数
remote_log() { echo "[$(date +'%Y-%m-%d %H:%M:%S')] $1"; }

# 清理所有未使用的镜像（包括未打标签的）
remote_log "清理未使用的 Docker 镜像..."
docker image prune -f

# 清理指定项目的旧版本镜像（保留最近3个版本）
remote_log "清理 PROJECT_NAME_PLACEHOLDER 的旧版本镜像..."
docker images PROJECT_NAME_PLACEHOLDER --format "{{.Tag}}" | \
  grep -E '^[0-9]+\.[0-9]+\.[0-9]+-[0-9]+$' | \
  sort -Vr | \
  tail -n +4 | \
  while read tag; do
    remote_log "删除旧镜像: PROJECT_NAME_PLACEHOLDER:$tag"
    docker rmi PROJECT_NAME_PLACEHOLDER:$tag 2>/dev/null || true
  done

# 尝试删除悬空镜像（如果有）
remote_log "清理悬空镜像..."
docker images -f "dangling=true" -q | xargs -r docker rmi 2>/dev/null || true

remote_log "镜像清理完成"
EOF
)

  # 替换占位符
  cleanup_script=$(echo "$cleanup_script" | sed \
    -e "s|REMOTE_DIR_PLACEHOLDER|$REMOTE_DIR|g" \
    -e "s|PROJECT_NAME_PLACEHOLDER|$PROJECT_NAME|g")

  # 执行远程清理
  echo "$cleanup_script" | ssh $REMOTE_USER@$REMOTE_HOST /bin/bash
}

# Rust 项目健康检查
health_check() {
  log "正在进行健康检查..."

  # 等待容器启动
  log "等待容器启动..."
  sleep 15

  # 检查容器状态
  local container_status=$(ssh $REMOTE_USER@$REMOTE_HOST "docker ps --filter name=$CONTAINER_NAME --format 'table {{.Names}}\t{{.Status}}\t{{.Ports}}'")

  if echo "$container_status" | grep -q "$CONTAINER_NAME"; then
    log_success "容器运行状态:"
    echo "$container_status"

    # 多种健康检查方式
    local health_checked=false

    # 方式1: 检查健康检查端点
    log "检查健康端点..."
    if ssh $REMOTE_USER@$REMOTE_HOST "curl -f -s http://localhost:22345/health > /dev/null 2>&1"; then
      log_success "健康检查端点响应正常"
      health_checked=true
    else
      log_warning "健康检查端点响应失败，但继续检查其他方式"
    fi

    # 方式2: 检查基础连接
    if [ "$health_checked" = false ]; then
      log "检查基础连接..."
      if ssh $REMOTE_USER@$REMOTE_HOST "curl -f -s --connect-timeout 10 http://localhost:22345/ > /dev/null 2>&1"; then
        log_success "应用基础连接正常"
        health_checked=true
      else
        log_warning "基础连接检查失败，但继续检查日志"
      fi
    fi

    # 方式3: 检查容器日志
    if [ "$health_checked" = false ]; then
      log "检查容器日志..."
      local recent_logs=$(ssh $REMOTE_USER@$REMOTE_HOST "docker logs $CONTAINER_NAME --tail 20 2>/dev/null")
      if [ -n "$recent_logs" ]; then
        log_warning "健康检查失败，但容器正在运行。最近日志:"
        echo "$recent_logs" | tail -10

        # 检查日志中是否有关键错误（忽略认证相关的错误）
        local critical_errors=$(echo "$recent_logs" | grep -i "error" | grep -v "access_token not found")
        if [ -n "$critical_errors" ]; then
          log_error "日志中发现关键错误信息:"
          echo "$critical_errors"
          return 1
        else
          log_warning "未在日志中发现关键错误，主要是认证问题，可能应用启动较慢或配置问题"
          return 0  # 认证错误不算关键错误，返回成功
        fi
      else
        log_error "无法获取容器日志"
        return 1
      fi
    else
      return 0
    fi
  else
    log_error "容器未运行"
    # 获取失败原因
    local failed_logs=$(ssh $REMOTE_USER@$REMOTE_HOST "docker logs $CONTAINER_NAME 2>&1 | tail -20")
    if [ -n "$failed_logs" ]; then
      log_error "容器启动失败，日志:"
      echo "$failed_logs"
    fi
    return 1
  fi
}

# 远程部署
remote_deploy() {
  log "正在远程服务器上部署..."

  local deploy_script=$(cat << 'EOF'
set -e

cd REMOTE_DIR_PLACEHOLDER

# 定义远程日志函数
remote_log() { echo "[$(date +'%Y-%m-%d %H:%M:%S')] $1"; }

remote_log "停止旧容器..."
docker stop CONTAINER_NAME_PLACEHOLDER 2>/dev/null || remote_log "没有运行的容器需要停止"

remote_log "删除旧容器..."
docker rm CONTAINER_NAME_PLACEHOLDER 2>/dev/null || remote_log "没有容器需要删除"

remote_log "清理旧镜像..."
docker rmi IMAGE_NAME_PLACEHOLDER:latest 2>/dev/null || remote_log "没有 latest 镜像需要删除"
docker rmi IMAGE_NAME_PLACEHOLDER:NEW_VERSION_PLACEHOLDER 2>/dev/null || remote_log "没有指定版本镜像需要删除"

# 加载新镜像
remote_log "加载新镜像..."
docker load -i LOCAL_TAR_FILE_PLACEHOLDER

# 启动新容器
remote_log "启动新容器..."
docker run -d \
  -v VOLUME_MAPPING_PLACEHOLDER \
  -p PORT_MAPPING_PLACEHOLDER \
  --restart=always \
  --name CONTAINER_NAME_PLACEHOLDER \
  IMAGE_NAME_PLACEHOLDER:NEW_VERSION_PLACEHOLDER

remote_log "清理远程临时文件..."
rm -f LOCAL_TAR_FILE_PLACEHOLDER

remote_log "远程部署完成"
EOF
)

  # 替换占位符
  deploy_script=$(echo "$deploy_script" | sed \
    -e "s|REMOTE_DIR_PLACEHOLDER|$REMOTE_DIR|g" \
    -e "s|CONTAINER_NAME_PLACEHOLDER|$CONTAINER_NAME|g" \
    -e "s|IMAGE_NAME_PLACEHOLDER|$IMAGE_NAME|g" \
    -e "s|NEW_VERSION_PLACEHOLDER|$NEW_VERSION|g" \
    -e "s|LOCAL_TAR_FILE_PLACEHOLDER|$LOCAL_TAR_FILE|g" \
    -e "s|VOLUME_MAPPING_PLACEHOLDER|$VOLUME_MAPPING|g" \
    -e "s|PORT_MAPPING_PLACEHOLDER|$PORT_MAPPING|g")

  # 执行远程部署
  echo "$deploy_script" | ssh $REMOTE_USER@$REMOTE_HOST /bin/bash

  check_success "远程部署"
}

# 清理资源
cleanup() {
  log "正在清理本地临时文件..."
  rm -f $LOCAL_TAR_FILE
  rm -rf .docker/version.json 2>/dev/null || true
}

# 显示部署信息
show_deploy_info() {
  log_success "=========================================="
  log_success "🚀 Rust 服务部署成功完成!"
  log_success "应用名称: $PROJECT_NAME"
  log_success "版本升级: $CURRENT_VERSION → $NEXT_VERSION"
  log_success "构建标签: $NEW_VERSION"
  log_success "构建时间: $BUILD_TIMESTAMP"
  log_success "服务地址: $REMOTE_HOST:$PORT_MAPPING"
  log_success "容器名称: $CONTAINER_NAME"
  log_success "本地保留镜像: 最近 $KEEP_IMAGE_VERSIONS 次构建"
  log_success "部署时间: $(date)"
  log_success "=========================================="
}

# 显示部署警告信息
show_deploy_warning() {
  log_warning "=========================================="
  log_warning "⚠️  Rust 服务部署完成，但有警告!"
  log_warning "应用名称: $PROJECT_NAME"
  log_warning "版本升级: $CURRENT_VERSION → $NEXT_VERSION"
  log_warning "构建标签: $NEW_VERSION"
  log_warning "构建时间: $BUILD_TIMESTAMP"
  log_warning "服务地址: $REMOTE_HOST:$PORT_MAPPING"
  log_warning "容器名称: $CONTAINER_NAME"
  log_warning "状态: 容器已运行，但健康检查未完全通过"
  log_warning "注意: 应用可能存在认证配置问题"
  log_warning "部署时间: $(date)"
  log_warning "=========================================="
}

# 主部署流程
main() {
  log "开始 Rust 项目 Docker 部署流程..."

  # 设置错误处理
  set -e

  # 0. 获取项目名称和设置相关变量
  get_project_name
  log "项目名称: $PROJECT_NAME"
  log "镜像名称: $IMAGE_NAME"
  log "容器名称: $CONTAINER_NAME"
  log "本地打包文件: $LOCAL_TAR_FILE"

  # 0.1 备份重要文件
  backup_files

  # 1. 检查 Docker 环境
  check_docker_environment

  # 2. 更新版本号
  if ! update_version; then
    log_error "版本号更新失败"
    restore_backup_files
    exit 1
  fi

  # 3. 生成版本信息
  generate_version_info

  # 4. 运行测试（可选）
  # run_tests

  # 5. 构建 Rust 项目
  if ! build_rust_project; then
    log_error "Rust 项目构建失败，正在回退版本..."
    restore_backup_files
    exit 1
  fi

  # 6. 使用 Docker 构建项目
  if ! build_with_docker; then
    log_error "Docker 构建失败，正在回退版本..."
    restore_backup_files
    exit 1
  fi

  # 7. 保存 Docker 镜像
  if ! save_docker_image; then
    log_error "Docker 镜像保存失败，正在回退版本..."
    restore_backup_files
    cleanup
    exit 1
  fi

  # 8. 上传到远程
  if ! upload_to_remote; then
    log_error "文件上传失败，正在回退版本..."
    restore_backup_files
    cleanup
    cleanup_local_images
    exit 1
  fi

  # 9. 远程部署
  if ! remote_deploy; then
    log_error "远程部署失败，正在回退版本..."
    restore_backup_files
    cleanup
    cleanup_local_images
    exit 1
  fi

  # 10. 清理远程旧镜像
  cleanup_remote_images

  # 11. 健康检查
  local health_status=0
  health_check || health_status=$?

  if [ $health_status -eq 0 ]; then
    log_success "健康检查通过"
    DEPLOYMENT_SUCCESS=true
  else
    log_warning "健康检查未完全通过，但部署已完成"
    DEPLOYMENT_SUCCESS=true  # 仍然标记为成功，因为容器在运行
  fi

  # 12. 清理本地旧镜像（保留最近2次）- 仅在部署成功时执行
  # if [ "$DEPLOYMENT_SUCCESS" = true ]; then
  #   cleanup_old_local_images
  # fi

  # 13. 清理资源
  cleanup

  # 14. 清理备份文件（部署成功后才清理备份）
  if [ "$DEPLOYMENT_SUCCESS" = true ]; then
    cleanup_backup_files
  fi

  # 15. 提交版本更新到 Git（可选）- 仅在部署成功时执行
  if [ "$DEPLOYMENT_SUCCESS" = true ]; then
    commit_version_update
  fi

  # 16. 显示部署信息
  if [ "$DEPLOYMENT_SUCCESS" = true ]; then
    if [ $health_status -eq 0 ]; then
      show_deploy_info
    else
      show_deploy_warning
    fi
  else
    log_error "部署失败"
    exit 1
  fi
}

# 错误处理
handle_error() {
  local exit_code=$?
  log_error "部署过程发生错误，退出码: $exit_code"
  restore_backup_files
  cleanup
  cleanup_local_images
  exit $exit_code
}

# 设置错误处理
trap handle_error ERR

# 运行主函数
main "$@"
