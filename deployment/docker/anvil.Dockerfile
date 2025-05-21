FROM ubuntu:24.04

RUN set -x; \
  arch=$(uname -m | sed -e s/aarch64/arm64/ -e s/x86_64/amd64/); \
  apt update && apt install -y curl && \
  curl -Lo /tmp/tt.tgz https://github.com/foundry-rs/foundry/releases/download/stable/foundry_stable_linux_${arch}.tar.gz && \
  tar xvf /tmp/tt.tgz -C /usr/bin && \
  rm /tmp/tt.tgz

RUN mkdir -p /workdir
WORKDIR /workdir

ENTRYPOINT ["anvil", "--host", "0.0.0.0", "--state", "/workdir/state"]
