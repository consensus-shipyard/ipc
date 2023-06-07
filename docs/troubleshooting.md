# Troubleshooting IPC

>💡 For background and setup information, make sure to start with the [README](/README.md).

## I need to upgrade my IPC agent

Sometimes, things break, and we'll need to push a quick path to fix some bug. If this happens, and you need to upgrade your agent version, kill you agent daemon if you have any running, pull the latest changes from this repo, build the binary, and start your daemon again. This should pick up the latest version for the agent. In the future, we will provide a better way to upgrade your agent.
```bash
# Pull latest changes
git pull
# Build the agent
make build
# Restart the daemon
./bin/ipc-agent daemon
```

## The eudico image is not building successful 

`make install-infra` may fail and not build the `eudico` image if your system is not configured correctly. If this happens, you can always try to build the image yourself to have a finer-grain report of the issues to help you debug them. For this you can [follow these instructions](https://github.com/consensus-shipyard/lotus/blob/spacenet/scripts/ipc/README.md).

High-level you just need to clone the [eudico repo](https://github.com/consensus-shipyard/lotus), and run `docker build -t eudico .` in the root of the repo.

## My subnet node doesn't start

Either because the dockerized subnet node after running `./bin/ipc-infra/run-subnet-docker.sh` gets stuck waiting for the API to be started with the following message: 
```
Not online yet... (could not get API info for FullNode: could not get api endpoint: API not running (no endpoint))
```
Or because when the script finishes no validator address has been reported as expected by the logs, the best way to debug this situation is to attach to the docker container:
```bash
docker exec -it <container_name> bash
```
 And check the logs with the following command, inside the container
```bash
tmux a
```
Generally, the issue is that:
- You haven't passed the validator key correctly and it couldn't be imported.
- There was some network instability, and lotus params couldn't be downloaded successfully.

## My agent is not submitting checkpoints after an error

Try running `./bin/ipc-agent config reload`, this should pick up the latest config and restart all checkpointing processes. If the error has been fixed or it was an network instability between the agent and your subnet daemon, checkpoints should start being committed again seamlessly.

### I set the wrong validator address or need to change it

It may be the case that while joining the subnet, you didn't set the multiaddress for your validator correctly and you need to update it. You'll realize that the network address of your validator is not configured correctly, because your agent throws an error when trying to connect to your subnet node, or starting the validator in your subnet throws a network-related error.

Changing the validator is as simple as running the following command:
```bash
./bin/ipc-agent subnet set-validator-net-addr --subnet <subnet-id> --validator-net-addr <new-validator-addr>
```
```console
# Example execution
$ ./bin/ipc-agent subnet set-validator-net-addr --subnet /r31415926/t2xwzbdu7z5sam6hc57xxwkctciuaz7oe5omipwbq --validator-net-addr "/dns/host.docker.internal/tcp/1349/p2p/12D3KooWDeN3bTvZEH11s9Gq5bDeZZLKgRZiMDcy2KmA6mUaT9KE"
```
