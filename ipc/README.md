# InterPlanetary Consensus (IPC)

**‼️ All the modules in the IPC stack (including the contracts) haven't been audited, tested in depth, or otherwise verified. Moreover, the system is missing critical recovery functionality in case of crashes. There are multiple ways in which you may lose funds moved into an IPC subnet, and we strongly advise against deploying IPC on mainnet and/or using it with tokens with real value.**

This repo is your entrypoint to the world of IPC. In this repo you will find:
- `ipc-cli`: The IPC CLI is a client application that provides a simple and easy-to-use interface to interact with IPC as a user and run all the processes required for the operation of a subnet.
- `./ipc/provider` A Rust crate that implements the `IpcProvider` library. This provider can be used to interact with IPC from Rust applications (and is what the `ipc-cli` uses under the hood).
- `./ipc/sdk`: Common Rust IPC types and bindings


>💡 **We've prepared a [quick start guide](/docs/quickstart-calibration.md) that will have you running and validating on your own subnet quickly, at the cost of detailed explanations.**

See:
- [docs/contracts.md](docs/contracts.md) for instructions on how to deploy FEVM actors on subnets
- [docs/usage.md](docs/usage.md) for instructions on how to use the `ipc-cli` to interact with subnets (from managing your identities, to sending funds to a subnet).
- [docs/deploying-hierarchy.md](docs/deploying-hierarchy.md) for instructions on how to deploy your own instance of IPC on a network.

## Branching Strategy

### Production branch

The production branch is `main`.
The `main` branch is always compatible with the "main" branch of Fendermint.
Updates to `main` **always** come from the `dev` branch.

### Development branch

The primary development branch is `dev`.
`dev` contains the most up-to-date software but may not be compatible with the rest of the stack. Only use `dev` if doing a full local deployment, but note that the packaged deployment scripts default to checking out eudico `main`. 

## Building

To build the `ipc-cli` you need to have Rust installed in your environment. We currently use Rust `stable` (as described in the `toolchain`). You can look for instructions on [how to run Rust and rustup following this link](https://www.rust-lang.org/tools/install).

>💡 According to the operating system you are running, you may have to install additional dependencies not installed in your system to follow these instructions like `build-essential`, `libssl-dev`, `git`, `curl`, and `pkg-config`. If something fails while building the binaries double-check these dependencies.

To build the binary for the `ipc-cli` you need to build the requirements in your environment, clone this repo, and build the binary following these steps:
```bash
git clone https://github.com/consensus-shipyard/ipc.git
cd ipc
rustup target add wasm32-unknown-unknown
make build
```

This builds the binary of the `ipc-cli` in the `./bin` folder of your repo. If you want to make the command available everywhere, add this folder to the binary `PATH` of your system. You can run the following command to see if the installation was successful:

## Fendermint

IPC uses [Fendermint](https://github.com/consensus-shipyard/fendermint) as the underlying peer implementation to run subnets. To ease the deployment of new nodes, Fendermint provides [a set of infrastructure scripts](https://github.com/consensus-shipyard/fendermint/infra) that make use of Docker. In order to install Docker, [click this link](https://docs.docker.com/get-docker/) and follow the instructions for your working environment.

>💡 Some users have reported some issues trying to build the required images using Docker Desktop. Consider installing a version of [Docker engine](https://docs.docker.com/engine/install/#server) supported by your system.

With Docker installed, you can then `make install-infra` in the root of the `ipc` repo. This will clone the fendermint repo, build the docker image that you need to run subnets, and make the infrastructure scripts available in `./bin/ipc-infra/fendermint/infra` folder.

In Unix-based systems, it is highly recommended to include your user in the `docker` group to avoid having to run many of the commands from this tutorial using `sudo`. You can achieve this running:
```bash
sudo usermod -aG docker $USER
newgrp docker
```

## Connecting to a rootnet

You can deploy an IPC hierarchy from any compatible rootnet. The recommended option is to use Filecoin Calibration, but you can also deploy your own.

### Running a subnet in Calibration
Calibration is the primary testnet for Filecoin. It already hosts the IPC actors and can be used as a rootnet on which to deploy new subnets.

In order to use the `ipc-cli` with Calibration we need to have access to a full node syncing with the network. The easiest way to achieve this is to use a [public RPC](https://docs.filecoin.io/networks/calibration/rpcs/). You also need the addresses of the deployed contracts.

If it is the first time that you use your `ipc-cli`, to initialize cli configuration you can run `./bin/ipc-cli config init`. This will populate a new default config file in `~/.ipc/config.toml`.

The suggested configuration for the `ipc-cli` is:

```
keystore_path = "~/.ipc"

# Filecoin Calibration
[[subnets]]
id = "/r314159"

[subnets.config]
network_type = "fevm"
provider_http = "https://api.calibration.node.glif.io/rpc/v1"
gateway_addr = "0x1AEe8A878a22280fc2753b3C63571C8F895D2FE3"
registry_addr = "0x0b4e239FF21b40120cDa817fba77bD1B366c1bcD"

# Mycelium Calibration
[[subnets]]
id = "/r314159/t410fx23amesh6qvzfzl744uzdr76vlsysb6nnp3us4q"

[subnets.config]
network_type = "fevm"
provider_http = "https://api.mycelium.calibration.node.glif.io/"
gateway_addr = "0x77aa40b105843728088c0132e43fc44348881da8"
registry_addr = "0x74539671a1d2f1c8f200826baba665179f53a1b7"
```

To be able to interact with Calibration and run new subnets, some FIL should be provided to, at least, the wallet that will be used by the `ipc-cli` to interact with IPC. You can request some tFIL for your address through the [Calibration Faucet](https://faucet.calibration.fildev.network/funds.html).

## Help

If you meet any obstacles, please check [docs/troubleshooting.md](docs/troubleshooting.md) or join us in **#ipc-help** in the [Filecoin Slack workspace](https://filecoin.io/slack).
