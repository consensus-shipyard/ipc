#!/bin/bash

# This script should be used as ENTRYPOINT for fendermint docker image.

CMD=$1

if [[ $CMD == 'ipc-cli' ]]; then
  /usr/local/bin/ipc-cli "${@:2}"
else
  if (( $# == 0)); then
    /usr/local/bin/fendermint run
  else
    /usr/local/bin/fendermint "$@"
  fi
fi
