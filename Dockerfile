FROM alpine:3.21 AS builder

RUN apk add --no-cache curl


WORKDIR /app
