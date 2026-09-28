# ── Builder ──────────────────────────────────────────
# Not: crates.io bağımlılıkları sürekli MSRV yükselttiği için builder
# bilerek "latest stable" kullanır (2026 itibarıyla edition2024 + rustc 1.88+ gerekir).
FROM rust:bookworm AS builder
WORKDIR /app
COPY Cargo.toml ./
COPY src ./src
COPY migrations ./migrations
# sqlx runtime sorgular kullanır; derleme için DB gerekmez.
ENV SQLX_OFFLINE=true
RUN cargo build --release && \
    cp target/release/todo-backend /todo-backend

# ── Runtime ──────────────────────────────────────────
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates wget && \
    rm -rf /var/lib/apt/lists/* && \
    useradd -m -u 10001 appuser
WORKDIR /app
COPY --from=builder /todo-backend /usr/local/bin/todo-backend
# migrate! macro gömülü; migrations klasörü runtime'da gerekmez.
USER appuser
EXPOSE 8080
ENV APP_HOST=0.0.0.0 APP_PORT=8080 RUST_LOG=info
HEALTHCHECK --interval=15s --timeout=5s --retries=5 \
  CMD wget -qO- http://127.0.0.1:${APP_PORT:-8080}/health | grep -q ok || exit 1
ENTRYPOINT ["todo-backend"]
