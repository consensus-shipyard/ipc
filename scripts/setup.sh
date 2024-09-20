#!/bin/sh
set -eu

GOPATH="${GOPATH:-$HOME/go}"
export FM_NETWORK=test

# Create a new Genesis file
rm -rf test-network
mkdir test-network
fendermint genesis --genesis-file test-network/genesis.json new --chain-name test --base-fee 1000 --timestamp 1680101412 --power-scale 3 --credit-debit-interval 10

# Create some keys
mkdir test-network/keys
for NAME in bob charlie dave; do
  fendermint key gen --out-dir test-network/keys --name $NAME;
done

## Use fixed address for alice to make dev w/ ADM cli less painful
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

# Configure Tendermint
rm -rf "$HOME/.cometbft"
"$GOPATH/bin/cometbft" init

## Convert the Genesis file
mv "$HOME/.cometbft/config/genesis.json" "$HOME/.cometbft/config/genesis.json.orig"
fendermint genesis --genesis-file test-network/genesis.json into-tendermint --out "$HOME/.cometbft/config/genesis.json"
## Convert the private key
mv "$HOME/.cometbft/config/priv_validator_key.json" "$HOME/.cometbft/config/priv_validator_key.json.orig"
fendermint key into-tendermint --secret-key test-network/keys/bob.sk --out "$HOME/.cometbft/config/priv_validator_key.json"

## Setup data directory and copy default app config
rm -rf "$HOME/.fendermint"
mkdir -p "$HOME/.fendermint/data"
cp -r ./fendermint/app/config "$HOME/.fendermint/config"

## Generate a network key for the IPLD resolver
mkdir -p "$HOME/.fendermint/keys"
fendermint key gen --out-dir "$HOME/.fendermint/keys" --name network

## Copy validator keys
cp test-network/keys/bob.pk "$HOME/.fendermint/keys/validator.pk"
cp test-network/keys/bob.sk "$HOME/.fendermint/keys/validator.sk"

## Copy IPC contracts
mkdir -p "$HOME/.fendermint/contracts"
cp -r ./contracts/out/* "$HOME/.fendermint/contracts"

# Build actors
(cd builtin-actors && make bundle-mainnet)
mkdir -p fendermint/builtin-actors/output
cp builtin-actors/output/builtin-actors-mainnet.car fendermint/builtin-actors/output/bundle.car
cp fendermint/builtin-actors/output/bundle.car "$HOME/.fendermint/bundle.car"
cargo build --release -p fendermint_actors
cp fendermint/actors/output/custom_actors_bundle.car "$HOME/.fendermint/custom_actors_bundle.car"
