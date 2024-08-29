FROM ubuntu:24.04 AS base
LABEL maintainer="Jun Kurihara"

SHELL ["/bin/sh", "-x", "-c"]
ENV SERIAL=2

########################################
FROM base AS builder

ARG CFLAGS=-Ofast
ARG BUILD_DEPS="curl make ca-certificates build-essential pkg-config libssl-dev"

WORKDIR /tmp

COPY . /tmp/

ARG RUSTFLAGS="-C link-arg=-s"

RUN update-ca-certificates 2> /dev/null || true

RUN apt update && apt install -qy --no-install-recommends $BUILD_DEPS && \
    curl -sSf https://sh.rustup.rs | bash -s -- -y --default-toolchain stable && \
    export PATH="$HOME/.cargo/bin:$PATH" && \
    echo "Building token server from source" && \
    cargo build --release --package=rust-token-server && \
    strip --strip-all /tmp/target/release/rust-token-server

########################################

FROM base AS runner
LABEL maintainer="Jun Kurihara"

ARG RUNTIME_DEPS="logrotate ca-certificates gosu libsqlite3-dev"

RUN apt update && \
    apt install -qy --no-install-recommends $RUNTIME_DEPS && \
    apt -qy clean && \
    rm -fr /tmp/* /var/tmp/* /var/cache/apt/* /var/lib/apt/lists/* /var/log/apt/* /var/log/*.log && \
    mkdir -p /opt/token-server/sbin

COPY --from=builder /tmp/target/release/rust-token-server /opt/token-server/sbin/
COPY server/docker-bin/entrypoint.sh /
COPY server/docker-bin/run.sh /

RUN chmod 755 /entrypoint.sh &&\
    chmod 755 /run.sh

EXPOSE 8000

CMD ["/entrypoint.sh"]

ENTRYPOINT ["/entrypoint.sh"]
