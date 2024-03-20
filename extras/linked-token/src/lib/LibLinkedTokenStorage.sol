// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity ^0.8.23;
import {IERC20} from "openzeppelin-contracts/token/ERC20/IERC20.sol";
import {SubnetID} from "@ipc/src/structs/Subnet.sol";
import {IpcEnvelope} from "@ipc/src/structs/CrossNet.sol";

struct LinkedTokenStorage {
    IERC20 _underlying;
    address _gatewayAddr;
    SubnetID _linkedSubnet;

    address _linkedContract;

    mapping(bytes32 => UnconfirmedTransfer) _unconfirmedTransfers;
    mapping(bytes32 => IpcEnvelope) inflightMsgs;

}

struct UnconfirmedTransfer {
    address sender;
    uint256 value;
}

