#!/bin/bash

# checks and commit changes in output artifacts
if [[ `git status ./out --porcelain` ]]; then
    echo "********** NOT ALL ABI ARTIFACTS ARE COMMITTED, AUTO PUSH **********\n";
    git add ./out
    git commit -m "commit ABI artifacts"
    git push
fi;
