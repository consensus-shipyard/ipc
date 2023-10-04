// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity 0.8.19;

import {SubnetActorManagerFacet} from "../../src/subnet/SubnetActorManagerFacet.sol";
import {LibStaking} from "../../src/lib/LibStaking.sol";

contract SubnetManagerTestUtil is SubnetActorManagerFacet {
    function confirmChange(uint64 _configurationNumber) external {
        LibStaking.confirmChange(_configurationNumber);
    }
}