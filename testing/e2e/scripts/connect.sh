#!/usr/bin/env bash

# Create a subnet configuration file to connect an agent to a node.

set -e

if [ $# -ne 2 ]
then
    echo "usage: ./connect.sh <agent-dir> <node-dir>"
    exit 1
fi

IPC_AGENT_DIR=$1
IPC_NODE_DIR=$2

source $IPC_AGENT_DIR/.env
source $IPC_NODE_DIR/.env

# This just looks like some kind of human readable name for the subnet.
if [ -z "${IPC_SUBNET_NAME}" ]; then
  IPC_SUBNET_NAME=$(basename $IPC_SUBNET_ID)
fi

echo "[*] Connecting agent-$IPC_AGENT_NR to $IPC_NODE_TYPE node-$IPC_NODE_NR in subnet $IPC_SUBNET_ID ($IPC_SUBNET_NAME)"

write_subnet_config() {
  TOKEN=$(echo $1  | tr -d '\r\n')
  WALLET=$(echo $2 | tr -d '\r\n')

  SUBNETS_DIR=$IPC_AGENT_DIR/subnets
  mkdir -p $SUBNETS_DIR

  SUBNET_CONFIG=$SUBNETS_DIR/node-$IPC_NODE_NR
  echo "[*] Writing subnet config to $SUBNET_CONFIG"

  # The JSON-API URL is from the perspective of one container connecting to another,
  # e.g. the agent container to the eudico daemon. It needs to mach the settings in
  # the compose file.
  cat <<EOF > $SUBNET_CONFIG

[[subnets]]
id = "${IPC_SUBNET_ID}"
gateway_addr = "t064"
network_name = "${IPC_SUBNET_NAME}"
jsonrpc_api_http = "http://${IPC_NODE_TYPE}-${IPC_NODE_NR}:1234/rpc/v1"
auth_token = "${TOKEN}"
accounts = ["${WALLET}"]

EOF
}


if [ "${IPC_NODE_TYPE}" == "eudico" ]; then
  # The following is based on `run-root-docker-1val.sh`
  DAEMON_ID=ipc-node-${IPC_NODE_NR}-daemon

  echo "[*] Waiting for the daemon to start"
  docker exec -it $DAEMON_ID eudico wait-api --timeout 350s
  sleep 5

  echo "[*] Creating admin token"
  TOKEN=$(docker exec -it $DAEMON_ID eudico auth create-token --perm admin)

  echo "[*] Getting default wallet"
  WALLET=$(docker exec -it $DAEMON_ID  eudico wallet default)

  write_subnet_config $TOKEN $WALLET

else
  echo "Don't know how to connect node type $IPC_NODE_TYPE";
  exit 1;
fi
