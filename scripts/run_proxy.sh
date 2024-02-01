#!/bin/sh
set -eu

fendermint proxy start --secret-key test-network/keys/alice.sk --chain-name test --broadcast-mode async
