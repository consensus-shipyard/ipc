#!/bin/bash
set -eu

sudo apt install -y build-essential clang make git

curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y

wget https://go.dev/dl/go1.20.5.linux-amd64.tar.gz
sudo tar -C /usr/local -xzf go1.20.5.linux-amd64.tar.gz
echo 'export PATH=$PATH:/usr/local/go/bin' >> ~/.bashrc
echo 'export PATH=$PATH:$HOME/go/bin' >> ~/.bashrc

source "$HOME"/.bashrc

git clone https://github.com/textileio/ipc.git
cd ipc
git checkout textile-exp1
cd ..

git clone https://github.com/cometbft/cometbft.git
cd cometbft
git checkout v0.37.1
make install
cd ..

cd ipc
./scripts/setup.sh
