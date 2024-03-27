#!/bin/sh
set -eu

rm -rf ~/.fendermint/data/rocksdb
FM_NETWORK=test FM_RESOLVER__CONNECTION__LISTEN_ADDR=/ip4/127.0.0.1/tcp/3001 FM_VALIDATOR_KEY__PATH="/Users/avichalpandey/work/ipc/test-network/keys/bob.sk" FM_VALIDATOR_KEY__KIND="regular" FM_IPC__SUBNET_ID=/r31415926 fendermint run
