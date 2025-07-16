# syntax=docker/dockerfile:1
FROM ghcr.io/foundry-rs/foundry:v1.2.3 AS foundry-provider

# Builder
FROM rust:bookworm AS builder

# copy the right ones over
COPY --from=foundry-provider /usr/local/bin/forge /usr/local/bin/
COPY --from=foundry-provider /usr/local/bin/cast /usr/local/bin/
COPY --from=foundry-provider /usr/local/bin/chisel /usr/local/bin/
COPY --from=foundry-provider /usr/local/bin/anvil /usr/local/bin/

RUN apt-get update && \
  apt-get install -y build-essential clang cmake protobuf-compiler curl bash && \
  rm -rf /var/lib/apt/lists/*

WORKDIR /app

RUN chsh -s $(which bash) $(whoami)
RUN echo "x$SHELL"

# install nvm <https://github.com/nvm-sh/nvm?tab=readme-ov-file#installing-and-updating>
RUN curl -sSL -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.40.3/install.sh | bash - && nvm --version
# install node.js 20
RUN nvm install 20 && npm --version
# install pnpm <https://pnpm.io/installation#using-npm>
RUN npm install -g pnpm@latest-10 && pnpm --version

RUN anvil --version && forge --version && cast --version && chisel --version

COPY . .

# Mounting speeds up local builds, but it doesn't get cached between builds on CI.
# OTOH it seems like one platform build can be blocked trying to acquire a lock on the build directory,
# so for cross builds this is probably not a good idea.
RUN --mount=type=cache,target=target \
  --mount=type=cache,target=$RUSTUP_HOME,from=rust,source=$RUSTUP_HOME \
  --mount=type=cache,target=$CARGO_HOME,from=rust,source=$CARGO_HOME \
  cargo install --locked --root output --path fendermint/app &&\
  cargo install --locked --root output --path ipc/cli
