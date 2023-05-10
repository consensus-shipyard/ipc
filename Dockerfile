# syntax=docker/dockerfile:1

# Builder
FROM rust:1.68 as builder

RUN apt-get update && \
  apt-get install -y build-essential clang cmake && \
  rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY . .

RUN --mount=type=cache,target=$RUSTUP_HOME,from=rust,source=$RUSTUP_HOME \
  --mount=type=cache,target=$CARGO_HOME,from=rust,source=$CARGO_HOME \
  --mount=type=cache,target=target \
  cargo install --root output --path fendermint/app


# Runner
FROM debian:bullseye-slim

ENV FM_HOME_DIR=/fendermint
ENV HOME=$FM_HOME_DIR
WORKDIR $FM_HOME_DIR

EXPOSE 26658

ENTRYPOINT ["fendermint"]
CMD ["run"]

STOPSIGNAL SIGTERM

ENV FM_ABCI__HOST=0.0.0.0

ARG BUILTIN_ACTORS_BUNDLE
COPY ${BUILTIN_ACTORS_BUNDLE} $FM_HOME_DIR/bundle.car
COPY --from=builder /app/fendermint/app/config $FM_HOME_DIR/config
COPY --from=builder /app/output/bin/fendermint /usr/local/bin/fendermint
