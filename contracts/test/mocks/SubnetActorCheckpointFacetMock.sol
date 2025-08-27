// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity ^0.8.23;

import {SignedHeader} from "tendermint-sol/proto/TendermintLight.sol";

import {SubnetActorCheckpointingFacet} from "../../contracts/subnet/SubnetActorCheckpointingFacet.sol";
import {BottomUpBatch} from "../../contracts/structs/BottomUpBatch.sol";
import {CompressedActivityRollup} from "../../contracts/structs/Activity.sol";
import {SubnetID} from "../../contracts/structs/Subnet.sol";

import {LibPower} from "../../contracts/lib/LibPower.sol";
import {LibActivity} from "../../contracts/lib/LibActivity.sol";
import {LibBottomUpBatch} from "../../contracts/lib/LibBottomUpBatch.sol";

contract SubnetActorCheckpointFacetMock is SubnetActorCheckpointingFacet {
    function commitSideEffects(
        uint256 h,
        SubnetID calldata subnetId,
        CompressedActivityRollup calldata activity,
        BottomUpBatch.Commitment calldata msgs,
        uint64 nextConfigurationNumber
    ) external {
        s.commitmentHeights.signedHeader = uint64(h);
        s.commitmentHeights.activity = uint64(h);
        s.commitmentHeights.configNumber = uint64(h);

        if (msgs.totalNumMsgs > 0) {
            LibBottomUpBatch.recordBottomUpBatchCommitment(uint64(h), msgs);
        }
        LibActivity.recordActivityRollup(subnetId, uint64(h), activity);

        // confirming the changes in membership in the child
        LibPower.confirmChange(nextConfigurationNumber);
    }

    function execBottomUpMsgBatch(
        uint256 checkpointHeight,
        BottomUpBatch.Inclusion[] calldata inclusions
    ) external whenNotPaused {
        _execBottomUpMsgBatch(uint64(checkpointHeight), inclusions);
    }
}
