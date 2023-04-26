#!/usr/bin/env bash

# TODO: Below looks incomplete. Keep an eye on the original:
# https://github.com/consensus-shipyard/lotus/blob/spacenet/scripts/ipc/src/subnet-validator.sh

set -e

# if [ $# -ne 1 ]
# then
#     echo "Provide the default validator wallet to import as the first argument"
#     exit 1
# fi

# VAL_KEY=$1
# TODO: Optionally we can set the ipc-agent enpdoint
# Right now lets use the default one
# AGENT=$2

eudico wait-api --timeout=300s
sleep 5
echo "[*] Importing validator key"
eudico wallet import --as-default --format=json-lotus /wallet.key
eudico mir validator config init
# eudico mir validator config validator-addr
# eudico mir validator run --membership=onchain --nosync
