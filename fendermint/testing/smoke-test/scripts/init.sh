#!/usr/bin/env bash

set -e

# Create test artifacts, which is basically the Tendermint genesis file.

CMT_DIR=/data/cometbft
FM_DIR=/data/fendermint

KEYS_DIR=$FM_DIR/keys
GENESIS_FILE=$FM_DIR/genesis.json

# Create a genesis file
fendermint genesis --genesis-file $GENESIS_FILE new --chain-name $FM_CHAIN_NAME --base-fee 1000  --timestamp 1680101412

# Create test keys
mkdir -p $KEYS_DIR
for NAME in alice bob charlie dave; do
  fendermint key gen --out-dir $KEYS_DIR --name $NAME;
done

# Create an account
fendermint \
  genesis --genesis-file $GENESIS_FILE \
  add-account --public-key $KEYS_DIR/alice.pk \
              --balance 1000000000000000000

# Create a multisig account
fendermint \
  genesis --genesis-file $GENESIS_FILE \
  add-multisig  --public-key $KEYS_DIR/bob.pk \
                --public-key $KEYS_DIR/charlie.pk \
                --public-key $KEYS_DIR/dave.pk \
                --threshold 2 --vesting-start 0 --vesting-duration 1000000 \
                --balance 3000000000000000000

# Create some Ethereum accounts
for NAME in emily eric; do
  fendermint key gen --out-dir $KEYS_DIR --name $NAME;
  fendermint \
    genesis --genesis-file $GENESIS_FILE \
    add-account --public-key $KEYS_DIR/$NAME.pk \
                --balance 1000000000000000000000 \
                --kind ethereum
done

# Add a validator
fendermint \
  genesis --genesis-file $GENESIS_FILE \
  add-validator --public-key $KEYS_DIR/bob.pk --power 1

# Convert FM genesis to CMT
fendermint \
  genesis --genesis-file $GENESIS_FILE \
  into-tendermint --out $CMT_DIR/config/genesis.json

# Convert FM validator key to CMT
fendermint \
  key into-tendermint --secret-key $KEYS_DIR/bob.sk \
    --out $CMT_DIR/config/priv_validator_key.json
