FROM --platform=$BUILDPLATFORM ubuntu:jammy

ENV FM_HOME_DIR=/fendermint

COPY fendermint/builtin-actors/output/bundle.car $FM_HOME_DIR/bundle.car
COPY contracts/out $FM_HOME_DIR/contracts
COPY fendermint/docker/docker-entry.sh /usr/local/bin/docker-entry.sh
COPY fendermint/actors/output/custom_actors_bundle.car  $FM_HOME_DIR/custom_actors_bundle.car
COPY fendermint/app/config $FM_HOME_DIR/config
COPY binary/fendermint /usr/local/bin/fendermint
COPY binary/ipc-cli /usr/local/bin/ipc-cli