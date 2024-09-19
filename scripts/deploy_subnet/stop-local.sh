#!/bin/bash

set -euo pipefail

dir=$(dirname -- "$(readlink -f -- "${BASH_SOURCE[0]}")")
IPC_FOLDER=$(readlink -f -- "$dir"/../..)
IPC_CONFIG_FOLDER=${HOME}/.ipc
subnet_id="/r31337/t410f6dl55afbyjbpupdtrmedyqrnmxdmpk7rxuduafq"

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
