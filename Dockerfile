# Build stage (converged with simply_ip_vault / simply_ip_sync)
FROM rust:slim AS builder

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY src src
COPY static static

RUN set -x \
    && apt-get update \
    && DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends \
        build-essential \
        ca-certificates \
        pkg-config \
    && cargo build --release \
    && apt-get clean autoclean -y \
    && rm -rf /var/cache/* /var/lib/apt/lists/* /var/log/* /var/tmp/* /tmp/*

# Runtime stage (inherits rootless base image)
FROM ghcr.io/studyfranco/new_local_user:master

USER root
COPY --from=builder /app/target/release/agent-hub /usr/local/bin/agent-hub
COPY run-web.sh /usr/local/bin/start-cli.sh
RUN chmod +x /usr/local/bin/agent-hub /usr/local/bin/start-cli.sh

USER thenewuser
WORKDIR /srv

EXPOSE 8080

HEALTHCHECK --interval=30s --timeout=5s --start-period=15s --retries=3 \
    CMD curl -fsS http://127.0.0.1:8080/ready || exit 1
