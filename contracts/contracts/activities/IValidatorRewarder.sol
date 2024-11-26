// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity ^0.8.23;

import {SubnetID} from "../structs/Subnet.sol";
import {Consensus} from "./Activity.sol";

/// @title ValidatorRewarder interface.
///
/// @dev Implement this interface and supply the address of the implementation contract at subnet creation to process
/// subnet summaries at this level, and disburse rewards to validators based on their block production activity.
///
/// This interface will be called by the subnet actor when a validator presents a _valid_ proof of consensus activity,
/// via the ValidatorRewardFacet#claim
///
interface IValidatorRewarder {
    /// @notice Called by the subnet manager contract to instruct the rewarder to process the subnet summary and
    /// disburse any relevant rewards.
    /// @dev This method should revert if the summary is invalid; this will cause the
    function disburseRewards(SubnetID calldata id, Consensus.ValidatorData calldata validatedData) external;
}
