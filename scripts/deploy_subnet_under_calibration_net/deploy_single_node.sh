

export FM_NETWORK=test
export GATEWAY=0x47C59245435cfDd7717E974d3A9F9FaE9D01730f
export REGISTRY=0xfF05a6FE75402Fd0828244AfbD591b23e30487c6

cd ipc
rm -rf test-network && mkdir test-network
rm -rf ~/.fendermint/
rm -rf ~/.cometbft
rm -rf .ipc/

mkdir -p ~/.fendermint/data
cp -r ./fendermint/app/config ~/.fendermint/config

# setup wallet
cd scripts/deploy_subnet_under_calibration_net
./prepare_local.sh

# Single Node
./target/release/ipc-cli subnet create --parent /r314159 --min-validator-stake 0.01 --min-cross-msg-fee 0 --min-validators 0 --bottomup-check-period 50 --permission-mode collateral --supply-source-kind native

export SUBNET=t410fp7ghvl5hczznloihmmxer3m27ap3iwcvrjj4uaa

# open ~/.ipc/config.toml and change the subnet id to the one you just created and the gateway and registry to the GATEWAY and REGISTRY
# open ~/.fendermint/config/config.toml and update the ipc and topdown section

./target/release/ipc-cli subnet join --subnet /r314159/${SUBNET} --collateral 0.02

# Fund an address
./target/release/ipc-cli cross-msg fund --subnet /r314159/${SUBNET} 0.01

# create genesis
cargo run -p fendermint_app --release -- genesis --genesis-file test-network/genesis.json ipc from-parent -s /r314159/${SUBNET} --parent-endpoint "https://api.calibration.node.glif.io/rpc/v1" --parent-gateway ${GATEWAY} --parent-registry ${REGISTRY}

# convert private key
cargo run -p fendermint_app --release -- key eth-to-fendermint --secret-key <path to private key> --name alice --out-dir test-network/keys
cargo run -p fendermint_app --release -- key eth-to-fendermint --secret-key /home/admin/secret_key  --name alice --out-dir test-network/keys

# add account to genesis
cargo run -p fendermint_app --release -- \
        genesis --genesis-file test-network/genesis.json \
        add-account --public-key test-network/keys/alice.pk --balance 10 --kind ethereum

cat test-network/genesis.json

cargo run -p fendermint_app --release -- \
  genesis --genesis-file test-network/genesis.json \
  into-tendermint --out ~/.cometbft/config/genesis.json

cargo run -p fendermint_app --release -- \
  key into-tendermint --secret-key test-network/keys/alice.sk --out ~/.cometbft/config/priv_validator_key.json


cargo run -p fendermint_app --release -- run