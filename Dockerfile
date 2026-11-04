FROM ghcr.io/studyfranco/new_local_user:master

LABEL maintainer="studyfranco@gmail.com"

USER root

RUN set -x \
    && apt update \
    && DEBIAN_FRONTEND=noninteractive apt-get install -y nodejs npm socat --no-install-recommends --fix-missing \
    && git clone --depth 1 https://github.com/formulahendry/acp-ui.git /tmp/acp-ui \
    && cd /tmp/acp-ui \
    && npm ci \
    && npm run build:web \
    && install -d -m 0755 /opt/acp-ui \
    && cp -r /tmp/acp-ui/dist-web /opt/acp-ui/dist-web \
    && git clone --depth 1 https://github.com/sjtrny/codex-web.git /opt/codex-web \
    && cd /opt/codex-web \
    && npm ci \
    && npm run build \
    && rm -rf /opt/codex-web/node_modules /opt/codex-web/.git \
    && python3 -m pip install --break-system-packages --no-cache-dir -r /opt/codex-web/requirements.txt \
    && chown -R thenewuser:thenewuser /opt/acp-ui /opt/codex-web \
    && chmod -R a+rX /opt/acp-ui /opt/codex-web \
    && apt-get purge -y nodejs npm \
    && apt autopurge -yy \
    && apt clean autoclean -y \
    && rm -rf /var/cache/* /var/lib/apt/lists/* /var/log/* /var/tmp/* /tmp/* /root/.npm /root/.cache

COPY --chown=thenewuser:thenewuser run-web.sh /usr/local/bin/start-cli.sh
RUN chmod 0755 /usr/local/bin/start-cli.sh

ENV ACP_UI_PORT="3000" \
    CODEX_RUNTIME_DIR="/run/codex" \
    CODEX_WORKSPACE_ROOT="/srv" \
    CODEX_DEFAULT_CWD="/srv" \
    CODEX_UPLOAD_DIR="/tmp/uploads"

EXPOSE 3000 8000

USER thenewuser
WORKDIR /tmp
