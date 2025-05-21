# This image is used to set up a subnet.
# It contains the required ipc-cli and fendermint versions and the foundry tools.

ARG fendermint_image

FROM $fendermint_image

# Install foundry
RUN set -ex; \
  arch=$(uname -m | sed -e s/aarch64/arm64/ -e s/x86_64/amd64/); \
  curl -Lo /tmp/tt.tgz https://github.com/foundry-rs/foundry/releases/download/stable/foundry_stable_linux_${arch}.tar.gz && \
  tar xvf /tmp/tt.tgz -C /bin && \
  rm /tmp/tt.tgz


ARG NODE_VERSION=22.14.0

# Install build tools
RUN set -ex; \
  apt update; \
  apt install -y git make python3 build-essential; \
  curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.40.1/install.sh | bash;

RUN set -ex; \
  \. $HOME/.nvm/nvm.sh; \
  nvm install $NODE_VERSION; \
  nvm use v$NODE_VERSION; \
  nvm alias default v$NODE_VERSION;

ENV PATH="/fendermint/.nvm/versions/node/v${NODE_VERSION}/bin/:${PATH}"

RUN npm install -g pnpm

ENTRYPOINT ["/bin/bash"]
