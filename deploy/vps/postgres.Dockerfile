ARG AGE_IMAGE=apache/age:release_PG16_1.6.0

FROM ${AGE_IMAGE}

ARG PGVECTOR_VERSION=v0.8.0

USER root

RUN apt-get update \
 && apt-get install -y --no-install-recommends \
      ca-certificates \
      git \
      build-essential \
      postgresql-server-dev-16 \
 && git clone --depth 1 --branch "${PGVECTOR_VERSION}" https://github.com/pgvector/pgvector.git /tmp/pgvector \
 && make -C /tmp/pgvector OPTFLAGS="" \
 && make -C /tmp/pgvector install OPTFLAGS="" \
 && rm -rf /tmp/pgvector /var/lib/apt/lists/*
