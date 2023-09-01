// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity 0.8.19;

import {SubnetID} from "./Subnet.sol";
import {FvmAddress} from "./FvmAddress.sol";
import {BottomUpCheckpoint, CrossMsg} from "./Checkpoint.sol";
import {Status} from "../enums/Status.sol";

struct SubnetID {
    /// @notice chainID of the root subnet
    uint64 root;
    /// @notice parent path of the subnet
    address[] route;
}

struct Subnet {
    uint256 stake;
    uint256 genesisEpoch;
    uint256 circSupply;
    uint64 topDownNonce;
    uint64 appliedBottomUpNonce;
    Status status;
    SubnetID id;
    BottomUpCheckpoint prevCheckpoint;
}

struct IPCAddress {
    SubnetID subnetId;
    FvmAddress rawAddress;
}
