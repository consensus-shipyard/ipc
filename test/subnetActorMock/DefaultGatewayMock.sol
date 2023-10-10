// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity 0.8.19;

import {IGateway} from "../../src/interfaces/IGateway.sol";
import {BottomUpCheckpoint, CrossMsg, ParentFinality} from "../../src/structs/Checkpoint.sol";
import {SubnetID} from "../../src/structs/Subnet.sol";
import {FvmAddress} from "../../src/structs/FvmAddress.sol";

contract DefaultGatewayMock is IGateway {
    uint8 private dummy;

    function register() external payable {
        // silent warning
        msg.value;

        // make method perform txn
        dummy = 1;
    }

    function addStake() external payable {
        // silent warning
        msg.value;
        // make method perform txn
        dummy = 1;
    }

    function releaseStake(uint256 amount) external {
        // silent warning
        amount;
        // make method perform txn
        dummy = 1;
    }

    function releaseRewardForRelayer(uint256 amount) external {
        // silent warning
        amount;
        // make method perform txn
        dummy = 1;
    }

    function kill() external {
        // silent warning
        msg.sender;
        // make method perform txn
        dummy = 1;
    }

    /// CommitChildCheck propagates the commitment of a checkpoint from a child subnet,
    /// process the cross-messages directed to the subnet.
    function commitBottomUpCheckpoint(BottomUpCheckpoint calldata bottomUpCheckpoint, CrossMsg[] calldata messages) external {
        // silent warning
        bottomUpCheckpoint;
        messages;
        // make method perform txn
        dummy = 1;
    }

    function fund(SubnetID calldata subnetId, FvmAddress calldata to) external payable {
        // silent warning
        subnetId;
        to;
        // make method perform txn
        dummy = 1;
    }

    function release(FvmAddress calldata to) external payable {
        // silent warning
        to;
        // make method perform txn
        dummy = 1;
    }

    function sendCrossMessage(CrossMsg memory crossMsg) external payable {
        // silent warning
        crossMsg;
        // make method perform txn
        dummy = 1;
    }

    function propagate(bytes32 msgCid) external payable {
        // silent warning
        msgCid;
        // make method perform txn
        dummy = 1;
    }

    function commitParentFinality(
        ParentFinality calldata finality
    ) external {
        // silent warning
        finality;
        // make method perform txn
        dummy = 1;
    }

    function createBottomUpCheckpoint(
        BottomUpCheckpoint calldata checkpoint,
        bytes32 membershipRootHash,
        uint256 membershipWeight
    ) external {
        // silent warning
        checkpoint;
        membershipRootHash;
        membershipWeight;
        // make method perform txn
        dummy = 1;
    }
}