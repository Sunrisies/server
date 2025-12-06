# 构建阶段
FROM rust:1.86.0-alpine AS builder

RUN apk add --no-cache build-base openssl perl

WORKDIR /app


# 先复制 Cargo.toml 和 Cargo.lock
COPY Cargo.toml Cargo.lock ./
COPY route-macros ./route-macros
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
COPY --from=builder /app/.env .
# 复制版本信息文件
COPY --from=builder /app/.docker/version.json .docker/version.json
CMD ["./web-server"]
