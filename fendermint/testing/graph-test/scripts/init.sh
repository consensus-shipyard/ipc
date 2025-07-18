#!/usr/bin/env bash

set -e

# Create test artifacts, which is basically the Tendermint genesis file.

KEYS_DIR=/data/keys
CMT_DIR=/data/${NODE_NAME}/cometbft
GENESIS_FILE=/data/genesis.json
SEALED_GENESIS_FILE=/data/sealed.car

fendermint key gen --out-dir $KEYS_DIR --name owner
fendermint key into-eth --secret-key $KEYS_DIR/owner.sk --name $KEYS_DIR/contracts-owner

# Create a genesis file
fendermint \
  genesis --genesis-file $GENESIS_FILE \
  new \
    --chain-name $FM_CHAIN_NAME \
    --base-fee 1000 \
    --timestamp 1680101412 \
    --power-scale 0 \
    --ipc-contracts-owner $(cat $KEYS_DIR/contracts-owner.addr)

# Create some validators
mkdir -p $KEYS_DIR
for NAME in victoria veronica vivienne; do
  fendermint key gen --out-dir $KEYS_DIR --name $NAME;

  # Create Ethereum accounts for them.
  fendermint \
    genesis --genesis-file $GENESIS_FILE \
    add-account --public-key $KEYS_DIR/$NAME.pk \
                --balance 1000 \
                --kind ethereum

  # Convert FM validator key to CMT
  fendermint \
    key into-tendermint --secret-key $KEYS_DIR/$NAME.sk \
      --out $KEYS_DIR/$NAME.priv_validator_key.json

  # Convert FM validator key to ETH
  fendermint \
    key into-eth --out-dir $KEYS_DIR \
      --secret-key $KEYS_DIR/$NAME.sk --name $NAME-eth;
done

# Add a validator
VALIDATOR_NAME=victoria

fendermint \
  genesis --genesis-file $GENESIS_FILE \
  add-validator --public-key $KEYS_DIR/$VALIDATOR_NAME.pk --power 1

# Seal the genesis state
fendermint \
  genesis --genesis-file $GENESIS_FILE \
  ipc \
    seal-genesis \
      --builtin-actors-path /fendermint/bundle.car \
      --custom-actors-path /fendermint/custom_actors_bundle.car \
      --artifacts-path /fendermint/contracts \
      --output-path "${SEALED_GENESIS_FILE}"

# Convert FM genesis to CMT
fendermint \
  genesis --genesis-file $GENESIS_FILE \
  into-tendermint --out $CMT_DIR/config/genesis.json --app-state "${SEALED_GENESIS_FILE}"

# Copy the default validator key
cp $KEYS_DIR/$VALIDATOR_NAME.priv_validator_key.json \
   $CMT_DIR/config/priv_validator_key.json
