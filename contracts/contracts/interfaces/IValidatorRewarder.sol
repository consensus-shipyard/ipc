// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity ^0.8.23;

import {SubnetID} from "../structs/Subnet.sol";
import {Consensus} from "../structs/Activity.sol";

/// @title ValidatorRewarder interface.
///
/// @dev Implement this interface and supply the address of the implementation contract at subnet creation to process
/// consensus activity summaries at this level, and disburse rewards to validators based on their block production
/// activities inside the subnet.
///
/// This interface will be called by the subnet actor when a validator presents a _valid_ proof of consensus activity,
/// via the SubnetActivityActivityFacet#claim method.
interface IValidatorRewarder {
    /// Called by the subnet actor when a validator presents a _valid_ proof of consensus activity, via
    /// SubnetActorActivityFacet#claim() or its batch equivalents.
    /// @dev This method should revert if the summary is invalid; this will cause the claim submission to be rejected.
    function notifyValidClaim(
        SubnetID calldata id,
        uint64 checkpointHeight,
        Consensus.ValidatorData calldata validatedData
    ) external;
}
