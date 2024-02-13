# Deployment Script

## Deploy to remote host
We currently have a [Github workflow](https://github.com/consensus-shipyard/ipc/actions/workflows/deploy-to-dedicated-host.yaml) to deploy IPC infra to a dedicated host. You can go to the workflow page and click `Run workflow` button on the top right corner to initiate a deployment.

## Deploy to local machine
The same `deploy.sh` script can also be used to deploy locally. This is more or less equivalent to following [quickstart-calibration.md](https://github.com/consensus-shipyard/ipc/blob/main/docs/ipc/quickstart-calibration.md), but much more automated.

To run this script locally, you need to first manually prepare the environment and files.

1. Make sure you have your ipc repo located at $HOME/ipc.
2. Follow Step 2 and Step 3 in [Github workflow](https://github.com/consensus-shipyard/ipc/actions/workflows/deploy-to-dedicated-host.yaml) to prepare ipc config file and wallets. Remember to go to Calibration faucet to fund all of your addresses.
3. Run `bash deploy.sh local` to deploy IPC locally.

Please also notice that
1. The automated dependency installation isn't guarantee to work 100% time. If you encountered any dependency installation issue, please refer to the script and retry. Usually you can resolve the issues by creating a new terminal, sourcing `~/.bash.rc`, etc.
2. Depends on the RPC endpoint's quality of service for the calibration net, your command may or may not succeed when interacting with the RPC endpoint. Sometimes you will get rate limited, or other weird errors. In that case, you can choose a different calibration provider URL (TODO: Add link) to replace the value of `RPC_URL` variable in the script, then retry it.

## What does deploy.sh do?
