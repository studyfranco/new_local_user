FROM ghcr.io/studyfranco/new_local_user:goose

LABEL maintainer="studyfranco@gmail.com"

USER root

RUN set -x \
    && apt update \
    && DEBIAN_FRONTEND=noninteractive apt install -y atomicparsley --no-install-recommends \
    && python3 -m pip install --break-system-packages mnamer \
    && apt clean autoclean -y \
    && rm -rf /var/cache/* /var/lib/apt/lists/* /var/log/* /var/tmp/* /tmp/* /root/.cache

USER thenewuser
WORKDIR /tmp
