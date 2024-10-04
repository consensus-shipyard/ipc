# Scripts

> Scripts for deploying a local or remote network.

<!-- omit from toc -->
## Table of Contents

- [Background](#background)
- [Usage](#usage)
  - [Build dependencies](#build-dependencies)
    - [CometBFT](#cometbft)
    - [Fendermint](#fendermint)
  - [Running the devnet](#running-the-devnet)
    - [Setup](#setup)
  - [Starting the network](#starting-the-network)
- [Development](#development)

## Background

There are three ways to use these scripts:

1. Run a local network with three nodes and Anvil as the root chain (`localnet`).
2. Run a remote network with three nodes and Filecoin Calibration as the root chain (`testnet`).
3. Run a local network without any root chain (`devnet`).

Option (1) and (2) leverage Dockerized instances of Iroh, Fendermint, CometBFT, and ancillary
services. All of this logic is located in the `deploy_subnet` directory and described
[here](./deploy_subnet/README.md) in more detail. Option (3) requires you to have various dependencies
running directly on your host machine, so there's a bit of manual setup required.

The sections below outline the `devnet` setup.

## Usage

When developing locally, it's ideal to use the `localnet` setup in the `deploy_subnet` directory. But
`devnet` is much faster to set up and teardown since it doesn't require building Docker images or
running any networked services, and instead, relies on installed binaries.

### Build dependencies

The following assumes you're working in the root of this `ipc` repo, and **not** in this `scripts`
directory.

#### CometBFT

Navigate outside this repo and install CometBFT (note: you **must** install v0.37.1 since the
latest version is not compatible with the rest of the codebase):

```
git clone https://github.com/cometbft/cometbft.git
cd cometbft
git checkout v0.37.1
make install
```

#### Fendermint

The default `make` task will build the contracts, libraries, and CLIs needed to run a `devnet`.

```
make
```

### Running the devnet

#### Setup

First, configure the `devnet` with the following, which creates some keys and alters the configuration
located in `~/.fendermint`:

```
make config-devnet
```

The keys used here are a bit different from how the `localnet` setup works. The `localnet` relies on the
`ipc-cli` for much of its logic, whereas the `devnet` relies primarily on the `fendermint` CLI.

The keys you'll use can be found in the `test-network/keys` directory within this `scripts` folder.
Only the `alice` key is deterministically coded into the Fendermint startup process. You can export
any of the keys to a hex encoded EVM format.

From the root of this repo, run the following command, replacing `alice.sk` with the corresponding
filename and `alice_eth` with a desired name (for files that get created later):

```
fendermint key into-eth --secret-key ./scripts/test-network/keys/alice.sk --name alice_eth
```

Since the `alice.sk` will always be the same, the following is consistent across machines and can be
used for `devnet` interactions (e.g., creating buckets, accumulators, writing data, etc.):

- EVM address: `0xc05fe6b63ffa4b3c518e6ff1e597358ee839db01`
- Hex private key: `1c323d494d1d069fe4c891350a1ec691c4216c17418a0cb3c7533b143bd2b812`

### Starting the network

Finally, start all the services in separate terminal windows. The associated endpoints are shown
below:

- Iroh: `http://127.0.0.1:4919`

  ```
  make run-devnet-iroh
  ```

- Objects API: `http://127.0.0.8001`

  ```
  make run-devnet-objects
  ```

- Fendermint: `http://127.0.0.1:26658`

  ```
  make run-devnet-fendermint
  ```

- CometBFT: `http://127.0.0.1:26657`

  ```
  make run-devnet-cometbft
  ```

- EVM RPC: `http://127.0.0.1:8545` (this is optional)

  ```
  make run-devnet-evm
  ```

Once these are running, you should be able to interact with the `devnet` and _most_ of the expected
functionality. One thing to be aware of: there's no parent (rootnet), so certain pieces that involve
parent-child actions will not work. For example, the `hoku` CLI's `account` commands will not work
since these read from or write to the parent's state.

## Development

If you _ever_ make changes to any of this codebase—particularly, pieces that `fendermint` uses,
you'll need to make sure you rebuild everything and install via the `make install` command. Then,
follow the steps outlined above to get the `devnet` up and running.

If you're using the `rust-hoku` CLI, you must set the `NETWORK` environment variable to `devnet` in
order to use it. Note that `fendermint`, `ipc-cli`, and `hoku` use the same `NETWORK` flag but with
varying values, so be sure to open a new terminal window to avoid conflicts if you're using any of
them simultaneously.

```
export NETWORK=devnet
export PRIVATE_KEY=1c323d494d1d069fe4c891350a1ec691c4216c17418a0cb3c7533b143bd2b812
```

Similarly, you'll use the `devnet` in the SDK where necessary—e.g.,
`hoku_sdk::network::Network::Devnet.init()`.
