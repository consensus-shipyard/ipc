# syntax=docker/dockerfile:1

# Builder
FROM rust:1.68 as builder

RUN <<EOF
    set -e

    apt-get update
    apt-get install -y build-essential clang cmake

    rm -rf /var/lib/apt/lists/*
EOF

WORKDIR /app

COPY . .

RUN --mount=type=cache,target=$RUSTUP_HOME,from=rust,source=$RUSTUP_HOME \
  --mount=type=cache,target=$CARGO_HOME,from=rust,source=$CARGO_HOME \
  --mount=type=cache,target=target \
  cargo install --root output --path fendermint/app


# Runner
FROM debian:bullseye-slim

WORKDIR /app
ENV HOME=/app

ENV FM_ABCI__HOST=0.0.0.0

ARG BUILTIN_ACTORS_BUNDLE
COPY ${BUILTIN_ACTORS_BUNDLE} $HOME/.fendermint/bundle.car
COPY --from=builder /app/fendermint/app/config $HOME/.fendermint/config
COPY --from=builder /app/output/bin/fendermint /usr/local/bin/fendermint

ENTRYPOINT ["fendermint"]
CMD ["run"]

EXPOSE 26658
