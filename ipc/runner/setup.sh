#!/bin/bash
# Disable immediate exit so we can handle errors manually
set -o pipefail

# Function to print an error message and exit
error_exit() {
  echo "Error: $1"
  exit 1
}

###############################
# 1. Deploy contracts
echo "Deploying contracts..."
DEPLOY_OUTPUT=$(ipc-cli deploy --url http://localhost:8545 \
  --chain-id 31337 \
  --contracts-dir /Users/karlem/work/ipc/contracts/out \
  --from 0xf39fd6e51aad88f6f4ce6ab8827279cfffb92266) || error_exit "Deployment failed"
echo "$DEPLOY_OUTPUT"

###############################
# 2. Parse Registry address
REGISTRY=$(echo "$DEPLOY_OUTPUT" | grep -E "Registry:[[:space:]]*0x[0-9a-fA-F]{40}" | sed 's/.*Registry:[[:space:]]*//' | xargs)
if [ -z "$REGISTRY" ]; then
  error_exit "Could not parse Registry address."
fi
echo "Parsed Registry: $REGISTRY"

###############################
# 3. Parse Gateway address
# First attempt: look for a line like "Gateway : 0x..."
GATEWAY=$(echo "$DEPLOY_OUTPUT" | grep -E "Gateway[[:space:]]*:[[:space:]]*0x[0-9a-fA-F]{40}" | sed 's/.*Gateway[[:space:]]*:[[:space:]]*//' | xargs)
# If that fails, try grabbing the line immediately following the "Deploying top-level contract: GatewayDiamond" log.
if [ -z "$GATEWAY" ]; then
  GATEWAY=$(echo "$DEPLOY_OUTPUT" | awk '/Deploying top-level contract: GatewayDiamond/{getline; print}' | xargs)
fi
if [ -z "$GATEWAY" ]; then
  error_exit "Could not parse Gateway address."
fi
echo "Parsed Gateway: $GATEWAY"

###############################
# 4. Update ~/.ipc/config.toml
echo "Updating ~/.ipc/config.toml..."
cat <<EOF > ~/.ipc/config.toml || error_exit "Failed to update ~/.ipc/config.toml"
keystore_path = "~/.ipc"

# Anvil Subnet
[[subnets]]
id = "/r31337"

[subnets.config]
network_type = "fevm"
provider_http = "http://localhost:8545"
gateway_addr = "$GATEWAY"
registry_addr = "$REGISTRY"
EOF

###############################
# 5. Create the subnet
echo "Creating subnet..."
CREATE_OUTPUT=$(ipc-cli subnet create --parent /r31337 \
  --min-validators 1 \
  --min-validator-stake 1 \
  --active-validators-limit 50 \
  --bottomup-check-period 600 \
  --from 0xf39fd6e51aad88f6f4ce6ab8827279cfffb92266 \
  --permission-mode collateral \
  --supply-source-kind native \
  --collateral-source-kind native) || error_exit "Subnet creation failed"
echo "$CREATE_OUTPUT"

###############################
# 6. Parse the subnet ID and trim whitespace
SUBNET_ID=$(echo "$CREATE_OUTPUT" | grep "created subnet actor with id:" | sed 's/.*created subnet actor with id:[[:space:]]*//' | xargs)
if [ -z "$SUBNET_ID" ]; then
  error_exit "Could not parse subnet id."
fi
echo "Subnet ID: $SUBNET_ID"

###############################
# 7. Join the subnet
echo "Joining subnet..."
ipc-cli subnet join --from 0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266 \
  --subnet "$SUBNET_ID" --collateral 1 --initial-balance 10 || error_exit "Joining subnet failed"

###############################
# 8. List subnets to confirm
echo "Listing subnets..."
ipc-cli subnet list --parent /r31337 || error_exit "Listing subnets failed"

###############################
# 9. Set up FM network and directories
export FM_NETWORK=test || error_exit "Failed to set FM_NETWORK"
echo "Setting FM_NETWORK to test"
echo "Creating ~/.fendermint/data directory..."
rm -rf ~/.fendermint || error_exit "Failed to remove ~/.fendermint"
mkdir -p ~/.fendermint/data || error_exit "Failed to create ~/.fendermint/data directory"

###############################
# 10. Generate genesis file from the parent chain
echo "Generating genesis file from parent..."
fendermint genesis --genesis-file ~/.fendermint/genesis.json ipc from-parent \
  --subnet-id "$SUBNET_ID" \
  --parent-endpoint http://localhost:8545 \
  --parent-gateway "$GATEWAY" \
  --parent-registry "$REGISTRY" || error_exit "Genesis file generation failed"

###############################
# 11. Seal the genesis file
echo "Sealing genesis..."
fendermint genesis --genesis-file ~/.fendermint/genesis.json ipc seal-genesis \
  --artifacts-path /Users/karlem/work/ipc/contracts/out \
  --output-path ~/.fendermint/genesis.sealed.car || error_exit "Sealing genesis failed"

###############################
# 12. Initialize CometBFT
echo "Initializing CometBFT..."
rm -rf ~/.cometbft || error_exit "Failed to remove ~/.cometbft"
cometbft init || error_exit "CometBFT initialization failed"

###############################
# 13. Convert genesis into CometBFT format
echo "Converting genesis to CometBFT format..."
fendermint genesis --genesis-file ~/.fendermint/genesis.json into-tendermint \
  --app-state ~/.fendermint/genesis.sealed.car \
  --out ~/.cometbft/config/genesis.json || error_exit "Conversion to CometBFT format failed"

###############################
# 14. Create a secret file and generate keys
echo "Creating secret file..."
echo "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80" > secret || error_exit "Failed to create secret file"

echo "Generating fendermint key from Ethereum secret..."
fendermint key eth-to-fendermint --secret-key ./secret --name eth || error_exit "Failed to generate fendermint key (eth-to-fendermint)"

echo "Converting key into CometBFT format..."
fendermint key into-tendermint --secret-key eth.sk \
  --out ~/.cometbft/config/priv_validator_key.json || error_exit "Failed to convert key into CometBFT format"

###############################
# 15. Copy configuration files
echo "Copying configuration files..."
cp -r /Users/karlem/work/ipc/fendermint/app/config ~/.fendermint/config || error_exit "Failed to copy configuration files"

echo "Deployment and configuration complete."
