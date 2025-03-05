# syntax=docker/dockerfile:1

# Builder
FROM rust:1.81.0-bookworm AS builder

RUN apt-get update && \
  apt-get install -y build-essential clang cmake protobuf-compiler && \
  rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY . .

# Mounting speeds up local builds, but it doesn't get cached between builds on CI.
# OTOH it seems like one platform build can be blocked trying to acquire a lock on the build directory,
# so for cross builds this is probably not a good idea.
ARG RUST_VERSION=1.81.0
RUN --mount=type=cache,target=target \
  --mount=type=cache,target=$RUSTUP_HOME,from=rust,source=$RUSTUP_HOME \
  --mount=type=cache,target=$CARGO_HOME,from=rust,source=$CARGO_HOME \
  rustup toolchain install ${RUST_VERSION} --no-self-update &&\
  rustup target add wasm32-unknown-unknown &&\
  cargo install --locked --root output --path fendermint/app &&\
  cargo install --locked --root output --path ipc/cli
