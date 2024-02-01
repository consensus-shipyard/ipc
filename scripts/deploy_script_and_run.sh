#!/bin/bash

USER=ubuntu
HOST=192.168.64.2

set -x

scp deploy_calibration.sh ${USER}@${HOST}:~/
ssh ${USER}@${HOST} 'rm -rf ~/.ipc'
scp -r .ipc ${USER}@${HOST}:~/
ssh ${USER}@${HOST} 'bash -il ~/deploy_calibration.sh'