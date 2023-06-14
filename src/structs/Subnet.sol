// SPDX-License-Identifier: UNLICENSED
pragma solidity 0.8.18;

import "./Checkpoint.sol";
import "../enums/Status.sol";

/// @title Subnet id struct
/// @author LimeChain team
struct SubnetID {
    /// @notice chainID of the root subnet
    uint64 root;
    /// @notice parent path of the subnet
    address[] route;
}

struct Subnet {
    Status status;
    uint64 topDownNonce;
    uint64 appliedBottomUpNonce;
    uint256 stake;
    uint256 genesisEpoch;
    uint256 circSupply;
    SubnetID id;
    BottomUpCheckpoint prevCheckpoint;
    CrossMsg[] topDownMsgs;
}
