FROM ghcr.io/studyfranco/new_local_user:antigravity

LABEL maintainer="studyfranco@gmail.com"

USER root

RUN set -x \
    && mkdir -p /var/cache/apt/archives/partial /var/lib/apt/lists/partial && apt update \
    && DEBIAN_FRONTEND=noninteractive apt-get install -y shntool flac wavpack beets picard xvfb dos2unix recode fdupes coreutils --no-install-recommends --fix-missing \
    && apt autopurge -yy \
    && apt clean \
    && rm -rf /var/lib/apt/lists/* /var/log/* /var/tmp/* /tmp/*

USER thenewuser
WORKDIR /tmp
