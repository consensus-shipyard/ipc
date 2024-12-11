#!/bin/bash

set -euo pipefail

dir=$(dirname -- "$(readlink -f -- "${BASH_SOURCE[0]}")")
IPC_FOLDER=$(readlink -f -- "$dir"/../..)
subnet_id="/r31337/t410f6gbdxrbehnaeeo4mrq7wc5hgq6smnefys4qanwi"

cd "$IPC_FOLDER"
cargo make --makefile infra/fendermint/Makefile.toml \
    -e NODE_NAME=relayer \
    relayer-destroy

cd "$IPC_FOLDER"
cargo make --makefile infra/fendermint/Makefile.toml \
    -e NODE_NAME=anvil \
    anvil-destroy

cd "$IPC_FOLDER"
cargo make --makefile infra/fendermint/Makefile.toml \
    -e NODE_NAME=prometheus \
    prometheus-destroy

cd "$IPC_FOLDER"
cargo make --makefile infra/fendermint/Makefile.toml \
    -e NODE_NAME=grafana \
    grafana-destroy

cd "$IPC_FOLDER"
cargo make --makefile infra/fendermint/Makefile.toml \
    -e NODE_NAME=loki \
    loki-destroy

for i in {0..2}
  do
    cargo make --makefile infra/fendermint/Makefile.toml \
        -e NODE_NAME=validator-"$i" \
        -e SUBNET_ID="$subnet_id" \
        child-validator-down
  done
