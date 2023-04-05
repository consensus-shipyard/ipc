// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.7;

import "./Subnet.sol";

/// @title Checkpoint struct and related structs
/// @author LimeChain team
struct Checkpoint {
    CheckData data;
    bytes signature;
}

struct CheckData {
    SubnetID source;
    bytes tipSet;
    int64 epoch;
    bytes32 prevHash;
    ChildCheck[] children;
    CrossMsgMeta crossMsgs;
}

struct ChildCheck {
    SubnetID source;
    bytes32[] checks;
}

struct CrossMsgMeta {
    bytes32 msgsHash;
    uint64 nonce;
    uint256 value;
    uint256 fee;
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
    uint64 method;
    bytes params;
}

struct IPCAddress {
    SubnetID subnetId;
    address rawAddress;
}
