#!/bin/bash

set -e

# Finds the checkout location of the IPC Solidity actors.

CARGO_HOME=${CARGO_HOME}
IPC_ACTORS_TAG=$1
IPC_ACTORS_NOFETCH=${IPC_ACTORS_NOFETCH:-0}

if [ -z "$CARGO_HOME" ]; then
  CARGO_HOME=$(dirname $(dirname $(which cargo)))
fi

if [ ! -d "$CARGO_HOME" ]; then
  >&2 echo "CARGO_HOME does not exist: $CARGO_HOME"
  exit 1
fi

CARGO_CHECKOUTS_DIR=$CARGO_HOME/git/checkouts

if [ ! -d "$CARGO_CHECKOUTS_DIR" ]; then
  # It is possible that cargo hasn't checked out anything yet.
  # This needs to be handled in the Makefile.
  >&2 echo "CARGO_CHECKOUTS_DIR does not exist: $CARGO_CHECKOUTS_DIR"
  exit 0
fi

IPC_ACTORS_BINDING=$(find $CARGO_CHECKOUTS_DIR -type f -wholename "*/ipc-solidity-actors-*/*/binding/Cargo.toml")

if [ -z "$IPC_ACTORS_BINDING" ]; then
  >&2 echo "Cannot find IPC actor bindings"
  exit 1
fi

IPC_ACTORS_DIR=""

if [ $(echo "$IPC_ACTORS_BINDING" | wc -l) -gt 1 ]; then
  >&2 echo -e "Found multiple IPC actor bindings:\n$IPC_ACTORS_BINDING"

  if [ ! -z $IPC_ACTORS_TAG ]; then
    IPC_ACTORS_DIR=$PWD/../ipc-solidity-actors
    >&2 echo -e "Falling back to $IPC_ACTORS_TAG"
    # We could switch into one of the checkouts and use `git show -s --format="%H" $IPC_ACTORS_TAG`
    # to see if the hash is one of them, but there is no guarantee that the Makefile is kept in sync,
    # so we might as well just do another checkout.
    if [ ! -d $IPC_ACTORS_DIR ]; then \
      mkdir -p $IPC_ACTORS_DIR && \
      cd $IPC_ACTORS_DIR
      cd ..
      git clone https://github.com/consensus-shipyard/ipc-solidity-actors.git; \
    fi
    if [ $IPC_ACTORS_NOFETCH != "1" ]; then
      cd $IPC_ACTORS_DIR
      git fetch origin
      git checkout origin/$IPC_ACTORS_TAG
    fi
  else
    exit 1
  fi
else
  IPC_ACTORS_DIR=$(dirname $(dirname $IPC_ACTORS_BINDING))
fi

echo $IPC_ACTORS_DIR
