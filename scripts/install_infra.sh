#!/bin/bash
#
# Builds docker image and install the ipc-scripts required to conveniently
# deploy the infrastructure for IPC subnets.

set -e

rm -rf ./lotus
git clone https://github.com/consensus-shipyard/lotus.git
cd ./lotus
docker build -t eudico .
cd ..
mkdir -p ./bin/ipc-infra
cp -rf ./lotus/scripts/ipc/* ./bin/ipc-infra
rm -rf ./lotus
