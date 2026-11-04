FROM ghcr.io/studyfranco/new_local_user:master

LABEL maintainer="studyfranco@gmail.com"

USER root

RUN set -x \
    && apt update \
    && DEBIAN_FRONTEND=noninteractive apt-get install -y bzip2 libxcb1 dbus libglib2.0-0 --no-install-recommends --fix-missing \
    && curl -fsSL https://github.com/aaif-goose/goose/releases/download/stable/download_cli.sh | CONFIGURE=false GOOSE_BIN_DIR=/usr/local/bin bash \
    && chmod 0755 /usr/local/bin/goose \
    && install -d -m 0755 -o thenewuser -g thenewuser \
        /home/thenewuser/.config/goose \
        /home/thenewuser/.local/share/goose \
        /home/thenewuser/.local/state/goose \
    && apt autopurge -yy \
    && apt clean \
    && rm -rf /var/cache/* /var/lib/apt/lists/* /var/log/* /var/tmp/* /tmp/*

COPY --chown=thenewuser:thenewuser run-goose.sh /usr/local/bin/start-cli.sh
RUN chmod 0755 /usr/local/bin/start-cli.sh

ENV GOOSE_MODE="auto" \
    GOOSE_HOST="0.0.0.0" \
    GOOSE_PORT="3284"

EXPOSE 3284

USER thenewuser
WORKDIR /tmp
