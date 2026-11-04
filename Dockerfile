FROM ghcr.io/studyfranco/docker-baseimages-debian:testing-video

LABEL maintainer="studyfranco@gmail.com"

USER root

RUN set -x \
    && apt update \
    && apt dist-upgrade -y \
    && apt autopurge -yy \
    && apt clean autoclean -y \
    && rm -rf /var/cache/* /var/lib/apt/lists/* /var/log/* /var/tmp/* /tmp/*

RUN set -x \
    && apt update \
    && DEBIAN_FRONTEND=noninteractive apt-get install -y build-essential netcat-openbsd lsof procps jq rsync zip 7zip pkg-config openssl ca-certificates curl git --no-install-recommends --fix-missing \
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

RUN set -x \
    && groupadd -g 1000 thenewuser \
    && useradd -u 1000 -g 1000 -ms /bin/bash thenewuser \
    && install -d -m 0755 -o thenewuser -g thenewuser \
        /config \
        /home/thenewuser/.config \
        /home/thenewuser/.local/bin \
        /home/thenewuser/.local/share \
        /home/thenewuser/.local/state

COPY --chown=thenewuser:thenewuser run.sh /usr/local/bin/run.sh
RUN chmod 0755 /usr/local/bin/run.sh

ENV HOME="/home/thenewuser" \
    USER="thenewuser" \
    PATH="/home/thenewuser/.local/bin:/usr/local/bin:/usr/bin:/bin" \
    TMPDIR="/tmp" \
    TMUX_TMPDIR="/tmp" \
    XDG_RUNTIME_DIR="/tmp/runtime-thenewuser" \
    XDG_CACHE_HOME="/tmp/.cache" \
    XDG_CONFIG_HOME="/home/thenewuser/.config" \
    XDG_DATA_HOME="/home/thenewuser/.local/share" \
    XDG_STATE_HOME="/home/thenewuser/.local/state" \
    CARGO_HOME="/tmp/cargo" \
    PYTHONPYCACHEPREFIX="/tmp/pycache" \
    PIP_CACHE_DIR="/tmp/.cache/pip" \
    NPM_CONFIG_CACHE="/tmp/.cache/npm"

USER thenewuser
WORKDIR /tmp

ENTRYPOINT [ "/usr/local/bin/run.sh" ]
