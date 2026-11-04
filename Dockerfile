FROM ghcr.io/studyfranco/new_local_user:master

LABEL maintainer="studyfranco@gmail.com"

USER root

RUN set -x \
    && install -d -m 0755 -o thenewuser -g thenewuser /opt/codex /run/codex /home/thenewuser/.codex \
    && curl -fsSL https://chatgpt.com/codex/install.sh | CODEX_NON_INTERACTIVE=1 CODEX_HOME=/opt/codex CODEX_INSTALL_DIR=/usr/local/bin sh \
    && chmod -R a+rX /opt/codex \
    && rm -rf /var/cache/* /var/lib/apt/lists/* /var/log/* /var/tmp/* /tmp/*

COPY --chown=thenewuser:thenewuser run-codex.sh /usr/local/bin/start-cli.sh
RUN chmod 0755 /usr/local/bin/start-cli.sh

ENV CODEX_HOME="/home/thenewuser/.codex" \
    CODEX_HOST="0.0.0.0" \
    CODEX_PORT="4500"

EXPOSE 4500

USER thenewuser
WORKDIR /tmp
