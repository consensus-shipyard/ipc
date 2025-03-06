// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity ^0.8.23;

import {SubnetActorManagerFacet} from "../../contracts/subnet/SubnetActorManagerFacet.sol";
import {LibPower} from "../../contracts/lib/LibPower.sol";
import {SubnetActorRewardFacet} from "../../contracts/subnet/SubnetActorRewardFacet.sol";

contract SubnetActorMock is SubnetActorManagerFacet, SubnetActorRewardFacet {
    function confirmChange(uint64 _configurationNumber) external {
        LibPower.confirmChange(_configurationNumber);
    }

    function confirmNextChange() external {
        (uint64 nextConfigNum, ) = LibPower.getConfigurationNumbers();
        LibPower.confirmChange(nextConfigNum - 1);
    }
}
