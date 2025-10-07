// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity ^0.8.23;

import {SignedHeader} from "tendermint-sol/proto/TendermintLight.sol";

import {BottomUpCheckpointAlreadySubmitted, InvalidCheckpointEpoch} from "../errors/IPCErrors.sol";
import {IGateway} from "../interfaces/IGateway.sol";
import {BottomUpMsgBatch, BottomUpMsgBatchInfo} from "../structs/CrossNet.sol";
import {Validator, ValidatorSet, SubnetID} from "../structs/Subnet.sol";
import {ISubnetActorCheckpointing} from "../interfaces/ISubnetActor.sol";
import {ReentrancyGuard} from "../lib/LibReentrancyGuard.sol";
import {LibSubnetActorStorage} from "../lib/LibSubnetActorStorage.sol";
import {LibValidatorSet, LibPower} from "../lib/LibPower.sol";
import {EnumerableSet} from "@openzeppelin/contracts/utils/structs/EnumerableSet.sol";
import {LibSubnetActor} from "../lib/LibSubnetActor.sol";
import {Pausable} from "../lib/LibPausable.sol";
import {LibGateway} from "../lib/LibGateway.sol";
import {LibActivity} from "../lib/LibActivity.sol";
import {LibBottomUpBatch} from "../lib/LibBottomUpBatch.sol";
import {BottomUpBatch} from "../structs/BottomUpBatch.sol";
import {IpcEnvelope} from "../structs/CrossNet.sol";
import {CompressedActivityRollup} from "../structs/Activity.sol";
import {CometbftLightClient, StateCommitmentBreakDown} from "../lib/cometbft/CometbftLightClient.sol";

/// @notice Tracks the latest checkpoint heights for different commitment types
/// @dev Used to ensure sequential processing and prevent replay of checkpoints
struct LastCommitmentHeights {
    /// The height of the last submitted and verified CometBFT signed header
    uint64 signedHeader;
    /// The height of the last processed validator configuration change
    uint64 configNumber;
    /// The height of the last recorded activity rollup
    uint64 activity;
}

