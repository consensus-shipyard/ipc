---
description: Test networks are available for deployment of IPC subnets.
---

# Networks

### Filecoin Calibration Network

The [Filecoin Calibration network](https://docs.filecoin.io/networks/calibration) is the most realistic testnet simulation of the Filecoin mainnet. As the largest decentralized storage network, applications that require data storage and retrieval can leverage Filecoin's storage capabilities by connecting an IPC subnet to the Calibration net as the parent network.

The params to connect to Filecoin Calibration are:

* subnet\_id : `/r314159`
* provider\_http : `https://api.calibration.node.glif.io/rpc/v1`
* gateway\_addr : `0x1AEe8A878a22280fc2753b3C63571C8F895D2FE3`
* registry\_addr : `0x0b4e239FF21b40120cDa817fba77bD1B366c1bcD`

The Chain ID for Filecoin Calibration is `314159`. A [faucet](https://faucet.calibration.fildev.network/) and [explorer](https://calibration.filfox.info/en) are available.
