// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.7;

import "./Subnet.sol";

struct Checkpoint {
    CheckData data;
    bytes signature;
}

struct CheckData {
    SubnetID source;
    bytes tipSet;
    uint64 epoch;
    bytes prevCheck;
    ChildCheck[] children;
    CrossMsgMeta crossMsgs;
}

struct ChildCheck {
    SubnetID source;
    bytes[] checks;
}

struct CrossMsgMeta {
    CrossMsg msgsCid;
    uint256 nonce;
    uint256 value;
    uint256 fee;
}

struct CrossMsg {
    StorableMsg message;
    bool wrapped;
}

struct CrossMsgs {
    CrossMsg[] msgs;
}

struct StorableMsg {
    IPCAddress from;
    IPCAddress to;
    uint256 value;
    uint256 nonce;
    uint64 method;
    bytes params;
}

struct IPCAddress {
    SubnetID subnetId;
    address rawAddress;
}
