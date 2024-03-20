// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity ^0.8.23;

struct LinkedTokenStorage {
    IERC20 public _underlying;
    address public _gatewayAddr;
    SubnetID public _linkedSubnet;

    address public _linkedContract;

    mapping(bytes32 => UnconfirmedTransfer) public _unconfirmedTransfers;
    mapping(bytes32 => IpcEnvelope) public inflightMsgs;

}

struct UnconfirmedTransfer {
    address sender;
    uint256 value;
}

