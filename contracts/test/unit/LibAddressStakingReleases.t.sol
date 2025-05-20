// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity ^0.8.23;

import "forge-std/Test.sol";
import {NoCollateralToWithdraw} from "../../contracts/errors/IPCErrors.sol";
import {LibAddressStakingReleases, AddressStakingReleases, StakingRelease} from "../../contracts/lib/LibPower.sol";

contract AssetHelperTest is Test {
    using LibAddressStakingReleases for AddressStakingReleases;

    AddressStakingReleases private releaseQueue;

    function test_compactWorks() public {
        releaseQueue.push(StakingRelease({releaseAt: 100, amount: 10}));
        releaseQueue.push(StakingRelease({releaseAt: 110, amount: 10}));
        releaseQueue.push(StakingRelease({releaseAt: 120, amount: 10}));
        releaseQueue.push(StakingRelease({releaseAt: 130, amount: 10}));
        releaseQueue.push(StakingRelease({releaseAt: 140, amount: 10}));

        uint256 amount;
        uint16 releasesToCollect;

        vm.roll(99);
        (amount, releasesToCollect) = releaseQueue.compact();
        require(amount == 0, "should not have any amount to collect");
        require(releasesToCollect == 5, "should have 5 future releases");

        vm.roll(101);
        (amount, releasesToCollect) = releaseQueue.compact();
        require(amount == 10, "should have 10 to collect");
        require(releasesToCollect == 4, "should have 4 future releases");

        vm.roll(121);
        (amount, releasesToCollect) = releaseQueue.compact();
        require(amount == 20, "should have 20 to collect");
        require(releasesToCollect == 2, "should have 2 future releases");

        vm.roll(160);
        (amount, releasesToCollect) = releaseQueue.compact();
        require(amount == 20, "should  have 20 to collect after block 160");
        require(releasesToCollect == 0, "should have 0 future releases");

        vm.roll(180);
        vm.expectRevert(abi.encodeWithSelector(NoCollateralToWithdraw.selector));
        (amount, releasesToCollect) = releaseQueue.compact();
    }
}
