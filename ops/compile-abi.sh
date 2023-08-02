#!/bin/bash
# Compile contract and output core contracts ABI
set -e

if [ $# -ne 1 ]
then
    echo "Expected a single argument with the output directory for the compiled contracts"
    exit 1
fi

OUTPUT=$1

echo "[*] Compiling contracts and output core contracts ABI in $OUTPUT" 
forge build --via-ir --sizes --skip test --out=$OUTPUT

mkdir -p $OUTPUT

cp $OUTPUT/SubnetActorGetterFacet.sol/* $OUTPUT
cp $OUTPUT/SubnetActorManagerFacet.sol/* $OUTPUT
cp $OUTPUT/SubnetActorDiamond.sol/* $OUTPUT

cp $OUTPUT/GatewayGetterFacet.sol/* $OUTPUT
cp $OUTPUT/GatewayManagerFacet.sol/* $OUTPUT
cp $OUTPUT/GatewayRouterFacet.sol/* $OUTPUT
cp $OUTPUT/GatewayDiamond.sol/* $OUTPUT

cp $OUTPUT/SubnetRegistry.sol/* $OUTPUT