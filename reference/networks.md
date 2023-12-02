---
description: Test networks are available for deployment of IPC subnets.
---

# Networks

### Mycelium Calibration Network

Mycelium Calibration is a publicly available IPC subnet deployed on Filecoin Calibration as the rootnet. It's a test network that offers **short block time and fast finality** for builders to deploy smart contracts and build dApps.

The params to connect to Mycelium Calibration are:

* subnet\_id : `/r314159/t410f3psqt4olacthtwmksoxapz2r2hbsohs6kxe636y`
* provider\_http : `https://api.mycelium.calibration.node.glif.io/`
* gateway\_addr : `0x77aa40b105843728088c0132e43fc44348881da8`
* registry\_addr : `0x74539671a1d2f1c8f200826baba665179f53a1b7`

The Chain ID for Mycelium Calibration is `3875628474663538`. A [faucet](https://faucet.mycelium.calibration.node.glif.io/) and [explorer](https://explorer.mycelium.calibration.node.glif.io/) are available.

### Filecoin Calibration Network

The [Filecoin Calibration network](https://docs.filecoin.io/networks/calibration) is the most realistic testnet simulation of the Filecoin mainnet. As the largest decentralized storage network, applications that require data storage and retrieval can leverage Filecoin's storage capabilities by connecting an IPC subnet to the Calibration net as the parent network.

The params to connect to Filecoin Calibration are:

* subnet\_id : `/r314159`
* provider\_http : `https://api.calibration.node.glif.io/rpc/v1`
* gateway\_addr : `0x2e994B75095a39EB7C6dDA0516731B02AbcDbdb4`
* registry\_addr : `0x5FE1a06cA4534cB878260522a572e2254214261D`

The Chain ID for Filecoin Calibration is `314159`. A [faucet](https://faucet.calibration.fildev.network/) and [explorer](https://calibration.filfox.info/en) are available.
