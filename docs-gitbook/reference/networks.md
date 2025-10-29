---
description: Test networks are available for deployment of IPC subnets.
---

# Networks

## IPC Chain Testnet

The IPC Chain Testnet (also known as "IPC Cal") is a public testnet specifically designed for testing IPC features and subnet deployments. This network provides a dedicated environment for developers to experiment with IPC functionality.

The params to connect to IPC Chain Testnet are:

* **Network Name**: IPC Cal
* **RPC URL**: `http://node-1.test.ipc.space:8545`
* **Chain ID**: `385401`
* **Currency Symbol**: `tFIL`

For a step-by-step guide on connecting to this network with MetaMask, see [Connecting to IPC Chain Testnet](../user-guides/connect-to-testnet.md).

## Filecoin Calibration Network

The [Filecoin Calibration network](https://docs.filecoin.io/networks/calibration) is the most realistic testnet simulation of the Filecoin mainnet. As the largest decentralized storage network, applications that require data storage and retrieval can leverage Filecoin's storage capabilities by connecting an IPC subnet to the Calibration net as the parent network.

The params to connect to Filecoin Calibration are:

* subnet\_id : `/r314159`
* provider\_http : `https://api.calibration.node.glif.io/rpc/v1`
* gateway\_addr :  [![Gateway Address](https://img.shields.io/badge/dynamic/json?url=https%3A%2F%2Fraw.githubusercontent.com%2Fconsensus-shipyard%2Fipc%2Fcd%2Fcontracts%2Fdeployments%2Fr314159.json&query=%24.gateway_addr&label=Gateway%20Address)](https://github.com/consensus-shipyard/ipc/blob/cd/contracts/deployments/r314159.json)
* registry\_addr : [![Registry Address](https://img.shields.io/badge/dynamic/json?url=https%3A%2F%2Fraw.githubusercontent.com%2Fconsensus-shipyard%2Fipc%2Fcd%2Fcontracts%2Fdeployments%2Fr314159.json&query=%24.registry_addr&label=Registry%20Address)](https://github.com/consensus-shipyard/ipc/blob/cd/contracts/deployments/r314159.json)

The Chain ID for Filecoin Calibration is `314159`. A [faucet](https://faucet.calibration.fildev.network/) and [explorer](https://calibration.filfox.info/en) are available.
