FROM docker

RUN apk add bash

RUN mkdir -p /workdir
WORKDIR /workdir

COPY docker /workdir/docker
COPY localnet-data /workdir/localnet-data
RUN chown -R nobody:nobody /workdir
ENV RECALL_NODE_USER=nobody
RUN ls -la /workdir/localnet-data/anvil/

# This is needed to expose DIND endpoints correctly from inside the localnet container
ENV LOCALNET_CLI_BIND_HOST=0.0.0.0

ENTRYPOINT ["./docker/entrypoint-localnet.sh"]
