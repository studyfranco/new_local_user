FROM ghcr.io/studyfranco/new_local_user:goose

LABEL maintainer="studyfranco@gmail.com"

USER root

RUN set -x \
    && apt update \
    && DEBIAN_FRONTEND=noninteractive apt-get install -y \
        rustc cargo rustfmt clippy rust-analyzer \
        clang lld lldb gdb cmake libssl-dev \
        python3-dev pipx \
        --no-install-recommends --fix-missing \
    && python3 -m pip install --break-system-packages uv ruff black mypy pytest ipython \
    && apt autopurge -yy \
    && apt clean autoclean -y \
    && rm -rf /var/cache/* /var/lib/apt/lists/* /var/log/* /var/tmp/* /tmp/* /root/.cache

USER thenewuser
WORKDIR /tmp
