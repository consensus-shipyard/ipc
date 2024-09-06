// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity ^0.8.23;

import {QuorumObjKind} from "../structs/Quorum.sol";
import {Pausable} from "../lib/LibPausable.sol";
import {ReentrancyGuard} from "../lib/LibReentrancyGuard.sol";
import {SubnetActorModifiers} from "../lib/LibSubnetActorStorage.sol";
import {LibStaking} from "../lib/LibStaking.sol";
import {LibSubnetActor} from "../lib/LibSubnetActor.sol";
import {GenericTokenHelper} from "../lib/GenericTokenHelper.sol";
import {GenericToken} from "../structs/Subnet.sol";

contract SubnetActorRewardFacet is SubnetActorModifiers, ReentrancyGuard, Pausable {
    using GenericTokenHelper for GenericToken;

    /// @notice Validator claims their released collateral.
    function claim() external nonReentrant whenNotPaused {
        uint256 amount = LibStaking.claimCollateral(msg.sender);
        if (amount > 0) {
            s.collateralSource.transferFunds(payable(msg.sender), amount);
        }
    }
}
