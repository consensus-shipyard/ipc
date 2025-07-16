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

# changes the users shell, so `podman run -it` will use `bash`
RUN chsh -s $(which bash) $(whoami)

# SHELL ["/bin/bash -c"]would do the same, but it only works for docker format containers

RUN anvil --version && forge --version && cast --version && chisel --version

# install nvm <https://github.com/nvm-sh/nvm?tab=readme-ov-file#installing-and-updating>
RUN curl -sSL -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.40.3/install.sh | bash -
# note: since nvm requires bash, and .bashrc contains the preload of `nvm`, we run a few
# things in `/bin/bash` rather than the efault `RUN ..` environment, which is a limited
# shell implementation, and does not source `.bashrc`.

# install node.js 20
ENV NVM_DIR /root/.nvm
ENV NODE_VERSION 20.19.4
RUN /bin/bash -c "source ${NVM_DIR}/nvm.sh && nvm --version && nvm install ${NODE_VERSION} && nvm use --delete-prefix $NODE_VERSION"
ENV NODE_PATH ${NVM_DIR}/versions/node/${NODE_VERSION}/bin
ENV PATH ${NODE_PATH}:${PATH}
RUN echo "PATH=${NVM_DIR}/versions/node/${NODE_VERSION}/bin:${PATH} >> ~/.bashrc"
RUN /bin/bash -c "source ${NVM_DIR}/nvm.sh && npm --version"

# install pnpm <https://pnpm.io/installation#using-npm>
RUN /bin/bash -c "source ${NVM_DIR}/nvm.sh && npm install -g pnpm@latest-10 && pnpm --version"


COPY . .

# Mounting speeds up local builds, but it doesn't get cached between builds on CI.
# OTOH it seems like one platform build can be blocked trying to acquire a lock on the build directory,
# so for cross builds this is probably not a good idea.
RUN --mount=type=cache,target=target \
  --mount=type=cache,target=$RUSTUP_HOME,from=rust,source=$RUSTUP_HOME \
  --mount=type=cache,target=$CARGO_HOME,from=rust,source=$CARGO_HOME \
  /bin/bash -c "source ${NVM_DIR}/nvm.sh && cargo install --locked --root output --path fendermint/app" &&\
  /bin/bash -c "source ${NVM_DIR}/nvm.sh && cargo install --locked --root output --path ipc/cli"
