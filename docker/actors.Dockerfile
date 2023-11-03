# syntax=docker/dockerfile:1

# https://docs.docker.com/engine/reference/commandline/build/#output

# Builder
FROM rust:bookworm as actor-builder

RUN rustup target add wasm32-unknown-unknown

WORKDIR /app

ARG ACTORS_REPO=https://github.com/filecoin-project/builtin-actors.git
ARG ACTORS_TAG=v11.0.0

RUN git clone ${ACTORS_REPO} . && git checkout ${ACTORS_TAG}
RUN cargo run --release -- -o output/bundle.car

# Exporter
FROM scratch AS actor-exporter
COPY --from=actor-builder /app/output/bundle.car /
