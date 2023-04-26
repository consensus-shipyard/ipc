#!/usr/bin/env bash

# Based on https://github.com/consensus-shipyard/lotus/blob/spacenet/scripts/ipc/src/root-single-validator.sh

set -e

eudico wait-api --timeout=300s
sleep 5

# echo "[*] Generate a new wallet for validator and make default address"
# eudico wallet set-default `eudico wallet new`
# echo "[*] Importing wallet with funds in root"

# Do not fail if the keys have already been imported,
# which would be the case if this container was restarted.
set +e

echo "[*] Importing wallet key"
eudico wallet import --as-default --format=json-lotus  /wallet.key

echo "[*] Initializing validator config"
eudico mir validator config init

validator_addr=`eudico mir validator config validator-addr | grep -vE '(/ip6/)' | grep -v "127.0.0.1"  | grep -E '/tcp/1347'`
echo "[*] Addign validator with address $validator_addr"
eudico mir validator config add-validator $validator_addr

set -e

echo "[*] Starting validator"
eudico mir validator run --nosync
