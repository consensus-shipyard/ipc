# Hoku development

> Start a three-node network for remote or local development.

<!-- omit from toc -->

## Table of Contents

- [Background](#background)
- [Usage](#usage)
  - [Build dependencies](#build-dependencies)
    - [Linux](#linux)
    - [macOS](#macos)
  - [Environment variables](#environment-variables)
  - [Deploying \& stopping a subnet](#deploying--stopping-a-subnet)
    - [Testnet](#testnet)
    - [Localnet](#localnet)
  - [Logging](#logging)
    - [Deployment](#deployment)
    - [Validators \& network](#validators--network)
- [Development](#development)

## Background

These scripts handle deploying Hoku subnets. There are two target environments:

- Testnet: Deploy to a live network (Filecoin Calibration) as a fresh instance or attach to an
  existing one (i.e., pre-existing registry and gateway contracts).
- Localnet: Deploy to a local network as a fresh instance.

The scripts are designed to work for either a Linux or macOS machine. Other operating systems are
not supported.

## Usage

Regardless of the target environment, a three node network is started with a single entrypoint
script. All the dependencies and Dockerized node startup logic are handled in the `deploy.sh`
script, which you can pass a `localnet` argument to run a localnet—or run `make run-localnet` from
the root of this repo. Before getting started, you'll need to install the build dependencies for
your OS.

### Build dependencies

You'll need the following dependencies installed on your machine:

- `rustup`
- `cargo make`
- `toml-cli`
- `foundry`
- `node`
- `docker`
- `jq`

When you run `deploy.sh`, it _will_ check for the existence of these dependencies. For Linux
machines, the script _will also_ handle installation for all build dependencies. For macOS, it _will
not_ automatically install them. Instead, you'll have to do this manually, but the script _will_ log
and let you know what you're missing before proceeding. The section below outlines how to do this
for macOS.

Optionally, you can pass the `SKIP_DEPENDENCIES=true` environment variable to `deploy.sh` to skip
this entirely (e.g., if you already have these installed).

Also optional is the `hoku` CLI, which (if installed) will pre-buy credits for all accounts in the
localnet subnet setup. Follow the instructions in the `rust-hoku` repo to install it:
[here](https://github.com/hokunet/rust-hoku).

#### Linux

The `deploy.sh` will handle all the build dependencies for a Linux machine.

#### macOS

You'll need to install the following dependencies if they are not already available:

- Xcode from App Store or terminal: `xcode-select --install`
- Homebrew: Required for installing `jq` and possibly other dependencies. See the official Homebrew
  docs [here](https://brew.sh/) (but it's probably already installed).
- `docker`: Required to run the Dockerized nodes. See the official Docker docs
  [here](https://docs.docker.com/desktop/install/mac-install).
- `jq`: Needed for much of the JSON parsing logic when working with configuration files.
  ```shell
  brew install jq
  ```
- `rustup`: Required for building the Docker images (i.e., most of the stack is written in Rust).
  See the official Rust docs [here](https://www.rust-lang.org/tools/install) and make sure `cargo`
  gets installed.
  ```shell
  curl https://sh.rustup.rs -sSf | sh -s -- -y
  ```
- `cargo make`: Required for building the Docker images.
  ```shell
  cargo install cargo-make
  ```
- `toml-cli`: Needed for reading and writing various configuration files used by Docker images.
  ```shell
  cargo install toml-cli
  ```
- `foundryup`: Used for various onchain operations. See the official Foundry docs
  [here](https://book.getfoundry.sh/getting-started/installation) and make sure `cast` gets
  installed.
  ```shell
  curl -L https://foundry.paradigm.xyz | bash
  foundryup
  ```
- Node.js: Needed for compiling and deploying contracts with Hardhat. See the official Node.js docs
  [here](https://nodejs.org/en/download)—or use [nvm](https://github.com/nvm-sh/nvm) (described
  below for `bash` shells).
  ```shell
  curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.3/install.sh | bash
  source "$HOME/.bashrc" # This step will vary by OS
  nvm install --default lts/*
  ```

### Environment variables

All logic handled in the `deploy.sh` script uses the following environment variables, which are
defined in the `env.example` file. They are all optional but, for example, dictate what contracts
are deployed during the deployment flow:

- `IPC_FOLDER`: The path to the IPC folder (defaults to `${HOME}/ipc`). Not used for localnet
  deployments.
- `PARENT_GATEWAY_ADDRESS`: The EVM address of the parent gateway contract (default defined in
  `.ipc-cal/config.toml` or `.ipc-local/config.toml`).
- `PARENT_REGISTRY_ADDRESS`: The EVM address of the parent registry contract (default defined in
  `.ipc-cal/config.toml` or `.ipc-local/config.toml`).
- `PARENT_HTTP_AUTH_TOKEN`: An auth token for RPC calls to a [Glif.io](https://api.node.glif.io/)
  archive node for the rootnet (Filecoin Calibration)—only used if you are deploying to testnet.
- `SUPPLY_SOURCE_ADDRESS`: The address of the supply source (ERC20) for all deployed subnets.
- `FM_LOG_LEVEL`: Fendermint log level. One of `off`, `error`, `warn`, `info`, `debug`, or `trace`
  (default is `info`).

For localnet deployments, you won't need to set any of the above. There are also two additional
optional variables:

- `SKIP_DEPENDENCIES`: Skips the installation of build dependencies (see
  [Build dependencies](#build-dependencies)).
- `SKIP_BUILD`: Skips the build step for the entire stack. You **MUST** run this at least once
  before starting the network. Notably, this also installs the `ipc-cli` binary that is used heavily
  during the deployment script. After doing so, future `deploy.sh` invocations will use the existing
  build artifacts, so you can set `SKIP_BUILD=true` to save time.

Create a `.env` file with your desired values and source it in your shell:

```shell
source .env
```

Lastly, be sure to source your shell's rc file. This step will vary by OS. For example, `bash`:

```shell
source ${HOME}/.bashrc
```

### Deploying & stopping a subnet

All scripts use `cargo make` to start docker containers, volumes, and a docker network. You can use
`docker ps` to check the status of each container. Also, you **must** run the scripts from the root
of the repo!

#### Testnet

Testnet deployments will create a subnet with Filecoin Calibration as the rootnet (parent). The
`deploy.sh` script will create new validator private keys and a `genesis.json` config, and move them
to the config folder (defaults to `~/.ipc`).

```shell
./scripts/deploy_subnet/deploy.sh
```

By default, the script will deploy the latest `develop` branch. If you want to deploy a specific
branch, you can pass the branch name as an argument:

```shell
./scripts/deploy_subnet/deploy.sh <branch-name>
```

#### Localnet

A localnet deployments will create a subnet with `anvil` as the rootnet (parent). The scripts will
handle localnet validator keys (using the standard `anvil` accounts) and configs in the config
folder (defaults to `~/.ipc`). In the root of the `ipc` repo, you can run the localnet with the
following `Makefile` command:

```shell
make run-localnet
```

Alternatively, the `deploy.sh` can be ran directly. The key callout here is that you **must**
specify `localnet` (or `local`) as the argument, and it's _not_ possible to pass a specific branch
as an argument (i.e., it uses whatever branch is currently checked out in the local repo).

```shell
./scripts/deploy_subnet/deploy.sh localnet
```

If you want to, for example, skip the build and dependency installation steps, you can do so with:

```shell
SKIP_BUILD=true SKIP_DEPENDENCIES=true ./scripts/deploy_subnet/deploy.sh localnet
```

Lastly, if you're ready to stop the network, you can run `stop_local.sh`:

```shell
./scripts/deploy_subnet/stop_local.sh
```

The following outlines general observations for how long the localnet deployment process takes and
various metrics:

- Deploy + build images: ~7 minutes
- Deploy with prebuilt images: ~5 minutes (i.e., `SKIP_BUILD=true`)
- Blocks: ~1 per second
- Topdown messages: ~2 minutes (e.g., depositing funds from the rootnet)
- Bottomup messages: ~15 seconds
- Stopping the network: ~40–45 seconds

### Logging

#### Deployment

Deploying a network will log various steps and summary information to the console, including the RPC
URLs and contracts. If you're running a localnet, it'll also show the available accounts, private
keys, and respective balances.

#### Validators & network

Use `docker ps` to list the network's containers. Each validator has six containers, and there are a
few others used across the full network.

You can check a validator's logs with `docker logs <container-name>`. The following containers are
created (as shown for validator `0`), and you can replace the `0` with `1` or `2` to inspect the
other validators:

- `validator-0-fendermint`
- `validator-0-cometbft`
- `validator-0-promtail`
- `validator-0-objects`
- `validator-0-ethapi`
- `validator-0-iroh`
- `prometheus`
- `grafana`
- `anvil`
- `loki`

## Development

You can test using the subnet with the [`hoku` SDK & CLI](https://github.com/hokunet/rust-hoku).
Keys _are not_ logged if you're running a testnet. For localnet, keys _are_ logged with their
corresponding balances. You'll notice the first three accounts correspond to the validators and
marked as reserved. If you're trying to do non-validator actions (e.g., create a bucket or timehub),
it's best to avoid these accounts since nonce race conditions can occur.

```txt
Account balances:
Parent native: 9999 ETH
Parent HOKU:   100 HOKU
Subnet native: 5000 HOKU
Subnet credits: 5000000000000000000000

Accounts:
(0) 0xf39fd6e51aad88f6f4ce6ab8827279cfffb92266 (reserved)
(1) 0x70997970c51812dc3a010c7d01b50e0d17dc79c8 (reserved)
(2) 0x3c44cdddb6a900fa2b585dd299e03d12fa4293bc (reserved)
(3) 0x90f79bf6eb2c4f870365e785982e1f101e93b906 (available)
...

Private keys:
(0) ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80
(1) 59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d
(2) 5de4111afa1a4b94908f83103eb1f1706367c2e68ca870fc3fb9a804cdab365a
(3) 7c852118294e51e653712a81e05800f419141751be58f605c371e15141b007a6
...
```

You can use then these keys with the `hoku` SDK and CLI by creating an `.env` file and sourcing it,
or by setting the variables in your shell. Keep in mind a `NETWORK` variable is used by `fendermint`
and `ipc-cli`, and these differ from the `HOKU_NETWORK` value used by the CLI.

```dotenv
export HOKU_NETWORK=localnet
export HOKU_PRIVATE_KEY=<private_key>
```

You can now use `hoku` as normal, e.g., `hoku account deposit`, `hoku os create`, etc. Similarly,
the SDK lets you use the `localnet` by explicitly initializing it with
`hoku_sdk::network::Network::Localnet.init()`.
