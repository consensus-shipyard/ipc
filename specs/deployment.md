# IPC Spec - Deployment

This document explains the steps required to deploy a IPC subnet anchored against calibration, both using a docker container, as well as on your host PC.

# Deployment using Docker

See https://docs.ipc.space/quickstarts/deploy-a-subnet

# Export fendermint/cometbft from Docker to host

After provisioning your docker container, it is often difficult to replace their target binaries due to different architecture, glibc versions, etc. This makes it hard to compile, run and debug your own `fendermint` binary when developing.

To circumvent this we can copy all local fendermint/cometbft files from the docker containers to your local host PC and run the processess there using your own compiled fendermint/cometbft targets.

Given that you have the following running set of docker containers which are currently running and producing blocks:

- `fendermint`
- `ethapi`
- `cometbft`

Then, by running the following commands, we will be able to export all the files to your host PC and run them there using your own target binaries:

```bash
# Collect the container IDs
fendermint_cid=$(docker ps -aqf name=fendermint)
ethapi_cid=$(docker ps -aqf name=ethapi)
cometbft_cid=$(docker ps -aqf name=cometbft)

# Stop the containers
docker stop $fendermint_cid $ethapi_cid $cometbft_cid

# Export the files to host PC (make sure these files don't already exist)
docker cp $fendermint_cid:/fendermint ~/.fendermint
docker cp $cometbft_cid:/cometbft ~/.cometbft

# start fendermint (in a new window)
cargo run -p fendermint_app --release -- run

# start ethapi (in a new window)
cargo run -p fendermint_app --release -- eth run

# start cometbft (in a new window)
cd ~/.cometbft
cometbft start
```

<aside>
💡 Note: You need to adjust the ENV variables that are set inside the Docker environment  appropriatly on your host PC. You can list the env variables inside each docker container by running `docker exec CONTAINER_ID env` and replacing CONTAINER_ID appropriately.

</aside>
