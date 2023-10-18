# InterPlanetary Consensus (IPC)

**‼️ All the modules in the IPC stack (including the contracts) haven't been audited, tested in depth, or otherwise verified. Moreover, the system is missing critical recovery functionality in case of crashes. There are multiple ways in which you may lose funds moved into an IPC subnet, and we strongly advise against deploying IPC on mainnet and/or using it with tokens with real value.**

This repo is your entrypoint to the world of IPC. In this repo you will find:
- `ipc-cli`: The IPC CLI is a client application that provides a simple and easy-to-use interface to interact with IPC as a user and run all the processes required for the operation of a subnet.
- `./ipc/provider` A Rust crate that implements the `IpcProvider` library. This provider can be used to interact with IPC from Rust applications (and is what the `ipc-cli` uses under the hood).
- `./ipc/sdk`: Common Rust IPC types and bindings


>💡 **We've prepared a [quick start guide](/docs/quickstart-calibration.md) that will have you running and validating on your own subnet quickly, at the cost of detailed explanations.**

See:
- [docs/subnet.md](docs/subnet.md) for instructions on how to deploy a new subnet and the required architecture
- [docs/usage.md](docs/usage.md) for instructions on how to use the IPC Agent to interact with subnets
- [docs/deploying-hierarchy.md](docs/deploying-hierarchy.md) for instructions on how to deploy your own IPC root contract and hierarchy
- [docs/contracts.md](docs/contracts.md) for instructions on how to deploy FEVM actors on subnets
- [docs/troubleshooting.md](docs/troubleshooting.md) for answers to some common questions

