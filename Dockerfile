FROM ghcr.io/studyfranco/new_local_user:goose

LABEL maintainer="studyfranco@gmail.com"

USER root

RUN set -x \
    && mkdir -p /var/cache/apt/archives/partial /var/lib/apt/lists/partial && apt update \
    && DEBIAN_FRONTEND=noninteractive apt-get install -y \
        rustc cargo rustfmt rust-clippy rust-src rust-analyzer \
        clang lld lldb gdb cmake libssl-dev strace \
        python3-dev pipx \
        --no-install-recommends --fix-missing \
    && python3 -m pip install --break-system-packages uv ruff black mypy pytest ipython py-spy \
    && apt autopurge -yy \
    && apt clean autoclean -y \
    && rm -rf /var/lib/apt/lists/* /var/log/* /var/tmp/* /tmp/* /root/.cache

USER thenewuser
WORKDIR /tmp
