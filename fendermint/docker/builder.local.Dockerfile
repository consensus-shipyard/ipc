# syntax=docker/dockerfile:1

# Builder
FROM rust:bookworm AS builder

RUN apt-get update && \
  apt-get install -y build-essential clang cmake protobuf-compiler curl bash && \
  rm -rf /var/lib/apt/lists/*

WORKDIR /app

RUN chsh -s $(which bash) $(whoami)
RUN echo "x$SHELL"

# install foundry
RUN curl -L https://foundry.paradigm.xyz | bash -
ENV PATH=${PATH}:~/.cargo/bin:~/.foundry/bin
RUN ls -al ~/.foundry/bin && ~/.foundry/bin/foundryup

# install nvm
RUN curl -sSL -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.40.3/install.sh | bash -

# install pnpm
ENV ENV="~/.bashrc"
RUN echo $SHELL; curl -sSL https://get.pnpm.io/install.sh | ENV="$ENV" SHELL="$(which sh)" sh -
ENV PATH=${PATH}:~/.local/share/pnpm

COPY . .

# Mounting speeds up local builds, but it doesn't get cached between builds on CI.
# OTOH it seems like one platform build can be blocked trying to acquire a lock on the build directory,
# so for cross builds this is probably not a good idea.
RUN --mount=type=cache,target=target \
  --mount=type=cache,target=$RUSTUP_HOME,from=rust,source=$RUSTUP_HOME \
  --mount=type=cache,target=$CARGO_HOME,from=rust,source=$CARGO_HOME \
  cargo install --locked --root output --path fendermint/app &&\
  cargo install --locked --root output --path ipc/cli