/// @title Subnet Actor Checkpointing Facet
/// @notice Handles bottom-up checkpoint submission and verification for IPC subnets
/// @dev The current implementation is based on CometBFT light client verification and manages state commitments
///
/// This facet is responsible for:
/// - Verifying and storing CometBFT signed headers with BFT consensus validation
/// - Recording activity rollups from subnet validators
/// - Confirming validator set changes
/// - Executing bottom-up message batches with Merkle proof verification
/// - Ensuring sequential checkpoint submission at correct intervals
contract SubnetActorCheckpointingFacet is ISubnetActorCheckpointing, ReentrancyGuard, Pausable {
    using EnumerableSet for EnumerableSet.AddressSet;
    using LibValidatorSet for ValidatorSet;

    error AppHashNotEqual();
    error InvalidActivityCommiment();

    /// @inheritdoc ISubnetActorCheckpointing
    function lastBottomUpCheckpointHeight() external view returns (uint256) {
        return uint256(LibCheckpointingStorage.getStorage().commitmentHeights.signedHeader);
    }

    /// @inheritdoc ISubnetActorCheckpointing
    function submitBottomUpCheckpoint(bytes calldata rawData) external whenNotPaused {
        SubnetActorCheckpointingStorage storage checkpointStorage = LibCheckpointingStorage.getStorage();

        SignedHeader.Data memory header = abi.decode(rawData, (SignedHeader.Data));

        uint64 height = uint64(header.commit.height);
        // Enforcing a sequential submission
        ensureValidHeight(height, checkpointStorage.commitmentHeights.signedHeader);

        CometbftLightClient.verifyValidatorsQuorum(header);

        checkpointStorage.stateCommitments[height] = header.header.app_hash;
        checkpointStorage.commitmentHeights.signedHeader = height;
    }

    function getLastCommitmentHeights() external view returns (LastCommitmentHeights memory) {
        return LibCheckpointingStorage.getStorage().commitmentHeights;
    }

    function recordActivityRollup(
        uint64 checkpointHeight,
        SubnetID calldata subnet,
        CompressedActivityRollup calldata activity,
        StateCommitmentBreakDown calldata breakdown
    ) external whenNotPaused {
        SubnetActorCheckpointingStorage storage checkpointStorage = LibCheckpointingStorage.getStorage();

        if (breakdown.activityCommitment != keccak256(abi.encode(activity))) revert InvalidActivityCommiment();

        validateAppHash(checkpointHeight, breakdown);
        ensureValidHeight(checkpointHeight, checkpointStorage.commitmentHeights.activity);

        LibActivity.recordActivityRollup(subnet, checkpointHeight, activity);

        checkpointStorage.commitmentHeights.activity = checkpointHeight;
    }

    function confirmValidatorChange(
        uint64 checkpointHeight,
        StateCommitmentBreakDown calldata breakdown
    ) external whenNotPaused {
        SubnetActorCheckpointingStorage storage checkpointStorage = LibCheckpointingStorage.getStorage();

        validateAppHash(checkpointHeight, breakdown);
        ensureValidHeight(checkpointHeight, checkpointStorage.commitmentHeights.configNumber);

        LibPower.confirmChange(breakdown.validatorNextConfigurationNumber);

        checkpointStorage.commitmentHeights.configNumber = checkpointHeight;
    }

    /// @notice Executes the whole batch of bottom-up messages that have been committed to in a checkpoint.
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
        _execBottomUpMsgBatch(checkpointHeight, inclusions);
    }

    /// Record the bottom up msg batch commitment so that bottom up batch can be executed one by one
    function recordBottomUpMsgBatch(
        uint64 checkpointHeight,
        StateCommitmentBreakDown calldata breakdown
    ) external whenNotPaused {
        validateAppHash(checkpointHeight, breakdown);
        LibBottomUpBatch.recordBottomUpBatchCommitment(checkpointHeight, breakdown.msgBatchCommitment);
    }

    /// Execute the bottom up message batch after the commitment is registered
    function execBottomUpMsgBatchOnly(
        uint64 checkpointHeight,
        BottomUpBatch.Inclusion[] calldata inclusions
    ) external whenNotPaused {
        _execBottomUpMsgBatch(checkpointHeight, inclusions);
    }

    function _execBottomUpMsgBatch(uint64 checkpointHeight, BottomUpBatch.Inclusion[] calldata inclusions) internal {
        uint256 len = inclusions.length;
        IpcEnvelope[] memory msgs = new IpcEnvelope[](len);
        for (uint256 i = 0; i < len; ) {
            LibBottomUpBatch.processBottomUpBatchMsg(checkpointHeight, inclusions[i].msg, inclusions[i].proof);
            msgs[i] = inclusions[i].msg;
            unchecked {
                i++;
            }
        }

        IGateway(LibSubnetActorStorage.appStorage().ipcGatewayAddr).execBottomUpMsgBatch(msgs);

        // Propagate cross messages from checkpoint to other subnets
        IGateway(LibSubnetActorStorage.appStorage().ipcGatewayAddr).propagateAll();
    }

    function ensureValidHeight(uint64 blockHeight, uint64 lastHeight) internal view {
        uint256 bottomUpCheckPeriod = LibSubnetActorStorage.appStorage().bottomUpCheckPeriod;

        // cannot submit past bottom up checkpoint
        if (blockHeight <= lastHeight) {
            revert BottomUpCheckpointAlreadySubmitted();
        }

        uint256 nextCheckpointHeight = LibGateway.getNextEpoch(uint256(lastHeight), bottomUpCheckPeriod);
        if (blockHeight != uint64(nextCheckpointHeight)) {
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
        SubnetActorCheckpointingStorage storage checkpointStorage = LibCheckpointingStorage.getStorage();

        bytes memory expectedAppHash = checkpointStorage.stateCommitments[checkpointHeight];
        bytes memory actual = deriveAppHash(breakdown);

        if (keccak256(expectedAppHash) != keccak256(actual)) revert AppHashNotEqual();
    }
}

// ================ INTERNAL UTIL ===================

struct SubnetActorCheckpointingStorage {
    /// @notice contains all committed subnet state hash and block header
    mapping(uint64 => bytes) stateCommitments;
    LastCommitmentHeights commitmentHeights;
}

library LibCheckpointingStorage {
    bytes32 private constant NAMESPACE = keccak256("SubnetActorCheckpointingFacet.storage");

    function getStorage() internal pure returns (SubnetActorCheckpointingStorage storage ds) {
        bytes32 position = NAMESPACE;
        assembly {
            ds.slot := position
        }
        return ds;
    }
}
