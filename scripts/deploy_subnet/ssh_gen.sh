#!/bin/bash

set -euo pipefail

ssh-keygen -t ed25519 -N "" -C "git@github.com:recallnet/ipc.git" -f "$HOME"/.ssh/id_ed25519.recallnet.ipc
ssh-keygen -t ed25519 -N "" -C "git@github.com:recallnet/builtin-actors.git" -f "$HOME"/.ssh/id_ed25519.recallnet.builtin-actors
ssh-keygen -t ed25519 -N "" -C "git@github.com:recallnet/contracts.git" -f "$HOME"/.ssh/id_ed25519.recallnet.contracts
