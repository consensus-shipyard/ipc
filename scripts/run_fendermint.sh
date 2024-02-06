#!/bin/sh
set -eu

rm -rf ~/.fendermint/data/rocksdb
fendermint run
