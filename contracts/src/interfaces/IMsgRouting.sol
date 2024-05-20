// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity ^0.8.23;

import {BottomUpCheckpoint, BottomUpMsgBatch, IpcEnvelope, ParentFinality} from "../structs/CrossNet.sol";
import {SubnetID} from "../structs/Subnet.sol";
import {FvmAddress} from "../structs/FvmAddress.sol";

/// @title Gateway message routing interface
interface IMsgRouting {
    /// @notice Route a topdown message to the target network
    function routeTopdownMsg(IpcEnvelope calldata envelope) external payable;

    /// @notice Route a bottom up message to the target network
    function routeBottomUpMsg(IpcEnvelope calldata envelope) external payable returns (IpcEnvelope memory committed);
}
