#!/bin/bash

# Configure destination host
USER=ubuntu
HOST=10.46.49.242

if (($# != 1)); then
  echo "Arguments: <script_to_be_run_on>"
  exit 1
fi

script=$1

scp $script ${USER}@${HOST}:~/
ssh ${USER}@${HOST} "rm -rf ~/.ipc"
scp -r .ipc ${USER}@${HOST}:~/
ssh ${USER}@${HOST} "bash -il ~/${script}"
