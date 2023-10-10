// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity 0.8.19;

import "forge-std/Test.sol";

import "../src/structs/Checkpoint.sol";
import "../src/lib/CheckpointHelper.sol";

contract CheckpointHelperTest is Test {
    using CheckpointHelper for BottomUpCheckpoint;

    BottomUpCheckpoint public checkpoint;
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
}
