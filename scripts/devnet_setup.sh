#!/bin/bash

set -euo pipefail

export FM_NETWORK=test

# Clean up old network files
rm -rf test-network
mkdir test-network

## Setup data directory and copy default app config
rm -rf "$HOME/.fendermint"
mkdir -p "$HOME/.fendermint/data"
cp -r ./fendermint/app/config "$HOME/.fendermint/config"

# Init CometBFT
rm -rf "$HOME/.cometbft"
cometbft init

# Build actors
if [[ -z ${SKIP_BUILD+x} || "$SKIP_BUILD" == "" || "$SKIP_BUILD" == "false" ]]; then
  (cd builtin-actors && make bundle-mainnet)
  mkdir -p fendermint/builtin-actors/output
  cp builtin-actors/output/builtin-actors-mainnet.car fendermint/builtin-actors/output/bundle.car
  # These need to be built for release
  cargo build --release -p fendermint_actors
fi
cp fendermint/builtin-actors/output/bundle.car "$HOME/.fendermint/bundle.car"
cp fendermint/actors/output/custom_actors_bundle.car "$HOME/.fendermint/custom_actors_bundle.car"

## Copy IPC contracts
mkdir -p "$HOME/.fendermint/contracts"
cp -r ./contracts/out/* "$HOME/.fendermint/contracts"

# Create a new Genesis file
fendermint genesis --genesis-file test-network/genesis.json new --chain-name test --base-fee 1000 --timestamp 1680101412 --power-scale 3

# Create some keys
mkdir test-network/keys
for NAME in bob charlie dave; do
  fendermint key gen --out-dir test-network/keys --name $NAME;
done

## Use fixed address for alice to make dev w/ ADM cli less painful
## Private key hex: 1c323d494d1d069fe4c891350a1ec691c4216c17418a0cb3c7533b143bd2b812
echo "HDI9SU0dBp/kyJE1Ch7GkcQhbBdBigyzx1M7FDvSuBI=" | tr -d '\n' > test-network/keys/alice.sk
echo "Ayh506Z/KRZnDgtarffTZQympqQ8A4hfwse1gK9t0NJi" | tr -d '\n' > test-network/keys/alice.pk

# Add accounts to the Genesis file
## A stand-alone account
fendermint genesis --genesis-file test-network/genesis.json add-account --public-key test-network/keys/alice.pk --balance 1000 --kind ethereum
## A multi-sig account
fendermint genesis --genesis-file test-network/genesis.json add-multisig --public-key test-network/keys/bob.pk --public-key test-network/keys/charlie.pk --public-key test-network/keys/dave.pk --threshold 2 --vesting-start 0 --vesting-duration 1000000 --balance 30

# Add validators to the Genesis file
fendermint genesis --genesis-file test-network/genesis.json add-validator --public-key test-network/keys/bob.pk --power 1

# Add ipc to the Genesis file
fendermint genesis --genesis-file test-network/genesis.json ipc gateway --subnet-id /r31415926 --bottom-up-check-period 10 --msg-fee 1 --majority-percentage 65

# Seal Genesis file
fendermint genesis --genesis-file test-network/genesis.json ipc seal-genesis --builtin-actors-path "$HOME/.fendermint/bundle.car" --custom-actors-path "$HOME/.fendermint/custom_actors_bundle.car" --artifacts-path "$HOME/.fendermint/contracts" --output-path test-network/sealed_genesis.car

## Convert the Genesis file
mv "$HOME/.cometbft/config/genesis.json" "$HOME/.cometbft/config/genesis.json.orig"
fendermint genesis --genesis-file test-network/genesis.json into-tendermint --app-state test-network/sealed_genesis.car --out "$HOME/.cometbft/config/genesis.json"
## Convert the private key
mv "$HOME/.cometbft/config/priv_validator_key.json" "$HOME/.cometbft/config/priv_validator_key.json.orig"
fendermint key into-tendermint --secret-key test-network/keys/bob.sk --out "$HOME/.cometbft/config/priv_validator_key.json"

## Generate a network key for the IPLD resolver
mkdir -p "$HOME/.fendermint/keys"
fendermint key gen --out-dir "$HOME/.fendermint/keys" --name network

## Copy validator keys
cp test-network/keys/bob.pk "$HOME/.fendermint/keys/validator.pk"
cp test-network/keys/bob.sk "$HOME/.fendermint/keys/validator.sk"
