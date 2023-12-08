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
forge build -C ./src/ --via-ir --sizes --skip test --out=$OUTPUT
