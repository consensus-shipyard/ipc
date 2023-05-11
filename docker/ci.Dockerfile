# syntax=docker/dockerfile:1

# This file assumes that we use `make build` to build the binary,
# rather than the builder-runner pattern.

# Matching base image as the CI runner `runs-on` so the outside build is usable in the container.
FROM ubuntu:latest

RUN apt-get update && \
  apt-get install -y libstdc++6 && \
  rm -rf /var/lib/apt/lists/*

ENV FM_HOME_DIR=/fendermint
ENV HOME=$FM_HOME_DIR
WORKDIR $FM_HOME_DIR

EXPOSE 26658

ENTRYPOINT ["fendermint"]
CMD ["run"]

STOPSIGNAL SIGTERM

ENV FM_ABCI__HOST=0.0.0.0

COPY fendermint/app/config $FM_HOME_DIR/config
COPY docker/.artifacts/bundle.car $FM_HOME_DIR/bundle.car
COPY docker/.artifacts/fendermint /usr/local/bin/fendermint
