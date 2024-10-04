#!/bin/bash

set -euo pipefail

get_open_port() {
    local PORT=$1
    while lsof -i :"$PORT" >/dev/null 2>&1 || netstat -tuln | grep ":$PORT " >/dev/null 2>&1; do
        PORT=$((PORT+1))
    done
    echo "$PORT"
}

get_open_port "$1"
