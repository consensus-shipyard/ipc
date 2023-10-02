// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity 0.8.19;

import "forge-std/Test.sol";

import "../src/structs/Checkpoint.sol";
import "../src/lib/CheckpointHelper.sol";

contract CheckpointHelperTest is Test {
    using CheckpointHelper for BottomUpCheckpoint;
    using CheckpointHelper for TopDownCheckpoint;

    BottomUpCheckpoint public checkpoint;
    TopDownCheckpoint public topDownCheckpoint;
    CrossMsg public crossMsg;

    function test_ToHash_Works_BottomUpCheckpoint() public {
        checkpoint.blockHeight = 10;

        require(
            BottomUpCheckpoint({
                subnetID: SubnetID(0, new address[](0)),
                blockHeight: 10,
                crossMessagesHash: 0,
                blockHash: 0,
                nextConfigurationNumber: 0
            }).toHash() == checkpoint.toHash()
        );
    }

    function test_ToHash_Works_TopDownCheckpoint() public {
        topDownCheckpoint.epoch = 10;
        require(TopDownCheckpoint({epoch: 10, topDownMsgs: new CrossMsg[](0)}).toHash() == topDownCheckpoint.toHash());
    }
}
