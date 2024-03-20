#!/bin/sh
set -eu

rm -rf ~/.fendermint/data/rocksdb
FM_NETWORK=test FM_RESOLVER__CONNECTION__LISTEN_ADDR=/ip4/127.0.0.1/tcp/3001 FM_IPC__SUBNET_ID=/r31415926 fendermint run
