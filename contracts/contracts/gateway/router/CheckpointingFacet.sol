// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity ^0.8.23;

import {GatewayActorModifiers} from "../../lib/LibGatewayActorStorage.sol";
import {BottomUpCheckpoint} from "../../structs/CrossNet.sol";
import {LibGateway} from "../../lib/LibGateway.sol";
import {LibQuorum} from "../../lib/LibQuorum.sol";
import {Subnet} from "../../structs/Subnet.sol";
import {QuorumObjKind} from "../../structs/Quorum.sol";
import {Address} from "@openzeppelin/contracts/utils/Address.sol";

import {InvalidBatchSource, NotEnoughBalance, MaxMsgsPerBatchExceeded, InvalidCheckpointSource, CheckpointAlreadyExists} from "../../errors/IPCErrors.sol";
import {NotRegisteredSubnet, SubnetNotActive, SubnetNotFound, InvalidSubnet, CheckpointNotCreated} from "../../errors/IPCErrors.sol";
import {BatchNotCreated, InvalidBatchEpoch, BatchAlreadyExists, NotEnoughSubnetCircSupply, InvalidCheckpointEpoch} from "../../errors/IPCErrors.sol";

import {CrossMsgHelper} from "../../lib/CrossMsgHelper.sol";
import {IpcEnvelope, SubnetID, IpcMsgKind} from "../../structs/CrossNet.sol";
import {SubnetIDHelper} from "../../lib/SubnetIDHelper.sol";

import {ActivityRollupRecorded, FullActivityRollup} from "../../structs/Activity.sol";

contract CheckpointingFacet is GatewayActorModifiers {
    using SubnetIDHelper for SubnetID;
    using CrossMsgHelper for IpcEnvelope;

    /// @dev Emitted when a checkpoint is committed to gateway.
    event CheckpointCommitted(address indexed subnet, uint256 subnetHeight);

    /// @notice submit a verified checkpoint in the gateway to trigger side-effects.
    /// @dev this method is called by the corresponding subnet actor.
    ///     Called from a subnet actor if the checkpoint is cryptographically valid.
    /// @param checkpoint The bottom-up checkpoint to be committed.
    function commitCheckpoint(BottomUpCheckpoint calldata checkpoint) external {
        // checkpoint is used to implement access control
        if (checkpoint.subnetID.getActor() != msg.sender) {
            revert InvalidCheckpointSource();
        }
        (bool subnetExists, Subnet storage subnet) = LibGateway.getSubnet(msg.sender);
        if (!subnetExists) {
            revert SubnetNotFound();
        }
        if (!checkpoint.subnetID.equals(subnet.id)) {
            revert InvalidSubnet();
        }

        LibGateway.checkMsgLength(checkpoint.msgs);

        execBottomUpMsgs(checkpoint.msgs, subnet);

        emit CheckpointCommitted({subnet: checkpoint.subnetID.getAddress(), subnetHeight: checkpoint.blockHeight});
    }

    /// @notice creates a new bottom-up checkpoint
    /// @param checkpoint - a bottom-up checkpoint
    /// @param membershipRootHash - a root hash of the Merkle tree built from the validator public keys and their weight
    /// @param membershipWeight - the total weight of the membership
    /// @param activity - the full activity rollup
    function createBottomUpCheckpoint(
        BottomUpCheckpoint calldata checkpoint,
        bytes32 membershipRootHash,
        uint256 membershipWeight,
        FullActivityRollup calldata activity
    ) external systemActorOnly {
        if (LibGateway.bottomUpCheckpointExists(checkpoint.blockHeight)) {
            revert CheckpointAlreadyExists();
        }

        LibQuorum.createQuorumInfo({
            self: s.checkpointQuorumMap,
            objHeight: checkpoint.blockHeight,
            objHash: keccak256(abi.encode(checkpoint)),
            membershipRootHash: membershipRootHash,
            membershipWeight: membershipWeight,
            majorityPercentage: s.majorityPercentage
        });

        LibGateway.storeBottomUpCheckpoint(checkpoint);

        emit ActivityRollupRecorded(uint64(checkpoint.blockHeight), activity);
    }

    /// @notice checks whether the provided checkpoint signature for the block at height `height` is valid and accumulates that it
    /// @dev If adding the signature leads to reaching the threshold, then the checkpoint is removed from `incompleteCheckpoints`
    /// @param height - the height of the block in the checkpoint
    /// @param membershipProof - a Merkle proof that the validator was in the membership at height `height` with weight `weight`
    /// @param weight - the weight of the validator
    /// @param signature - the signature of the checkpoint
    function addCheckpointSignature(
        uint256 height,
        bytes32[] memory membershipProof,
        uint256 weight,
        bytes memory signature
    ) external {
        // check if the checkpoint was already pruned before getting checkpoint
        // and triggering the signature
        LibQuorum.isHeightAlreadyProcessed(s.checkpointQuorumMap, height);

        // slither-disable-next-line unused-return
        (bool exists, ) = LibGateway.getBottomUpCheckpoint(height);
        if (!exists) {
            revert CheckpointNotCreated();
        }
        LibQuorum.addQuorumSignature({
            self: s.checkpointQuorumMap,
            height: height,
            membershipProof: membershipProof,
            weight: weight,
            signature: signature
        });
    }

    /// @notice submit a batch of cross-net messages for execution.
    /// @param msgs The batch of bottom-up cross-network messages to be executed.
    function execBottomUpMsgs(IpcEnvelope[] calldata msgs, Subnet storage subnet) internal {
        uint256 totalValue;
        uint256 crossMsgLength = msgs.length;

        for (uint256 i; i < crossMsgLength; ) {
            totalValue += msgs[i].value;
            unchecked {
                ++i;
            }
        }

        uint256 totalAmount = totalValue;

        if (subnet.circSupply < totalAmount) {
            revert NotEnoughSubnetCircSupply();
        }

        subnet.circSupply -= totalAmount;

        // execute cross-messages
        LibGateway.applyMessages(subnet.id, msgs);
    }
}
