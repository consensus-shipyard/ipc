#!/bin/bash

set -euo pipefail

DASHES='------'
dir=$(dirname -- "$(readlink -f -- "${BASH_SOURCE[0]}")")
source "$dir"/util.sh
IPC_FOLDER="$dir"/../..
IPC_CONFIG_FOLDER="$HOME"/.ipc

# Copy configs
rm -rf "$IPC_CONFIG_FOLDER"
mkdir -p "$IPC_CONFIG_FOLDER"
cp "$IPC_FOLDER"/scripts/deploy_subnet_no_parent/.ipc/config.toml "$IPC_CONFIG_FOLDER"
cp "$IPC_FOLDER"/scripts/deploy_subnet_no_parent/.ipc/genesis.json "$IPC_CONFIG_FOLDER"
cp "$IPC_FOLDER"/infra/prometheus/prometheus.yaml "$IPC_CONFIG_FOLDER"
cp "$IPC_FOLDER"/infra/loki/loki-config.yaml "$IPC_CONFIG_FOLDER"
cp "$IPC_FOLDER"/infra/promtail/promtail-config.yaml "$IPC_CONFIG_FOLDER"
cp "$IPC_FOLDER"/infra/iroh/iroh.config.toml "$IPC_CONFIG_FOLDER"

if [[ "$SKIP_BUILD" == "" || "$SKIP_BUILD" == "false" ]]; then
  echo "$DASHES starting build for ipc cli $DASHES"
  # Build ipc-cli
  cd "$IPC_FOLDER"/ipc
  make install
else
  echo "$DASHES skpping build for ipc cli $DASHES"
fi


# use Anvil default pk values
ipc-cli wallet import --private-key 0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80 --wallet-type evm
ipc-cli wallet import --private-key 0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d --wallet-type evm
ipc-cli wallet import --private-key 0x5de4111afa1a4b94908f83103eb1f1706367c2e68ca870fc3fb9a804cdab365a --wallet-type evm

# Update genesis template with validator accounts
for i in {0..2}
do
  addr=$(jq .["$i"].address "$IPC_CONFIG_FOLDER"/evm_keystore.json | tr -d '"')
  faddr=$(ipc-cli util eth-to-f4-addr --addr "$addr" | sed -n 's/.*f4 address: //p' | tr -d ' ')
  pk=$(ipc-cli wallet pub-key --wallet-type evm --address "$addr" | xxd -r -p -c 1000000 | xbase64)
  jq --arg faddr "$faddr" '.app_state.accounts += [{"balance": "1000000000000000000000", "meta": {"Account": {"owner": $faddr}}}]' < "$IPC_CONFIG_FOLDER"/genesis.json > /tmp/tmp.json && mv /tmp/tmp.json "$IPC_CONFIG_FOLDER"/genesis.json
  jq --arg pk "$pk" '.app_state.validators += [{"power": "10000000000000000000", "public_key": $pk}]' < "$IPC_CONFIG_FOLDER"/genesis.json > /tmp/tmp.json && mv /tmp/tmp.json "$IPC_CONFIG_FOLDER"/genesis.json
done
