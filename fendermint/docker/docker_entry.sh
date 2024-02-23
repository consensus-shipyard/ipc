#!/bin/bash

# This script should be used as ENTRYPOINT for Fendermint docker image.

CMD=$1
echo "Running docker_entry.sh"

if [[ $CMD == 'ipc-cli' ]]; then
  echo "You are running ipc-cli"
  echo "Forwarding argument: "
  echo "${@:2}"
  /usr/local/bin/ipc-cli "${@:2}"
else
  echo "You are running fendermint"
  echo "Forwarding argument: "
  echo "$@"
  if (( $# == 0)); then
    echo "No argument provided. Exec run command by default"
    /usr/local/bin/fendermint run
  else
    /usr/local/bin/fendermint "$@"
  fi
fi
