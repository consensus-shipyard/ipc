// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity ^0.8.23;

import {MaxMsgsPerBatchExceeded, InvalidSubnet, InvalidXnetMessage, InvalidSignatureErr, InvalidXnetMessageReason, BottomUpCheckpointAlreadySubmitted, CannotSubmitFutureCheckpoint, InvalidCheckpointEpoch} from "../errors/IPCErrors.sol";
import {CallMsg, IpcMsgKind, IpcEnvelope, OutcomeType, BottomUpMsgBatch, BottomUpCheckpoint} from "../structs/CrossNet.sol";
import {MultisignatureChecker} from "../lib/LibMultisignatureChecker.sol";
import {ReentrancyGuard} from "../lib/LibReentrancyGuard.sol";
import {Pausable} from "../lib/LibPausable.sol";
import {LibUtil} from "../lib/LibUtil.sol";
import {LibPowerChange, ProofOfPower, LibPowerQuery} from "../lib/power/LibPower.sol";
import {LibSubnetActorQuery} from "./SubnetActorFacet.sol";
import {SubnetIDHelper} from "../lib/SubnetIDHelper.sol";
import {SubnetID} from "../structs/CrossNet.sol";
import {LibBottomUpExecution} from "../lib/LibMsgExecution.sol";

struct SubnetCheckpointStorage {
    /// @notice The committed bottom up checkpoint hashes
    mapping(uint256 => bytes32) pastCheckpointHashes;
    /// @notice The previously committed bottom up checkpoint
    uint256 lastBottomUpCheckpointHeight;

    // ========= List of configuration params =======
    /// @notice The majority percentage that forms a quorum
    uint256 majorityPercentage;
    /// @notice The bottom up checkpoint period
    uint64 bottomUpCheckPeriod;
    /// @notice The max number of bottom up messages a checkpoint can store
    uint64 maxMsgsPerBottomUpBatch;
}

/// @notice This facet interfaces with the child subnet in the parent network. It receives the bottom up checkpoints
///         sent from the child to the parent.
contract SubnetCheckpointingFacet is ReentrancyGuard, Pausable {
    using LibSubnetCheckpoint for SubnetCheckpointStorage;

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
        SubnetCheckpointStorage storage s = LibSubnetCheckpoint.checkpointStorage();

        s.ensureValidCheckpoint(checkpoint);

        bytes32 checkpointHash = keccak256(abi.encode(checkpoint));

        // validate signatures and quorum threshold, revert if validation fails
        s.validateActiveQuorumSignatures({signatories: signatories, hash: checkpointHash, signatures: signatures});

        // If the checkpoint height is the next expected height then this is a new checkpoint which must be executed
        // in the Gateway Actor, the checkpoint and the relayer must be stored, last bottom-up checkpoint updated.
        s.newCheckpointSubmitted(checkpointHash, checkpoint.blockHeight);

        LibSubnetCheckpoint.execBottomUpMsgs(checkpoint.msgs);

        // confirming the changes in membership in the child
        LibPowerChange.confirmChange(checkpoint.nextConfigurationNumber);
    }
}

library LibSubnetCheckpoint {
    using SubnetIDHelper for SubnetID;

    function checkpointStorage() internal pure returns (SubnetCheckpointStorage storage ds) {
        bytes32 position = keccak256("ipc.subnet.bottomup.checkpoint.storage");
        assembly {
            ds.slot := position
        }
    }

    function newCheckpointSubmitted(
        SubnetCheckpointStorage storage s,
        bytes32 checkpointHash,
        uint256 blockHeight
    ) internal {
        s.pastCheckpointHashes[blockHeight] = checkpointHash;
        s.lastBottomUpCheckpointHeight = blockHeight;   
    }

    /// @notice Checks whether the signatures are valid for the provided signatories and hash within the current validator set.
    ///         Reverts otherwise.
    /// @dev Signatories in `signatories` and their signatures in `signatures` must be provided in the same order.
    ///       Having it public allows external users to perform sanity-check verification if needed.
    /// @param signatories The addresses of the signatories.
    /// @param hash The hash of the checkpoint.
    /// @param signatures The packed signatures of the checkpoint.
    function validateActiveQuorumSignatures(
        SubnetCheckpointStorage storage s,
        address[] memory signatories,
        bytes32 hash,
        bytes[] memory signatures
    ) public view {
        // This call reverts if at least one of the signatories (validator) is not in the active validator set.
        uint256[] memory collaterals = LibPowerQuery.confirmedPowerOfActiveValidators(signatories);
        uint256 activeCollateral = LibPowerQuery.confirmedPowerOfAllActiveValidators();

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
    function ensureValidCheckpoint(
        SubnetCheckpointStorage storage s,
        BottomUpCheckpoint calldata checkpoint
    ) internal view {
        if (!LibSubnetActorQuery.id().equals(checkpoint.subnetID)) {
            revert InvalidSubnet();
        }

        uint64 maxMsgsPerBottomUpBatch = s.maxMsgsPerBottomUpBatch;

        if (checkpoint.msgs.length > maxMsgsPerBottomUpBatch) {
            revert MaxMsgsPerBatchExceeded();
        }

        uint256 lastBottomUpCheckpointHeight = s.lastBottomUpCheckpointHeight;
        uint256 bottomUpCheckPeriod = s.bottomUpCheckPeriod;

        // cannot submit past bottom up checkpoint
        if (checkpoint.blockHeight <= lastBottomUpCheckpointHeight) {
            revert BottomUpCheckpointAlreadySubmitted();
        }

        uint256 nextCheckpointHeight = LibUtil.nextBottomUpCheckpointEpoch(lastBottomUpCheckpointHeight, bottomUpCheckPeriod);

        if (checkpoint.blockHeight > nextCheckpointHeight) {
            revert CannotSubmitFutureCheckpoint();
        }

        // the expected bottom up checkpoint height, valid height
        if (checkpoint.blockHeight == nextCheckpointHeight) {
            return;
        }

        // if the bottom up messages' length is max, we consider that epoch valid, allow early submission
        if (checkpoint.msgs.length == maxMsgsPerBottomUpBatch) {
            return;
        }

        revert InvalidCheckpointEpoch();
    }
    
    /// @notice submit a batch of cross-net messages for execution.
    /// @param msgs The batch of bottom-up cross-network messages to be executed.
    function execBottomUpMsgs(IpcEnvelope[] calldata msgs) internal {
        uint256 totalValue;
        uint256 crossMsgLength = msgs.length;

        for (uint256 i; i < crossMsgLength; ) {
            // TODO: validate it is indeed bottom up messages

            totalValue += msgs[i].value;
            unchecked {
                ++i;
            }

            LibBottomUpExecution.applyMsg(msgs[i]);
        }

        // TODO: udpate subnet circulation supply

        // if (subnet.circSupply < totalAmount) {
        //     revert NotEnoughSubnetCircSupply();
        // }

        // subnet.circSupply -= totalAmount;
    }
}