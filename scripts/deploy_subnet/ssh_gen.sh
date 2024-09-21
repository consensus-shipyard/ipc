#!/bin/bash

set -euo pipefail

ssh-keygen -t ed25519 -N "" -C "git@github.com:hokunet/ipc.git" -f "$HOME"/.ssh/id_ed25519.hokunet.ipc
ssh-keygen -t ed25519 -N "" -C "git@github.com:hokunet/builtin-actors.git" -f "$HOME"/.ssh/id_ed25519.hokunet.builtin-actors
ssh-keygen -t ed25519 -N "" -C "git@github.com:hokunet/contracts.git" -f "$HOME"/.ssh/id_ed25519.hokunet.contracts
