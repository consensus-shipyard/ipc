// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity ^0.8.23;

import {ActivitySummary} from "../structs/CrossNet.sol";
import {QuorumObjKind} from "../structs/Quorum.sol";
import {Pausable} from "../lib/LibPausable.sol";
import {ReentrancyGuard} from "../lib/LibReentrancyGuard.sol";
import {SubnetActorModifiers} from "../lib/LibSubnetActorStorage.sol";
import {LibStaking} from "../lib/LibStaking.sol";
import {LibSubnetActor} from "../lib/LibSubnetActor.sol";
import {AssetHelper} from "../lib/AssetHelper.sol";
import {Asset} from "../structs/Subnet.sol";

contract SubnetActorRewardFacet is SubnetActorModifiers, ReentrancyGuard, Pausable {
    using AssetHelper for Asset;

    // TODO(rewards): add this function so that relayers can submit summaries to process reward payouts in the root network.
    function submitSummary(SubnetID subnetId, ActivitySummary memory summary) external nonReentrant whenNotPaused {
        // TODO(rewards):
        //  1. Check that the subnet is active.
        //  2. Check that the subnet has a non-zero ValidatorRewarder.
        //  3. Hash the activity summary to get the commitment.
        //  4. Validate that the commitment is pending and presentable, and validate that it matches the expected subnet.
        //  5. Send the summary to the ValidatorRewarder#disburseRewards.
        //  6. If OK (not reverted), drop the summary from the pending and presentable commitments.
    }

    /// @notice Validator claims their released collateral.
    function claim() external nonReentrant whenNotPaused {
        uint256 amount = LibStaking.claimCollateral(msg.sender);
        if (amount > 0) {
            s.collateralSource.transferFunds(payable(msg.sender), amount);
        }
    }
}
