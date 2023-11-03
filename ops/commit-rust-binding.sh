#!/bin/bash

# checks and commit changes in rust binding
if [[ `git status ./binding --porcelain` ]]; then
    echo "********** NOT ALL RUST BINDINGS COMMITTED, AUTO PUSH **********\n";
    git add ./binding
    git commit -m "commit rust binding"
    git push
fi;
