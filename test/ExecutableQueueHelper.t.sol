// SPDX-License-Identifier: UNLICENSED
pragma solidity 0.8.18;

import "forge-std/Test.sol";

import "../src/lib/ExecutableQueueHelper.sol";
import "../src/structs/ExecutableQueue.sol";

contract ExecutableQueueHelperTest is Test {
    using ExecutableQueueHelper for ExecutableQueue;

    ExecutableQueue private queue;

    uint64 constant BLOCKS_PER_EPOCH = 10;
    uint64 constant EPOCH_ONE = 10;
    uint64 constant EPOCH_TWO = 20;
    uint64 constant EPOCH_THREE = 30;
    uint64 constant EPOCH_FOUR = 40;

    function setUp() public {
        queue.period = BLOCKS_PER_EPOCH;
    }

    function test_Push_Works_ZeroEpoch() public {
        queue.push(0);

        require(queue.epochs[0] == false);
    }

    function test_Push_Works_OneEpoch() public {
        _assertPush(EPOCH_ONE);

        require(queue.first == EPOCH_ONE);
        require(queue.last == EPOCH_ONE);
    }

    function test_Push_Works_NewFirst() public {
        _assertPush(EPOCH_TWO);

        require(queue.first == EPOCH_TWO);
        require(queue.last == EPOCH_TWO);

        _assertPush(EPOCH_ONE);

        require(queue.first == EPOCH_ONE);
        require(queue.last == EPOCH_TWO);
    }

    function test_Push_Works_NewLast() public {
        _assertPush(EPOCH_ONE);

        require(queue.first == EPOCH_ONE);
        require(queue.last == EPOCH_ONE);

        _assertPush(EPOCH_TWO);

        require(queue.first == EPOCH_ONE);
        require(queue.last == EPOCH_TWO);
    }

    function test_Remove_Works_FirstEpoch() public {
        _assertPush(EPOCH_ONE);
        _assertPush(EPOCH_THREE);
        _assertPush(EPOCH_FOUR);

        _assertRemove(EPOCH_ONE);

        require(queue.first == EPOCH_THREE);
        require(queue.last == EPOCH_FOUR);
    }

    function test_Remove_Works_LastEpoch() public {
        _assertPush(EPOCH_ONE);
        _assertPush(EPOCH_FOUR);

        _assertRemove(EPOCH_FOUR);

        require(queue.first == EPOCH_ONE);
        require(queue.last == EPOCH_ONE);
    }

    function test_Remove_Works_MiddleEpoch() public {
        _assertPush(EPOCH_ONE);
        _assertPush(EPOCH_TWO);
        _assertPush(EPOCH_THREE);

        require(queue.first == EPOCH_ONE);
        require(queue.last == EPOCH_THREE);

        _assertRemove(EPOCH_TWO);

        require(queue.first == EPOCH_ONE);
        require(queue.last == EPOCH_THREE);
    }

    function test_Remove_Works_NonExistingEpoch() public {
        _assertRemove(EPOCH_TWO);

        require(queue.first == 0);
        require(queue.last == 0);
    }

    function _assertPush(uint64 epoch) private {
        queue.push(epoch);

        require(queue.epochs[epoch] == true);
    }

    function _assertRemove(uint64 epoch) private {
        queue.remove(epoch);

        require(queue.epochs[epoch] == false);
    }
}