For a detailed overview of the entire IPC stack design, please check the up-to-date **[IPC Design Reference](https://github.com/consensus-shipyard/IPC-design-reference-spec/blob/main/main.pdf)** doc.

## Branching Strategy

### Production branch

The production branch is `main`.
The `main` branch is always compatible with the "main" branch of Fendermint.
Updates to `main` **always** come from the `dev` branch.

### Development branch

The primary development branch is `dev`.
`dev` contains the most up-to-date software but may not be compatible with the rest of the stack. Only use `dev` if doing a full local deployment, but note that the packaged deployment scripts default to checking out eudico `main`. 

## Building

To build the IPC Agent you need to have Rust installed in your environment. We currently use Rust `stable` (as described in the `toolchain`). You can look for instructions on [how to run Rust and rustup following this link](https://www.rust-lang.org/tools/install).

>💡 According to the operating system you are running, you may have to install additional dependencies not installed in your system to follow these instructions like `build-essential`, `libssl-dev`, `git`, `curl`, and `pkg-config`. If something fails while building the binaries double-check these dependencies.

To build the binary for the `ipc-cli` you need to build the requirements in your environment, clone this repo, and build the binary following these steps:
```bash
git clone https://github.com/consensus-shipyard/ipc-agent.git
cd ipc-agent
rustup target add wasm32-unknown-unknown
make build
```

This builds the binary of the `ipc-cli` in the `./bin` folder of your repo. If you want to make the command available everywhere, add this folder to the binary `PATH` of your system. You can run the following command to see if the installation was successful:

## Fendermint

IPC uses [Fendermint](https://github.com/consensus-shipyard/fendermint) as the underlying peer implementation to run subnets. To ease the deployment of new nodes, Fendermint provides [a set of infrastructure scripts](https://github.com/consensus-shipyard/fendermint/infra) that make use of Docker. In order to install Docker, [click this link](https://docs.docker.com/get-docker/) and follow the instructions for your working environment.

>💡 Some users have reported some issues trying to build the required images using Docker Desktop. Consider installing a version of [Docker engine](https://docs.docker.com/engine/install/#server) supported by your system.

<!-- With Docker installed, you can then `make install-infra` in the root of the `ipc-agent` repo. This will clone the eudico repo, build the docker image that you need to run subnets, and install the infrastructure scripts in the `./bin` folder. -->

<!-- In Unix-based systems, it is highly recommended to include your user in the `docker` group to avoid having to run many of the commands from this tutorial using `sudo`. You can achieve this running: -->
<!-- ```bash -->
<!-- sudo usermod -aG docker $USER -->
<!-- newgrp docker -->
<!-- ``` -->

<!-- ## Connecting to a rootnet -->

<!-- You can deploy an IPC hierarchy from any compatible rootnet. The recommended option is to use Filecoin Calibration, but you can also deploy your own.  -->

<!-- ### Option 1: Calibration -->
<!-- Calibration is the primary testnet for Filecoin. It already hosts the IPC actors and can be used as a rootnet on which to deploy new subnets.  -->

<!-- In order to use the IPC agent with Calibration we need to have access to a full node syncing with the network. The easiest way to achieve this is to use a [public RPC](https://docs.filecoin.io/networks/calibration/rpcs/). You also need the addresses of the deployed contracts The suggested configuration for the IPC agent is: -->

<!-- ``` -->
<!-- # Default configuration for Filecoin Calibration -->
<!-- [[subnets]] -->
<!-- id = "/r314159" -->
<!-- [subnets.config] -->
<!-- gateway_addr = "0x5fBdA31a37E05D8cceF146f7704f4fCe33e2F96F" -->
<!-- network_type = "fevm" -->
<!-- provider_http = "https://api.calibration.node.glif.io/rpc/v1" -->
<!-- registry_addr = "0xb505eD453138A782b5c51f45952E067798F4777d" -->
<!-- ``` -->

<!-- To be able to interact with Calibration and run new subnets, some FIL should be provided to, at least, the wallet that will be used by the agent to interact with IPC. You can request some tFIL for your address through the [Calibration Faucet](https://faucet.calibration.fildev.network/funds.html). -->

<!-- ### Option 2: Local deployment -->
<!-- To deploy a Example rootnet locally for testing you can use the IPC scripts installed in `./bin/ipc-infra` by running: -->
<!-- ```bash -->
<!-- ./bin/ipc-infra/run-root-docker-1val.sh <lotus-api-port> <validator-libp2p-port> -->
<!-- ``` -->

<!-- For instance, running `./bin/ipc-infra/run-root-docker-1val.sh 1234 1235` will run a rootnet daemon listening at `localhost:1234`, and a single validator mining in the rootnet listening through its libp2p host in `localhost:1235`. -->

<!-- *Example*: -->
<!-- ```console -->
<!-- $ ./bin/ipc-infra/run-root-docker-1val.sh 1234 1235 -->
<!-- (...) -->
<!-- >>> Root daemon running in container: 84711d67cf162e30747c4525d69728c4dea8c6b4b35cd89f6d0947fee14bf908 -->
<!-- >>> Token to /r31415926 daemon: eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJBbGxvdyI6WyJyZWFkIiwid3JpdGUiLCJzaWduIiwiYWRtaW4iXX0.j94YYOr8_AWhGGHQd0q8JuQVuNhJA017SK9EUkqDOO0 -->
<!-- >>> Default wallet: t1cp4q4lqsdhob23ysywffg2tvbmar5cshia4rweq -->
<!-- ``` -->
<!-- This information will be relevant to configure our agent to connect to this rootnet node. -->

<!-- ## Configuring the agent -->

<!-- The default config path for the agent is `~/.ipc-agent/config.toml`. The agent will always try to pick up the config from this path unless told otherwise. To populate an example config file in the default path, you can run the following command: -->
<!-- ```bash -->
<!-- ./bin/ipc-agent config init -->
<!-- ``` -->

<!-- The `/r31415926` section of the agent's `config.toml` must be updated to connect to your node. In the examples above, we need to set the endpoint of our rootnet node to be `127.0.0.1:1234`, and replace the `auth_token` and `account` with the ones provided by our node. -->

<!-- *Example*: -->
<!-- ```toml -->
<!-- [[subnets]] -->
<!-- id = "/r31415926" -->

<!-- [subnets.config] -->
<!-- network_type = "fvm" -->
<!-- accounts = ["t1cp4q4lqsdhob23ysywffg2tvbmar5cshia4rweq"] -->
<!-- auth_token = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJBbGxvdyI6WyJyZWFkIiwid3JpdGUiLCJzaWduIiwiYWRtaW4iXX0.j94YYOr8_AWhGGHQd0q8JuQVuNhJA017SK9EUkqDOO0" -->
<!-- gateway_addr = "t064" -->
<!-- jsonrpc_api_http = "http://127.0.0.1:1234/rpc/v1" -->
<!-- ``` -->

<!-- > 💡 In the current implementation of subnets, the gateway is always deployed in the `t064` address, but this may change in the future. For Calibration as the parent, the gateay address is provided above. -->

<!-- > 💡 If you are already running the daemon, then run `./bin/ipc-agent config reload` to pick up the config changes. -->

<!-- ## Running -->
<!-- The IPC agent runs as a foreground daemon process that spawns a new JSON RPC server to interact with it, and all the processes to automatically handle checkpoints and the execution of cross-net messages for the subnets our agent is participating in. The agent determines the list of subnets it should interact with from its config file. -->

<!-- Alternatively, the agent can also be used as a CLI to interact with IPC. Under the hood, this cli sends new commands to the RPC server of the daemon. To run the IPC agent daemon you can run: -->
<!-- ```bash -->
<!-- ./bin/ipc-agent daemon -->
<!-- ``` -->

<!-- The RPC server of the daemon will be listening to the endpoint determined in the `json_rpc_address` field of the config. If you are looking for your agent to be accessible from Docker or externally, remember to listen on `0.0.0.0` instead of `127.0.0.1` as specified in the default config. -->

<!-- To check if the agent has connected to the rootnet successfully, you can try using it to create a new wallet. -->

<!-- *Example*: -->
<!-- ```console -->
<!-- $ ./bin/ipc-agent wallet new -w fvm --key-type bls -->
<!-- 2023-03-30T12:01:11Z INFO  ipc_agent::cli::commands::manager::wallet] created new wallet with address WalletNewResponse { address: "t3u7djutz4kwshntg4abams37ssy63irkfykqimodh4fs7krdst3y5qwcptvexmvic6gs5q6qygerminm2r3la" } in subnet "/r31415926" -->
<!-- ``` -->

## Help

If you meet any obstacles, please check [docs/troubleshooting.md](docs/troubleshooting.md) or join us in **#ipc-help** in the [Filecoin Slack workspace](https://filecoin.io/slack).
