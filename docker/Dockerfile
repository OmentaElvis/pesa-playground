FROM node:23-slim AS frontend
RUN npm install -g pnpm
WORKDIR /app

COPY web/package.json web/pnpm-lock.yaml ./web/
RUN mkdir -p web && cd web && pnpm install --frozen-lockfile
COPY web/ ./web/
RUN cd web && pnpm run build:web

FROM rust:1.89 AS builder
WORKDIR /app

# Copy manifests for the ENTIRE workspace so cargo can resolve it.
COPY Cargo.toml Cargo.lock ./
COPY crates/pesa-axum/Cargo.toml ./crates/pesa-axum/
COPY crates/pesa-core/Cargo.toml ./crates/pesa-core/
COPY crates/pesa-macros/Cargo.toml ./crates/pesa-macros/

# still copy tauri but we will not be building it.
COPY crates/pesa-tauri/Cargo.toml ./crates/pesa-tauri/

# Create dummy source files for ALL workspace members.
RUN mkdir -p crates/pesa-axum/src && echo "fn main() {}" > crates/pesa-axum/src/main.rs
RUN mkdir -p crates/pesa-core/src && echo "pub fn lib() {}" > crates/pesa-core/src/lib.rs
RUN mkdir -p crates/pesa-macros/src && touch crates/pesa-macros/src/lib.rs
RUN mkdir -p crates/pesa-tauri/src && echo "fn main() {}" > crates/pesa-tauri/src/main.rs && echo "pub fn lib() {}" > crates/pesa-tauri/src/lib.rs

# Build dependencies for ONLY the pesa-axum binary.
RUN cargo build --release --bin pesa-axum

# Now copy the actual application source code for the crates we need.
COPY crates/pesa-axum/src ./crates/pesa-axum/src
COPY crates/pesa-core/src ./crates/pesa-core/src
COPY crates/pesa-macros/src ./crates/pesa-macros/src

# Build the specific binary again. This will be fast as deps are cached.
# We use touch to ensure the final crate gets recompiled with its new source.
RUN touch crates/pesa-core/src/lib.rs crates/pesa-macros/src/lib.rs crates/pesa-axum/src/main.rs && cargo build --release --bin pesa-axum

FROM debian:12-slim
WORKDIR /app

RUN apt-get update && apt-get install -y libssl3 zlib1g libzstd1 && rm -rf /var/lib/apt/lists/* \
    && useradd -ms /bin/bash appuser

USER appuser
WORKDIR /home/appuser/app

COPY --from=frontend /app/web/build ./webroot
COPY --from=builder /app/target/release/pesa-axum .

ENV APP_PORT=3000
ENV RUST_LOG_COLOR=always

# Expose the port the application will run on
EXPOSE ${APP_PORT}
STOPSIGNAL SIGINT

ENV TERM=xterm-256color

# The --webroot flag tells the binary where to find the static frontend files
ENTRYPOINT ["/bin/sh", "-c", "exec ./pesa-axum --webroot ./webroot --port ${APP_PORT}"]
