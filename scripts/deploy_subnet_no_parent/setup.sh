#!/bin/bash

set -euo pipefail

dir=$(dirname -- "$(readlink -f -- "${BASH_SOURCE[0]}")")
source "$dir"/util.sh
IPC_FOLDER="$dir"/../..
IPC_CONFIG_FOLDER="$HOME"/.ipc

rm -rf "$IPC_CONFIG_FOLDER"
mkdir -p "$IPC_CONFIG_FOLDER"
cp "$IPC_FOLDER"/scripts/deploy_subnet_no_parent/.ipc/config.toml "$IPC_CONFIG_FOLDER"
cp "$IPC_FOLDER"/scripts/deploy_subnet_no_parent/.ipc/genesis.json "$IPC_CONFIG_FOLDER"
cp "$IPC_FOLDER"/infra/prometheus/prometheus.yaml "$IPC_CONFIG_FOLDER"

# Build ipc-cli
cd "$IPC_FOLDER"/ipc
make install

for _ in {0..2}
do
  ipc-cli wallet new --wallet-type evm 1> /dev/null
done

for i in {0..2}
do
  addr=$(jq .["$i"].address "$IPC_CONFIG_FOLDER"/evm_keystore.json | tr -d '"')
  faddr=$(ipc-cli util eth-to-f4-addr --addr "$addr" | sed -n 's/.*f4 address: //p' | tr -d ' ')
  pk=$(ipc-cli wallet pub-key --wallet-type evm --address "$addr" | xxd -r -p -c 1000000 | xbase64)
  jq --arg faddr "$faddr" '.app_state.accounts += [{"balance": "1000000000000000000", "meta": {"Account": {"owner": $faddr}}}]' "$IPC_CONFIG_FOLDER"/genesis.json > /tmp/tmp.json && mv /tmp/tmp.json "$IPC_CONFIG_FOLDER"/genesis.json
  jq --arg pk "$pk" '.app_state.validators += [{"power": "10000000000000000000", "public_key": $pk}]' "$IPC_CONFIG_FOLDER"/genesis.json > /tmp/tmp.json && mv /tmp/tmp.json "$IPC_CONFIG_FOLDER"/genesis.json
done
