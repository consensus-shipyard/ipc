#!/bin/bash
#
# Builds docker image and install the ipc-scripts required to conveniently
# deploy the infrastructure for IPC subnets.

set -e

rm -rf ./lotus
git clone --branch dev https://github.com/consensus-shipyard/lotus.git
cd ./lotus

uname=$(uname);
case "$uname" in
    (*Darwin*) docker build -t eudico --build-arg FFI_BUILD_FROM_SOURCE=1 . ;;
    (*) docker build -t eudico . ;;
esac;

cd ..
mkdir -p ./bin/ipc-infra
cp -rf ./lotus/scripts/ipc/* ./bin/ipc-infra
rm -rf ./lotus
