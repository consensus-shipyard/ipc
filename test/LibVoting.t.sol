// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity 0.8.19;

import "forge-std/Test.sol";
import "forge-std/console.sol";

import "../src/lib/LibVoting.sol";

contract LibVotingTest is Test {
    function test_basic() public {
        LibVoting.initVoting(50, 10);

        require(LibVoting.getSubmissionPeriod() == 10, "submission period correct");

        LibVoting.initGenesisEpoch(15);
        require(LibVoting.getGenesisEpoch() == 15, "genesis epoch correct");

        vm.expectRevert(EpochNotVotable.selector);
        LibVoting.applyValidEpochOnly(5);
    }

    function test_fails_EpochAlreadyExecuted() public {
        LibVoting.initVoting(50, 10);
        require(LibVoting.getSubmissionPeriod() == 10, "submission period correct");
        LibVoting.initGenesisEpoch(15);
        vm.expectRevert(EpochAlreadyExecuted.selector);
        LibVoting.applyValidEpochOnly(0);
    }

    function test_fails_EpochNotVotable() public {
        LibVoting.initVoting(50, 10);
        require(LibVoting.getSubmissionPeriod() == 10, "submission period correct");
        LibVoting.initGenesisEpoch(15);
        vm.expectRevert(EpochNotVotable.selector);
        LibVoting.applyValidEpochOnly(16);
    }

    function test_valid_epoch() public {
        LibVoting.initVoting(50, 10);
        require(LibVoting.getSubmissionPeriod() == 10, "submission period correct");
        LibVoting.initGenesisEpoch(15);
        LibVoting.applyValidEpochOnly(25);
    }

    function test_fails_epoch_100() public {
        LibVoting.initVoting(50, 10);
        require(LibVoting.getSubmissionPeriod() == 10, "submission period correct");
        LibVoting.initGenesisEpoch(100);
        vm.expectRevert(EpochNotVotable.selector);
        LibVoting.applyValidEpochOnly(99);
    }

    function test_works_epoch_100() public {
        LibVoting.initVoting(50, 10);
        require(LibVoting.getSubmissionPeriod() == 10, "submission period correct");
        LibVoting.initGenesisEpoch(100);
        LibVoting.applyValidEpochOnly(100);
    }
}
