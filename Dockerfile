# 构建阶段
FROM rust:1.93.1-alpine AS builder

# 生产机在境内，官方源 dl-cdn.alpinelinux.org 下载会停滞（曾卡在 g++ 步骤 19 分钟），
# 构建前先切到阿里云镜像，避免每次构建都赌网络。
RUN sed -i 's#dl-cdn.alpinelinux.org#mirrors.aliyun.com#g' /etc/apk/repositories && \
    apk add --no-cache build-base openssl perl

WORKDIR /app


# 先复制 Cargo.toml 和 Cargo.lock
COPY Cargo.toml Cargo.lock ./
COPY route-macros ./route-macros
COPY route-macros-types ./route-macros-types
# 创建一个空的 src/main.rs 来缓存依赖
RUN mkdir src && \
    echo "fn main() {}" > src/main.rs && \
    cargo build --release && \
    rm -rf src

COPY . .

# RUN apt-get update && apt-get install -y musl-tools && \
    # rustup target add x86_64-unknown-linux-musl && apt-get install libssl-dev -y &&  apt-get install pkg-config -y
RUN cargo build --release

# 运行阶段
FROM alpine:latest
WORKDIR /app
COPY --from=builder /app/target/release/web-server .
# 复制版本信息文件
COPY --from=builder /app/.docker/version.json .docker/version.json
CMD ["./web-server"]
