// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity ^0.8.23;

import {Pausable} from "../lib/LibPausable.sol";
import {ReentrancyGuard} from "../lib/LibReentrancyGuard.sol";
import {SubnetActorModifiers} from "../lib/LibSubnetActorStorage.sol";
import {LibPower} from "../lib/LibPower.sol";
import {LibSubnetActor} from "../lib/LibSubnetActor.sol";
import {AssetHelper} from "../lib/AssetHelper.sol";
import {Asset} from "../structs/Subnet.sol";

contract SubnetActorRewardFacet is SubnetActorModifiers, ReentrancyGuard, Pausable {
    using AssetHelper for Asset;

    /// @notice Validator claims their released collateral.
    function claim() external nonReentrant whenNotPaused {
        uint256 amount = LibPower.claimCollateral(msg.sender);
        if (amount > 0) {
            s.collateralSource.transferFunds(payable(msg.sender), amount);
        }
    }
}
