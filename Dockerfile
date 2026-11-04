FROM ghcr.io/studyfranco/new_local_user:master

LABEL maintainer="studyfranco@gmail.com"

USER root

RUN set -x \
    && install -d -m 0755 -o thenewuser -g thenewuser /home/thenewuser/.gemini \
    && su - thenewuser -c "curl -fsSL https://antigravity.google/cli/install.sh | bash" \
    && (cp -Lf /home/thenewuser/.local/bin/agy /usr/local/bin/agy 2>/dev/null || cp -Lf /home/thenewuser/.gemini/bin/agy /usr/local/bin/agy 2>/dev/null || true) \
    && chmod 0755 /usr/local/bin/agy 2>/dev/null || true \
    && rm -rf /var/cache/* /var/lib/apt/lists/* /var/log/* /var/tmp/* /tmp/*

COPY --chown=thenewuser:thenewuser run-antigravity.sh /usr/local/bin/start-cli.sh
RUN chmod 0755 /usr/local/bin/start-cli.sh

ENV PATH="/home/thenewuser/.local/bin:/home/thenewuser/.gemini/bin:${PATH}"

USER thenewuser
WORKDIR /tmp
