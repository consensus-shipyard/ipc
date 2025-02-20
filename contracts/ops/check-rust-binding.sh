#!/bin/bash
set -eu
set -o pipefail

# checks if there are changes in rust binding
if [[ `git status ../contract-bindings --porcelain` ]]; then
    echo "!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!"
    echo "********** NOT ALL RUST BINDINGS COMMITTED, COMMIT THEM **********\n";
    git status ../contract-bindings --porcelain
    echo "!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!\n"
    exit 1;
fi;