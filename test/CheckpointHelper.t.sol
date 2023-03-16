// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.7;

import "forge-std/Test.sol";
import "forge-std/console.sol";

import "../src/structs/Checkpoint.sol";
import "../src/lib/CheckpointHelper.sol";

contract CheckpointHelperTest is Test {
    using CheckpointHelper for Checkpoint;

    Checkpoint public checkpoint;

    function test_ToHash_Works_EmptyCheckpoint() public view {
        require(
            CheckpointHelper.EMPTY_CHECKPOINT_DATA_HASH == checkpoint.toHash()
        );
    }

    function test_ToHash_Works_NonEmptyCheckpoint() public {
        checkpoint.data.epoch = 10;

        // checkpoint with epoch = 10
        bytes32 expected = 0xe13e6ce468f1108376290016582facd126f48f28ef238d0551f5b7a98d8ee334;

        require(expected == checkpoint.toHash());
    }

    function test_HasCrossMsgMeta_Works_True() public {
        checkpoint.data.crossMsgs.nonce = 1;
        checkpoint.data.crossMsgs.value = 1;

        require(checkpoint.hasCrossMsgMeta() == true);
    }

    function test_HasCrossMsgMeta_Works_False() public view {
        require(checkpoint.hasCrossMsgMeta() == false);
    }
}
