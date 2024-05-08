#!/usr/bin/env bash

# This script exports the standalone testnet files setup by the materializer to the guest machine so we can run them locally

set -e

fendermint_cid=$(docker ps -aqf name=moso-fendermint)
if [ -z "$fendermint_cid" ]; then
    echo "Fendermint container does not exist"
    exit 1
fi
echo "Fendermint container id: $fendermint_cid"

ethapi_cid=$(docker ps -aqf name=moso-ethapi)
if [ -z "$ethapi_cid" ]; then
    echo "Ethapi container does not exist"
    exit 1
fi
echo "Ethapi container id: $ethapi_cid"

cometbft_cid=$(docker ps -aqf name=moso-cometbft)
if [ -z "$cometbft_cid" ]; then
    echo "Cometbft container does not exist"
    exit 1
fi
echo "Cometbft container id: $cometbft_cid"

# Create the out directory
rm -rf out
mkdir -p out/fendermint out/cometbft

# Stop the containers
echo docker stop $fendermint_cid $ethapi_cid $cometbft_cid
docker stop $fendermint_cid $ethapi_cid $cometbft_cid

# Copy the data from the containers
docker cp $fendermint_cid:/fendermint out
docker cp $cometbft_cid:/cometbft out

rm -rf ~/.fendermint && cp -r out/fendermint ~/.fendermint
rm -rf ~/.cometbft && cp -r out/cometbft ~/.cometbft

echo "Data exported successfully"
echo
echo "start fendermint eth api"
echo source ~/workspace/ipc/fendermint/testing/materializer/scripts/fendermint.env
echo ~/workspace/ipc/target/release/fendermint eth run
echo
echo "start fendermint (remember to edit the fendermint.env appropriately)"
echo source ~/workspace/ipc/fendermint/testing/materializer/scripts/fendermint.env
echo ~/workspace/ipc/target/release/fendermint run
echo
echo "start cometbft (remember to edit the cometbft.env appropriately)"
echo cd ~/.cometbft
echo source ~/workspace/ipc/fendermint/testing/materializer/scripts/cometbft.env
echo cometbft start
echo
echo "Once started, you can run the following commands"
echo "Query cometbft:"
echo "  curl http://localhost:26657/status"
echo "Query fendermint eth api:"
echo "  curl -X POST  -H 'Content-Type: application/json' -d '{"jsonrpc":"2.0","id":0,"method":"eth_chainId","params":[]}' http://localhost:8445"
