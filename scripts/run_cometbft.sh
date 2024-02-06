#!/bin/sh
set -eu

cometbft unsafe-reset-all
cometbft start --consensus.create_empty_blocks=false
