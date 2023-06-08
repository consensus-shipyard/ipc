// SPDX-License-Identifier: UNLICENSED
pragma solidity 0.8.18;

import "./Subnet.sol";

/// @title BottomUpCheckpoint struct
/// @author LimeChain team
struct BottomUpCheckpoint {
    SubnetID source;
    uint64 epoch;
    uint256 fee;
    CrossMsg[] crossMsgs;
    ChildCheck[] children;
    bytes32 prevHash;
}

struct TopDownCheckpoint {
    uint64 epoch;
    CrossMsg[] topDownMsgs;
}

struct ChildCheck {
    SubnetID source;
    bytes32[] checks;
}

struct CrossMsg {
    StorableMsg message;
    bool wrapped;
}

struct StorableMsg {
    IPCAddress from;
    IPCAddress to;
    uint256 value;
    uint64 nonce;
    bytes4 method;
    bytes params;
}

struct IPCAddress {
    SubnetID subnetId;
    address rawAddress;
}
