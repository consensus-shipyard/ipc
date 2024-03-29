#!/bin/sh

echo "setup clean"
set -xeu

rm -rf test-network
rm -rf ~/.cometbft
rm -rf ~/.fendermint
(cd fendermint && make clean)
