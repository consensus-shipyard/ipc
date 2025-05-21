#!/usr/bin/env bash

set -ex

# Setup signal handling for graceful shutdown
function cleanup() {
  echo "Received termination signal. Shutting down containers..."
  cd /workdir/localnet-data
  for d in node-*; do
    (
      cd $d/workdir
      docker compose down
    )
  done
  docker stop localnet-anvil
  docker network rm recall-localnet
  pkill -TERM dockerd
  exit 0
}

# Register the cleanup function for these signals
trap cleanup SIGTERM SIGINT

nohup dockerd &> /dev/null &
DOCKERD_PID=$!
while ! docker info > /dev/null; do
  sleep 1
done

docker build -t anvil -f ./docker/anvil.Dockerfile ./docker
docker network create recall-localnet || true
docker run --rm --name localnet-anvil -u nobody -d --network recall-localnet -p 0.0.0.0:8545:8545 -v /workdir/localnet-data/anvil:/workdir anvil

cd localnet-data
for d in node-*; do
  (
    cd $d/workdir
    docker compose up -d
  )
done

# Keep container running until terminated
echo "All containers started. Waiting for termination signal..."
wait $DOCKERD_PID
