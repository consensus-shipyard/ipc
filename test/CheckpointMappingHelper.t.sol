// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.7;

import "forge-std/Test.sol";

import "../src/structs/Checkpoint.sol";
import "../src/structs/Subnet.sol";
import "../src/lib/SubnetIDHelper.sol";
import "../src/lib/CheckpointHelper.sol";
import "../src/lib/CheckpointMappingHelper.sol";

contract CheckpointMappingHelperTest is Test {
    using SubnetIDHelper for SubnetID;
    using CheckpointHelper for Checkpoint;
    using CheckpointMappingHelper for mapping(int64 => Checkpoint);

    int64 constant BLOCKS_PER_EPOCH = 10;

    int64 constant EPOCH_ONE = 0 * BLOCKS_PER_EPOCH;
    int64 constant EPOCH_TWO = 1 * BLOCKS_PER_EPOCH;
    int64 constant EPOCH_THREE = 2 * BLOCKS_PER_EPOCH;
    int64 constant NON_EXISTING_EPOCH = 100000;

    mapping(int64 => Checkpoint) public checkpoints;

    function test_GetPrevCheckpointHash_Works_NegativeEpoch() public {
        createCheckpoint(EPOCH_ONE, 5);

        bytes32 emptyHash = checkpoints.getPrevCheckpointHash(-100, -10);

        require(emptyHash == checkpoints[NON_EXISTING_EPOCH].toHash());
    }

    function test_GetPrevCheckpointHash_Works_Empty(
        int8 nonExistingEpoch
    ) public view {
        bytes32 emptyHash = checkpoints.getPrevCheckpointHash(
            nonExistingEpoch,
            BLOCKS_PER_EPOCH
        );

        require(emptyHash == checkpoints[nonExistingEpoch].toHash());
    }

    function test_GetPrevCheckpointHash_Works_EpochOne() public {
        createCheckpoint(EPOCH_ONE, 5);
        createCheckpoint(EPOCH_TWO, 15);

        bytes32 prevCheckpointHash = checkpoints.getPrevCheckpointHash(
            EPOCH_TWO,
            BLOCKS_PER_EPOCH
        );

        require(prevCheckpointHash == checkpoints[EPOCH_ONE].toHash());
    }

    function test_GetPrevCheckpointHash_Works_EpochTwo() public {
        createCheckpoint(EPOCH_ONE, 5);
        createCheckpoint(EPOCH_TWO, 15);
        createCheckpoint(EPOCH_THREE, 25);

        bytes32 prevCheckpointHash = checkpoints.getPrevCheckpointHash(
            EPOCH_THREE,
            BLOCKS_PER_EPOCH
        );

        require(prevCheckpointHash == checkpoints[EPOCH_TWO].toHash());
    }

    function test_GetCheckpointPerEpoch_Works_SameEpochNextBlock() public {
        createCheckpoint(EPOCH_ONE, 5);

        (bool exists, int epoch, Checkpoint storage checkpoint) = checkpoints
            .getCheckpointPerEpoch(8, BLOCKS_PER_EPOCH);

        require(exists);
        require(epoch == EPOCH_ONE);
        require(checkpoint.toHash() == checkpoints[EPOCH_ONE].toHash());
    }

    function test_GetCheckpointPerEpoch_Works_SameEpochPreviousBlock() public {
        createCheckpoint(EPOCH_ONE, 5);

        (bool exists, int epoch, Checkpoint storage checkpoint) = checkpoints
            .getCheckpointPerEpoch(3, BLOCKS_PER_EPOCH);

        require(exists);
        require(epoch == EPOCH_ONE);
        require(checkpoint.toHash() == checkpoints[EPOCH_ONE].toHash());
    }

    function test_GetCheckpointPerEpoch_Works_EpochTwo() public {
        createCheckpoint(EPOCH_ONE, 5);
        createCheckpoint(EPOCH_TWO, 12);

        (bool exists, int epoch, Checkpoint storage checkpoint) = checkpoints
            .getCheckpointPerEpoch(12, BLOCKS_PER_EPOCH);

        require(exists);
        require(epoch == EPOCH_TWO);
        require(checkpoint.toHash() == checkpoints[EPOCH_TWO].toHash());
    }

    function test_GetCheckpointPerEpoch_Works_FutureEpoch(
        int64 futureEpoch
    ) public {
        vm.assume(
            futureEpoch > EPOCH_THREE && futureEpoch % BLOCKS_PER_EPOCH == 0
        );

        createCheckpoint(EPOCH_ONE, 5);
        createCheckpoint(EPOCH_TWO, 15);

        for (
            uint nextBlock = 0;
            nextBlock < uint64(BLOCKS_PER_EPOCH);
            nextBlock++
        ) {
            uint256 futureBlockNumber = uint64(futureEpoch) + nextBlock;

            (bool exists, int64 epoch, ) = checkpoints.getCheckpointPerEpoch(
                futureBlockNumber,
                BLOCKS_PER_EPOCH
            );

            require(exists == false);
            require(epoch == futureEpoch);
        }
    }

    function test_GetCheckpointPerEpoch_Works_NegativePeriod() public {
        createCheckpoint(EPOCH_ONE, 5);

        (bool exists, int epoch, Checkpoint storage checkpoint) = checkpoints
            .getCheckpointPerEpoch(5, -10);

        require(exists);
        require(epoch == EPOCH_ONE);
        require(checkpoint.toHash() == checkpoints[EPOCH_ONE].toHash());
    }

    function createCheckpoint(
        int64 epoch,
        uint64 blockNumber
    ) internal returns (Checkpoint storage cp) {
        cp = checkpoints[epoch];

        address[] memory route = new address[](1);
        route[0] = makeAddr("root");

        cp.data.source = SubnetID(route).createSubnetId(address(100));
        cp.data.epoch = int64(blockNumber);
        cp.signature = new bytes(blockNumber);
    }
}
