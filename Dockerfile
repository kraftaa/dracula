FROM rust:bullseye AS builder

# Build workspace
WORKDIR /dracula
COPY . /dracula

RUN rustup install 1.69.0; \
    rustup default 1.69.0; \
    cargo build --release

FROM debian:bullseye-slim AS dracula

ARG POSTGRESQL_VERSION=15.2

RUN apt-get update && apt-get install ca-certificates \
      bash \
      curl git time build-essential libc6-dev libpq-dev libssl-dev linux-libc-dev brotli libbrotli-dev \
      pkgconf sudo cmake xutils-dev zlib1g-dev -y && \
      rm -rf /var/cache/apt/* \

RUN echo "Building libpq" && \
    cd /tmp && \
    curl -LO "https://ftp.postgresql.org/pub/source/v$POSTGRESQL_VERSION/postgresql-$POSTGRESQL_VERSION.tar.gz" && \
    tar xzf "postgresql-$POSTGRESQL_VERSION.tar.gz" && cd "postgresql-$POSTGRESQL_VERSION" && \
    CC=gcc CPPFLAGS=-I/usr/local/include LDFLAGS=-L/usr/local/lib ./configure --with-openssl --without-readline --prefix=/usr/local && \
    cd src/interfaces/libpq && make all-static-lib && sudo make install-lib-static && \
    cd ../../bin/pg_config && make && sudo make install && \
    rm -r /tmp/*

COPY --from=builder /dracula/target/release/dracula /usr/local/bin/dracula