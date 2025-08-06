// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity ^0.8.23;

import {SignedHeader} from "tendermint-sol/proto/TendermintLight.sol";

import {InvalidBatchEpoch, InvalidSignatureErr, DuplicateValidatorSignaturesFound, SignatureAddressesNotSorted, BottomUpCheckpointAlreadySubmitted, InvalidCheckpointEpoch} from "../errors/IPCErrors.sol";
import {IGateway} from "../interfaces/IGateway.sol";
import {BottomUpCheckpoint, BottomUpMsgBatch, BottomUpMsgBatchInfo} from "../structs/CrossNet.sol";
import {Validator, ValidatorSet, SubnetID} from "../structs/Subnet.sol";
import {MultisignatureChecker} from "../lib/LibMultisignatureChecker.sol";
import {ReentrancyGuard} from "../lib/LibReentrancyGuard.sol";
import {SubnetActorModifiers} from "../lib/LibSubnetActorStorage.sol";
import {LibValidatorSet, LibPower} from "../lib/LibPower.sol";
import {EnumerableSet} from "@openzeppelin/contracts/utils/structs/EnumerableSet.sol";
import {LibSubnetActor} from "../lib/LibSubnetActor.sol";
import {Pausable} from "../lib/LibPausable.sol";
import {LibGateway} from "../lib/LibGateway.sol";
import {LibActivity} from "../lib/LibActivity.sol";
import {LibBottomUpBatch} from "../lib/LibBottomUpBatch.sol";
import {BottomUpBatch} from "../structs/BottomUpBatch.sol";
import {IpcEnvelope, StateCommitment} from "../structs/CrossNet.sol";
import {CompressedActivityRollup} from "../structs/Activity.sol";
import {CometbftLightClient} from "../lib/cometbft/CometbftLightClient.sol";

/// Breakdown how the app hash is generated
struct StateCommitmentBreakDown {
    bytes stateRoot; // fvm state root
    BottomUpBatch.Commitment msgBatchCommitment;
    uint64 validatorNextConfigurationNumber;
    bytes32 activityCommitment;
}

