#!/bin/bash

set -euxo pipefail

if [[ ! -v IPC_FOLDER ]]; then
    IPC_FOLDER=${HOME}/ipc
else
    IPC_FOLDER=${IPC_FOLDER}
fi

# Use "dummy" subnet
subnet_id="/r314159/t410f726d2jv6uj4mpkcbgg5ndlpp3l7dd5rlcpgzkoi"

# Stop validators
cd ${IPC_FOLDER}
for i in {0..2}
do
  cargo make --makefile infra/fendermint/Makefile.toml \
      -e NODE_NAME=validator-${i} \
      -e SUBNET_ID=${subnet_id} \
      child-validator-down
done
