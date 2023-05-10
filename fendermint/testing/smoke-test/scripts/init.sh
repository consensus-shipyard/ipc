#!/usr/bin/env bash

# Create test artifacts, which is basically the Tendermint genesis file.

KEYS_DIR=/data/fendermint/keys
GENESIS_FILE=/data/fendermint/genesis.json

# Create a genesis file
fendermint genesis --genesis-file $GENESIS_FILE new --network-name smoke --base-fee 1000  --timestamp 1680101412

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

# Add a validator
fendermint \
  genesis --genesis-file $GENESIS_FILE \
  add-validator --public-key $KEYS_DIR/bob.pk --power 1

# Convert FM genesis to TM
fendermint \
  genesis --genesis-file $GENESIS_FILE \
  into-tendermint --out /data/tendermint/config/genesis.json

# Convert FM validator key to TM
fendermint \
  key into-tendermint --secret-key $KEYS_DIR/bob.sk \
    --out /data/tendermint/config/priv_validator_key.json