contract SubnetActorCheckpointFacet is SubnetActorModifiers, ReentrancyGuard, Pausable {
    using EnumerableSet for EnumerableSet.AddressSet;
    using LibValidatorSet for ValidatorSet;

    error AppHashNotEqual();

    // TODO: The parameter should be SignedHeader.Data, but ethers-rust failed to generate the bindings
    function submitSignedHeader(bytes calldata rawData) external whenNotPaused {
        SignedHeader.Data memory header = abi.decode(rawData, (SignedHeader.Data));

        uint64 height = uint64(header.commit.height);
        // Enforcing a sequential submission
        ensureValidHeight(height);

        CometbftLightClient.verifyValidatorsQuorum(header);

        s.stateCommitments[height] = StateCommitment({blockHeight: height, commitment: header.header.app_hash});
        s.lastBottomUpCheckpointHeight = height;
    }

    /// @notice Apply the changes associated with the checkpoint
    function applyChanges(
        uint64 checkpointHeight,
        SubnetID calldata subnet,
        CompressedActivityRollup calldata activity,
        StateCommitmentBreakDown calldata breakdown,
        BottomUpBatch.Inclusion[] calldata inclusions
    ) external {
        validateAppHash(checkpointHeight, breakdown);

        LibPower.confirmChange(breakdown.validatorNextConfigurationNumber);

        LibActivity.recordActivityRollup(subnet, checkpointHeight, activity);

        LibBottomUpBatch.recordBottomUpBatchCommitment(checkpointHeight, breakdown.msgBatchCommitment);

        uint256 len = inclusions.length;
        IpcEnvelope[] memory msgs = new IpcEnvelope[](len);
        for (uint256 i = 0; i < len; ) {
            LibBottomUpBatch.processBottomUpBatchMsg(checkpointHeight, inclusions[i].msg, inclusions[i].proof);
            msgs[i] = inclusions[i].msg;
            unchecked {
                i++;
            }
        }

        IGateway(s.ipcGatewayAddr).execBottomUpMsgBatch(msgs);

        // Propagate cross messages from checkpoint to other subnets
        IGateway(s.ipcGatewayAddr).propagateAll();
    }

    function recordActivityRollup(
        uint64 checkpointHeight,
        SubnetID calldata subnet,
        CompressedActivityRollup calldata activity,
        StateCommitmentBreakDown calldata breakdown
    ) external whenNotPaused {
        validateAppHash(checkpointHeight, breakdown);
        LibActivity.recordActivityRollup(subnet, checkpointHeight, activity);
    }

    function confirmValidatorChange(
        uint64 checkpointHeight,
        StateCommitmentBreakDown calldata breakdown
    ) external whenNotPaused {
        validateAppHash(checkpointHeight, breakdown);
        LibPower.confirmChange(breakdown.validatorNextConfigurationNumber);
    }

    /// @notice Executes bottom-up messages that have been committed to in a checkpoint.
    /// @dev Each message in the batch must include a valid Merkle proof of inclusion.
    ///      This function verifies each proof, processes the message, and submits it to the gateway for execution.
    ///      It also triggers propagation of cross-subnet messages after successful execution.
    /// @param checkpointHeight The height of the checkpoint containing the committed messages.
    /// @param inclusions An array of inclusion proofs and messages to be executed.
    function execBottomUpMsgBatch(
        uint64 checkpointHeight,
        StateCommitmentBreakDown calldata breakdown,
        BottomUpBatch.Inclusion[] calldata inclusions
    ) external whenNotPaused {
        validateAppHash(checkpointHeight, breakdown);

        LibBottomUpBatch.recordBottomUpBatchCommitment(checkpointHeight, breakdown.msgBatchCommitment);

        uint256 len = inclusions.length;
        IpcEnvelope[] memory msgs = new IpcEnvelope[](len);
        for (uint256 i = 0; i < len; ) {
            LibBottomUpBatch.processBottomUpBatchMsg(checkpointHeight, inclusions[i].msg, inclusions[i].proof);
            msgs[i] = inclusions[i].msg;
            unchecked {
                i++;
            }
        }

        IGateway(s.ipcGatewayAddr).execBottomUpMsgBatch(msgs);

        // Propagate cross messages from checkpoint to other subnets
        IGateway(s.ipcGatewayAddr).propagateAll();
    }

    function ensureValidHeight(uint64 blockHeight) internal view {
        uint256 lastBottomUpCheckpointHeight = s.lastBottomUpCheckpointHeight;
        uint256 bottomUpCheckPeriod = s.bottomUpCheckPeriod;

        // cannot submit past bottom up checkpoint
        if (blockHeight <= lastBottomUpCheckpointHeight) {
            revert BottomUpCheckpointAlreadySubmitted();
        }

        uint256 nextCheckpointHeight = LibGateway.getNextEpoch(lastBottomUpCheckpointHeight, bottomUpCheckPeriod);
        if (blockHeight != nextCheckpointHeight) {
            revert InvalidCheckpointEpoch();
        }
    }

    function deriveAppHash(StateCommitmentBreakDown calldata breakdown) internal pure returns (bytes memory appHash) {
        bytes32 derived = keccak256(abi.encode(breakdown));

        appHash = new bytes(32);
        assembly {
            mstore(add(appHash, 32), derived)
        }
        return appHash;
    }

    function validateAppHash(
        uint64 checkpointHeight,
        StateCommitmentBreakDown calldata breakdown
    ) internal view whenNotPaused {
        bytes memory expectedAppHash = s.stateCommitments[checkpointHeight].commitment;
        bytes memory actual = deriveAppHash(breakdown);

        if (keccak256(expectedAppHash) != keccak256(actual)) revert AppHashNotEqual();
    }
}
