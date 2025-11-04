FROM node:23-slim AS frontend
RUN npm install -g pnpm
WORKDIR /app

COPY web/package.json web/pnpm-lock.yaml ./web/
RUN mkdir -p web && cd web && pnpm install --frozen-lockfile
COPY web/ ./web/
RUN cd web && pnpm run build:web

FROM rust:1.89 AS builder
WORKDIR /app
COPY . .
RUN cargo build --release --bin pesa-axum

FROM debian:12-slim
WORKDIR /app

RUN apt-get update && apt-get install -y libssl3 zlib1g libzstd1 && rm -rf /var/lib/apt/lists/* \
    && useradd -ms /bin/bash appuser

USER appuser
WORKDIR /home/appuser/app

COPY --from=frontend /app/web/build ./webroot
COPY --from=builder /app/target/release/pesa-axum .

EXPOSE 3000

CMD ["./pesa-axum", "--webroot", "./webroot"]
