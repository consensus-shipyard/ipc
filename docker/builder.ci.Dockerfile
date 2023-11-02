# syntax=docker/dockerfile:1

# https://www.docker.com/blog/faster-multi-platform-builds-dockerfile-cross-compilation-guide/
# https://www.docker.com/blog/cross-compiling-rust-code-for-multiple-architectures/
# https://www.docker.com/blog/multi-arch-build-and-images-the-simple-way/
# https://github.com/cross-rs/cross/wiki/Recipes#openssl

# Using `ubuntu` here because when I try `rust:bookworm` like in `builder.local.Dockerfile` then
# even though I add `aarch64` rustup target as a RUN step, it can't compile `core` later on
# unless that step is repeated in the same command as the cargo build. That doesn't happen
# with the `ubuntu` base and Rust installed.
FROM --platform=$BUILDPLATFORM ubuntu:latest as builder

RUN apt-get update && \
  apt-get install -y build-essential clang cmake protobuf-compiler curl \
  openssl libssl-dev pkg-config

# Get Rust
RUN curl https://sh.rustup.rs -sSf | sh -s -- --default-toolchain stable -y
ENV PATH="/root/.cargo/bin:${PATH}"

ENV CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc \
  CC_aarch64_unknown_linux_gnu=aarch64-linux-gnu-gcc \
  CXX_aarch64_unknown_linux_gnu=aarch64-linux-gnu-g++

WORKDIR /app

# Update the version here if our `rust-toolchain.toml` would cause something new to be fetched every time.
ARG RUST_VERSION=1.73
RUN rustup install ${RUST_VERSION} && rustup target add wasm-unknown-unknown

# Defined here so anything above it can be cached as a common dependency.
ARG TARGETARCH

# Only installing MacOS specific libraries if necessary.
RUN if [ "${TARGETARCH}" = "arm64" ]; then \
  apt-get install -y g++-aarch64-linux-gnu libc6-dev-arm64-cross; \
  rustup target add aarch64-unknown-linux-gnu; \
  rustup toolchain install stable-aarch64-unknown-linux-gnu; \
  fi

COPY . .

# On CI we use `docker buildx` with multiple `--platform` arguments, and `--cache-from=type=gha` to cache the layers.
# If we used `--mount=type=cache` here then it looks like the different platforms would be mounted at the same place
# and then one of them can get blocked trying to acquire a lock on the build directory.

RUN set -eux; \
  case "${TARGETARCH}" in \
  amd64) ARCH='x86_64'  ;; \
  arm64) ARCH='aarch64' ;; \
  *) echo >&2 "unsupported architecture: ${TARGETARCH}"; exit 1 ;; \
  esac; \
  rustup show ; \
  cargo install --root output --path fendermint/app --target ${ARCH}-unknown-linux-gnu
