#!/bin/bash

# Configure destination host
USER=ubuntu
HOST=172.104.248.232

if (($# != 1)); then
  echo "Arguments: <script_to_be_run_on>"
  exit 1
fi

script=$1

scp $script ${USER}@${HOST}:~/
ssh ${USER}@${HOST} "sudo rm -rf ~/.ipc"
scp -r .ipc ${USER}@${HOST}:~/
ssh ${USER}@${HOST} "bash -il ~/${script}"
