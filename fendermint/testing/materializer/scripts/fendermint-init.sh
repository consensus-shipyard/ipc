#!/usr/bin/env bash

set -e

# Volumes mounted to the fendermint init step.
KEYS_DIR=/data/keys
CMT_DIR=/data/cometbft
GENESIS_FILE=/data/genesis.json

NETWORK_KEY_NAME=network_key
VALIDATOR_KEY_NAME=validator_key
VALIDATOR_KEY_FILE=$KEYS_DIR/$VALIDATOR_KEY_NAME.sk

if [ -f VALIDATOR_KEY_FILE ]; then
  # Convert FM validator key to CMT
  fendermint \
    key into-tendermint --secret-key $VALIDATOR_KEY_FILE \
      --out $CMT_DIR/config/priv_validator_key.json
fi

# Convert FM genesis to CMT
fendermint \
  genesis --genesis-file $GENESIS_FILE \
  into-tendermint --out $CMT_DIR/config/genesis.json

# Create a network key for the fendermint IPLD resolver
fendermint \
  key gen --out-dir $KEYS_DIR --name $NETWORK_KEY_NAME
