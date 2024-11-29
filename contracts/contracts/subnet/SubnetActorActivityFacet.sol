// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity ^0.8.23;

import {Consensus} from "../structs/Activity.sol";
import {LibActivity} from "../lib/LibActivity.sol";
import {LibDiamond} from "../lib/LibDiamond.sol";
import {NotAuthorized} from "../errors/IPCErrors.sol";
import {Pausable} from "../lib/LibPausable.sol";
import {ReentrancyGuard} from "../lib/LibReentrancyGuard.sol";
import {SubnetIDHelper} from "../lib/SubnetIDHelper.sol";
import {SubnetID} from "../structs/Subnet.sol";

/// The validator reward facet for the parent subnet, i.e. for validators in the child subnet
/// to claim their reward in the parent subnet, which should be the current subnet this facet
/// is deployed.
contract SubnetActorActivityFacet is ReentrancyGuard, Pausable {
    // Entrypoint for validators to batch claim rewards in the parent subnet, for a given subnet,
    // against multiple checkpoints at once. Atomically succeeds or reverts.
    function batchSubnetClaim(
        SubnetID calldata subnet,
        uint64[] calldata checkpointHeights,
        Consensus.ValidatorClaim[] calldata claims
    ) external nonReentrant whenNotPaused {
        require(checkpointHeights.length == claims.length, "length mismatch");
        uint256 len = claims.length;
        for (uint256 i = 0; i < len; ) {
            _claim(subnet, checkpointHeights[i], claims[i].data, claims[i].proof);
            unchecked {
                i++;
            }
        }
    }

    /// Entrypoint for validators to claim their reward for doing work in the child subnet.
    function claim(
        SubnetID calldata subnet,
        uint64 checkpointHeight,
        Consensus.ValidatorData calldata data,
        Consensus.MerkleHash[] calldata proof
    ) external nonReentrant whenNotPaused {
        _claim(subnet, checkpointHeight, data, proof);
    }

    // ======== Internal functions ===========

    function _claim(
        SubnetID calldata subnetId,
        uint64 checkpointHeight,
        Consensus.ValidatorData calldata detail,
        Consensus.MerkleHash[] calldata proof
    ) internal {
        // Note: No need to check if the subnet is active. If the subnet is not active, the checkpointHeight
        // will never exist.
        if (msg.sender != detail.validator) {
            revert NotAuthorized(msg.sender);
        }

        LibActivity.processConsensusClaim(subnetId, checkpointHeight, detail, proof);
    }
}
