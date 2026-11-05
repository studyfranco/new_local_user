FROM ghcr.io/studyfranco/new_local_user:master

LABEL maintainer="studyfranco@gmail.com"

USER root

RUN install -d -m 0755 /etc/apt/keyrings \
    && set -x \
    && curl -fsSL https://downloads.claude.ai/keys/claude-code.asc -o /etc/apt/keyrings/claude-code.asc \
    && echo "deb [signed-by=/etc/apt/keyrings/claude-code.asc] https://downloads.claude.ai/claude-code/apt/stable stable main" | tee /etc/apt/sources.list.d/claude-code.list \
    && mkdir -p /var/cache/apt/archives/partial /var/lib/apt/lists/partial && apt update \
    && DEBIAN_FRONTEND=noninteractive apt-get install -y claude-code --no-install-recommends --fix-missing \
    && install -d -m 0755 -o thenewuser -g thenewuser /home/thenewuser/.claude \
    && apt autopurge -yy \
    && apt clean \
    && rm -rf /var/lib/apt/lists/* /var/log/* /var/tmp/* /tmp/*

COPY --chown=thenewuser:thenewuser run-claude.sh /usr/local/bin/start-cli.sh
RUN chmod 0755 /usr/local/bin/start-cli.sh

ENV remote_name="claude_code" \
    CLAUDE_CONFIG_DIR="/home/thenewuser/.claude"

USER thenewuser
WORKDIR /tmp
