# Fendermint

Fendermint is an effort to implement [IPC with Tendermint Core](https://docs.google.com/document/d/1cFoTdoRuYgxmWJia6K-b5vmEj-4MvyHCNvShZpyconU/edit#). There is a preliminary [roadmap](https://docs.google.com/spreadsheets/d/1eVwkHEPGNg0js8DKRDIX7sugf5JqbI9zRBddIqzJFfI/edit#gid=0) that lays out the tasks towards implementing subnets that run IPLD and FVM under the Filecoin rootnet, sharing components with the Lotus/Eudico based implementation.

## Quick Start

- [Local testnets](../docs/fendermint/localnet.md)

## Docs

Please have a look in the [docs](../docs/fendermint/README.md) to see an overview of the project, how to run the components, and previous demos.

## IPC

Fendermint is built with support for [IPC](https://github.com/consensus-shipyard/ipc) by design. If you are looking to deploy the infrastructure Fendermint-based IPC subnet, refer to the [IPC main repo](https://github.com/consensus-shipyard/ipc), or have a look at the [IPC infrastructure docs](../docs/fendermint/ipc.md).

## Building from source

**Linux.** Install the following pre-requisites (instructions for Ubuntu):

- Install system packages: `sudo apt install build-essential clang cmake pkg-config libssl-dev protobuf-compiler`.
- Install Rust. See [instructions](https://www.rust-lang.org/tools/install).
- Install cargo-make: `cargo install --force cargo-make`.
- Install Foundry. See [instructions](https://book.getfoundry.sh/getting-started/installation).
- Install Docker (if you intend to build Docker images). See [instructions](https://docs.docker.com/engine/install/ubuntu/).

## Testing

The following command runs unit and integration tests:

```bash
make test
```

while the next command builds docker images and runs an end-to-end test using the
[SimpleCoin](./fendermint/rpc/examples/simplecoin.rs) and the
[ethers](./fendermint/eth/api/examples/ethers.rs) examples:

```bash
make e2e
```

## IPC Solidity Actors

We generate Rust bindings for the Solidity actors we need to invoke from the [ipc-solidity-actors](https://github.com/consensus-shipyard/ipc-solidity-actors) library, some of which are deployed during the genesis process. The bindings live in [ipc_actors](./fendermint/vm/ipc_actors/), and are generated automatically during the build, or with the following command:

```bash
make ipc-actors-abi
```

To run it, you will have to install [forge](https://book.getfoundry.sh/getting-started/installation).

The list of contracts for which we generate Rust bindings are in [build.rs](./fendermint/vm/ipc_actors/build.rs) and needs to be maintained by hand, for example if a new "diamond facet" is added to a contract, it has to be added here. Diamond facets also have to be added manually in [ipc.rs](./fendermint/vm/actor_interface/src/ipc.rs) where the contracts which need to be deployed during genesis are described. These facets cannot be divined from the ABI description, so they have to be maintained explicitly.

To test whether the genesis process works, we can run the following unit test:

```bash
cargo test --release -p fendermint_vm_interpreter load_genesis
```

## Pre-built Docker Image

The CI build publishes a [Docker image](https://github.com/consensus-shipyard/fendermint/pkgs/container/fendermint) to Github Container Registry upon a successful build on the `main` branch. This is the same image as the one used in the End-to-End tests; it contains the built-in actor bundle and IPC Solidity actors, ready to be deployed during genesis.

The image can be pulled with the following command:

```bash
docker pull ghcr.io/consensus-shipyard/fendermint:latest
```
