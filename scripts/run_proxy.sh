#!/bin/sh
set -eu

fendermint proxy run --secret-key test-network/keys/alice.sk --account-kind ethereum --chain-name test --broadcast-mode commit --sequence 0
