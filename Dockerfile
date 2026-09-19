FROM ghcr.io/studyfranco/docker-baseimages-debian:testing-video

LABEL maintainer="studyfranco@gmail.com"

RUN set -x \
    && apt update \
    && apt dist-upgrade -y \
    && apt autopurge -yy \
    && apt clean autoclean -y \
    && rm -rf /var/cache/* /var/lib/apt/lists/* /var/log/* /var/tmp/* /tmp/*

RUN set -x \
    && apt update \
    && DEBIAN_FRONTEND=noninteractive apt-get install -y build-essential netcat-openbsd lsof procps jq rsync zip 7zip pkg-config openssl --no-install-recommends --fix-missing \
    && apt autopurge -yy \
    && apt clean \
    && rm -rf /var/cache/* /var/lib/apt/lists/* /var/log/* /var/tmp/* /tmp/*

RUN set -x \
    && apt update \
    && DEBIAN_FRONTEND=noninteractive apt-get install -y strace tmux --no-install-recommends --fix-missing \
    && apt autopurge -yy \
    && apt clean \
    && rm -rf /var/cache/* /var/lib/apt/lists/* /var/log/* /var/tmp/* /tmp/*

RUN set -x \
    && apt update \
    && DEBIAN_FRONTEND=noninteractive apt install -y python3-venv python3-pip --no-install-recommends \
    && python3 -m pip install --break-system-packages py-spy \
    && apt clean autoclean -y \
    && rm -rf /var/cache/* /var/lib/apt/lists/* /var/log/* /var/tmp/* /tmp/* /root/.cache

RUN install -d -m 0755 /etc/apt/keyrings \
    && set -x \
    && curl -fsSL https://downloads.claude.ai/keys/claude-code.asc -o /etc/apt/keyrings/claude-code.asc \
    && echo "deb [signed-by=/etc/apt/keyrings/claude-code.asc] https://downloads.claude.ai/claude-code/apt/stable stable main" | tee /etc/apt/sources.list.d/claude-code.list \
    && apt update \
    && DEBIAN_FRONTEND=noninteractive apt-get install -y claude-code --no-install-recommends --fix-missing \
    && apt autopurge -yy \
    && apt clean \
    && rm -rf /var/cache/* /var/lib/apt/lists/* /var/log/* /var/tmp/* /tmp/*

RUN set -x \
    && useradd -ms /bin/bash thenewuser \
    && gosu nobody true

COPY init.sh /
COPY --chown=thenewuser:thenewuser run.sh run-claude.sh /home/thenewuser/
RUN chmod +x /init.sh \
    && chmod +x /home/thenewuser/run.sh \
    && chmod +x /home/thenewuser/run-claude.sh

ENV PGID="1000" \
    PUID="1000" \
    XDG_CACHE_HOME=/tmp/.cache \
    CARGO_HOME=/tmp/cargo \
    PYTHONPYCACHEPREFIX=/tmp/pycache

ENTRYPOINT [ "/init.sh" ]