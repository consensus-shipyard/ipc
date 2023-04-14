# Troubleshooting

## I need to upgrade my IPC agent

Sometimes, things break, and we'll need to push a quick path to fix some bug. If this happens, and you need to upgrade your agent version, kill you agent daemon if you have any running, pull the latest changes from this repo, build the binary, and start your daemon again. This should pick up the latest version for the agent. In the future, we will provide a better way to upgrade your agent.
```bash
# Pull latest changes
$ git pull
# Build the agent
$ make build
# Restart the daemon
$ ./bin/ipc-agent daemon
```

## The eudico image is not building successful 

`make install-infra` may fail and not build the `eudico` image if your system is not configured correctly. If this happens, you can always try to build the image yourself to have a finer-grain report of the issues to help you debug them. For this you can [follow these instructions](https://github.com/consensus-shipyard/lotus/blob/spacenet/scripts/ipc/README.md).

High-level you just need to clone the [eudico repo](https://github.com/consensus-shipyard/lotus), and run `docker build -t eudico .` in the root of the repo.

## My subnet node doesn't start

Either because the dockerized subnet node after running `./bin/ipc-infra/run-subnet-docker.sh` gets stuck waiting for the API to be started with the following message: 
```
Not online yet... (could not get API info for FullNode: could not get api endpoint: API not running (no endpoint))
```
Or because when the script finishes no validator address has been reported as expected by the logs, the best way to debug this situation is to attach to the docker container and check the logs with the following command:
```bash
$ docker exec -it <container_name/id> bash

# Inside the container
tmux a
```
Generally, the issue is that:
- You haven't passed the validator key correctly and it couldn't be imported.
- There was some network instability, and lotus params couldn't be downloaded successfully.

## My agent is not submitting checkpoints after an error

Try running `./bin/ipc-agent config reload`, this should pick up the latest config and restart all checkpointing processes. If the error has been fixed or it was an network instability between the agent and your subnet daemon, checkpoints should start being committed again seamlessly.
