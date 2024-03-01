#!/bin/sh
set -eu

fendermint proxy run --secret-key test-network/keys/alice.sk --chain-name test --broadcast-mode async --sequence 0
