// SPDX-License-Identifier: UNLICENSED
pragma solidity 0.8.18;

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
        checkpoint.epoch = 10;

        require(
            BottomUpCheckpoint({
                source: SubnetID(0, new address[](0)),
                epoch: 10,
                crossMsgs: new CrossMsg[](0),
                fee: 0,
                prevHash: EMPTY_HASH,
                children: new ChildCheck[](0)
            }).toHash() == checkpoint.toHash()
        );
    }

    function test_ToHash_Works_TopDownCheckpoint() public {
        topDownCheckpoint.epoch = 10;
        require(TopDownCheckpoint({epoch: 10, topDownMsgs: new CrossMsg[](0)}).toHash() == topDownCheckpoint.toHash());
    }

    function test_Sorted_SingleElement() public {
        crossMsg.message.nonce = 10;
        checkpoint.crossMsgs.push(crossMsg);
        require(isSorted(checkpoint));
    }

    function test_Sorted_True() public {
        crossMsg.message.nonce = 10;
        checkpoint.crossMsgs.push(crossMsg);
        crossMsg.message.nonce = 20;
        checkpoint.crossMsgs.push(crossMsg);
        crossMsg.message.nonce = 30;
        checkpoint.crossMsgs.push(crossMsg);
        require(isSorted(checkpoint));
    }

    function test_Sorted_False() public {
        crossMsg.message.nonce = 10;
        checkpoint.crossMsgs.push(crossMsg);
        crossMsg.message.nonce = 20;
        checkpoint.crossMsgs.push(crossMsg);
        crossMsg.message.nonce = 10;
        checkpoint.crossMsgs.push(crossMsg);
        require(isSorted(checkpoint) == false);
    }

    function isSorted(BottomUpCheckpoint memory _checkpoint) public pure returns (bool) {
        if (_checkpoint.crossMsgs.length < 2) return true;
        for (uint256 i = 1; i < _checkpoint.crossMsgs.length;) {
            if (_checkpoint.crossMsgs[i].message.nonce <= _checkpoint.crossMsgs[i - 1].message.nonce) return false;

            unchecked {
                ++i;
            }
        }
        return true;
    }
}
