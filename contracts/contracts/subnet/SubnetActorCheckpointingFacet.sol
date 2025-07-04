// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity ^0.8.23;

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
import {IpcEnvelope} from "../structs/CrossNet.sol";

contract SubnetActorCheckpointingFacet is SubnetActorModifiers, ReentrancyGuard, Pausable {
    using EnumerableSet for EnumerableSet.AddressSet;
    using LibValidatorSet for ValidatorSet;

    /// @notice Submits a checkpoint commitment for execution.
    /// @dev    It triggers the commitment of the checkpoint and any other side-effects that
    ///         need to be triggered by the checkpoint such as relayer reward book keeping.
    /// @param checkpoint The executed bottom-up checkpoint.
    /// @param signatories The addresses of validators signing the checkpoint.
    /// @param signatures The signatures of validators on the checkpoint.
    function submitCheckpoint(
        BottomUpCheckpoint calldata checkpoint,
        address[] calldata signatories,
        bytes[] calldata signatures
    ) external whenNotPaused {
        ensureValidCheckpoint(checkpoint);

        bytes32 checkpointHash = keccak256(abi.encode(checkpoint));

        // validate signatures and quorum threshold, revert if validation fails
        validateActiveQuorumSignatures({signatories: signatories, hash: checkpointHash, signatures: signatures});

        // If the checkpoint height is the next expected height then this is a new checkpoint which must be executed
        // in the Gateway Actor, the checkpoint and the relayer must be stored, last bottom-up checkpoint updated.
        s.committedCheckpoints[checkpoint.blockHeight] = checkpoint;

        s.lastBottomUpCheckpointHeight = checkpoint.blockHeight;

        // Commit in gateway to distribute rewards
        IGateway(s.ipcGatewayAddr).commitCheckpoint(checkpoint);

        if (checkpoint.msgs.totalNumMsgs > 0) {
            LibBottomUpBatch.recordBottomUpBatchCommitment(uint64(checkpoint.blockHeight), checkpoint.msgs);
        }
        LibActivity.recordActivityRollup(checkpoint.subnetID, uint64(checkpoint.blockHeight), checkpoint.activity);

        // confirming the changes in membership in the child
        LibPower.confirmChange(checkpoint.nextConfigurationNumber);
    }

    /// @notice Checks whether the signatures are valid for the provided signatories and hash within the current validator set.
    ///         Reverts otherwise.
    /// @dev Signatories in `signatories` and their signatures in `signatures` must be provided in the same order.
    ///       Having it public allows external users to perform sanity-check verification if needed.
    /// @param signatories The addresses of the signatories.
    /// @param hash The hash of the checkpoint.
    /// @param signatures The packed signatures of the checkpoint.
    function validateActiveQuorumSignatures(
        address[] memory signatories,
        bytes32 hash,
        bytes[] memory signatures
    ) public view {
        for (uint256 i = 1; i < signatories.length; ) {
            if (signatories[i] < signatories[i - 1]) {
                revert SignatureAddressesNotSorted();
            }
            if (signatories[i] == signatories[i - 1]) {
                revert DuplicateValidatorSignaturesFound();
            }

            unchecked {
                i++;
            }
        }

        // This call reverts if at least one of the signatories (validator) is not in the active validator set.
        uint256[] memory collaterals = s.validatorSet.getTotalPowerOfValidators(signatories);
        uint256 activeCollateral = s.validatorSet.getTotalActivePower();

        uint256 threshold = (activeCollateral * s.majorityPercentage) / 100;

        (bool valid, MultisignatureChecker.Error err) = MultisignatureChecker.isValidWeightedMultiSignature({
            signatories: signatories,
            weights: collaterals,
            threshold: threshold,
            hash: hash,
            signatures: signatures
        });

        if (!valid) {
            revert InvalidSignatureErr(uint8(err));
        }
    }

    /// @notice Ensures the checkpoint is valid.
    /// @dev The checkpoint block height must be equal to the last bottom-up checkpoint height or
    /// @dev the next one or the number of bottom up messages exceeds the max batch size.
    function ensureValidCheckpoint(BottomUpCheckpoint calldata checkpoint) internal view {
        uint256 lastBottomUpCheckpointHeight = s.lastBottomUpCheckpointHeight;
        uint256 bottomUpCheckPeriod = s.bottomUpCheckPeriod;

        // cannot submit past bottom up checkpoint
        if (checkpoint.blockHeight <= lastBottomUpCheckpointHeight) {
            revert BottomUpCheckpointAlreadySubmitted();
        }

        uint256 nextCheckpointHeight = LibGateway.getNextEpoch(lastBottomUpCheckpointHeight, bottomUpCheckPeriod);
        if (checkpoint.blockHeight != nextCheckpointHeight) {
            revert InvalidCheckpointEpoch();
        }
    }

    /// @notice Executes bottom-up messages that have been committed to in a checkpoint.
    /// @dev Each message in the batch must include a valid Merkle proof of inclusion.
    ///      This function verifies each proof, processes the message, and submits it to the gateway for execution.
    ///      It also triggers propagation of cross-subnet messages after successful execution.
    /// @param checkpointHeight The height of the checkpoint containing the committed messages.
    /// @param inclusions An array of inclusion proofs and messages to be executed.
    function execBottomUpMsgBatch(
        uint256 checkpointHeight,
        BottomUpBatch.Inclusion[] calldata inclusions
    ) external whenNotPaused {
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
}
