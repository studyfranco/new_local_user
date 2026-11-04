FROM ghcr.io/studyfranco/new_local_user:master

USER root

COPY Cargo.toml Cargo.lock /tmp/agent-hub-build/
COPY src /tmp/agent-hub-build/src
COPY static /tmp/agent-hub-build/static
COPY run-web.sh /usr/local/bin/start-cli.sh

RUN apt-get update && \
    apt-get install -y --no-install-recommends \
        ca-certificates \
        rustc \
        cargo \
        pkg-config \
        build-essential && \
    cargo build --release --manifest-path /tmp/agent-hub-build/Cargo.toml && \
    install -m 0755 /tmp/agent-hub-build/target/release/agent-hub /usr/local/bin/agent-hub && \
    chmod +x /usr/local/bin/start-cli.sh && \
    rm -rf /tmp/agent-hub-build /root/.cargo && \
    apt-get purge -y --auto-remove rustc cargo pkg-config build-essential && \
    apt-get clean && \
    rm -rf /var/lib/apt/lists/*

USER thenewuser
WORKDIR /srv

EXPOSE 8080
