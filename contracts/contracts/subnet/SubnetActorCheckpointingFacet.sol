// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity ^0.8.23;

import {LightHeader, CanonicalVote} from "tendermint-sol/proto/TendermintLight.sol";

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
import {CometbftLightClient, ValidatorSignPayload, AppHashBreakdown} from "../lib/cometbft/CometbftLightClient.sol";

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

    /// @inheritdoc ISubnetActorCheckpointing
    function lastBottomUpCheckpointHeight() external view returns (uint256) {
        return uint256(LibCheckpointingStorage.getStorage().lastSubmissionHeight);
    }

    /// @inheritdoc ISubnetActorCheckpointing
    function submitBottomUpCheckpoint(bytes calldata rawData) external whenNotPaused {
        SubnetActorCheckpointingStorage storage checkpointStorage = LibCheckpointingStorage.getStorage();

        // In the original cometbft SignedHeader, which is used for pre-commit quorum certificate, the validator signatures
        // and the light client header they are signing are all contained in SignedHeader struct. However, for light client
        // verification, protobuf encoding is required. This means type conversion for SignedHeader needs to be done on chain.
        // This process could be gas costly.
        // To be more gas efficient, the voteTemplate contains the common fields that valiators are using to sign the pre-commit
        // quorum payload. Their signatures are grouped into `ValidatorSignPayload[] signatures` to reduce as much on chain
        // data conversion as possible.
        (
            LightHeader.Data memory header,
            ValidatorSignPayload[] memory signatures,
            CanonicalVote.Data memory voteTemplate
        ) = abi.decode(rawData, (LightHeader.Data, ValidatorSignPayload[], CanonicalVote.Data));

        uint64 height = uint64(header.height);
        // Enforcing a sequential submission
        ensureValidHeight(height, checkpointStorage.lastSubmissionHeight);

        /// Performs protobuf encoding against the header, can be gas intensive
        CometbftLightClient.verifyValidatorsQuorum(header, signatures, voteTemplate);

        checkpointStorage.appHash[height] = header.app_hash;
        checkpointStorage.lastSubmissionHeight = height;
    }

    /// @dev Once the checkpoint is submitted, it is just the signed cometbft app hash. The app hash is the hash of AppHashBreakdown.
    /// The app hash break down is the aggregate of commitments for configuration number or message batch root.
    /// It's not submitted together with `submitBottomUpCheckpoint` for gas considerations.
    function recordAppHashBreakdown(
        uint64 checkpointHeight,
        SubnetID calldata subnet,
        AppHashBreakdown calldata breakdown
    ) external whenNotPaused {
        validateAppHash(checkpointHeight, breakdown);

        LibPower.confirmChange(breakdown.validatorNextConfigurationNumber);
        LibBottomUpBatch.recordBottomUpBatchCommitment(checkpointHeight, breakdown.msgBatchCommitment);
        LibActivity.recordActivityRollup(subnet, checkpointHeight, breakdown.activityCommitment);
    }

    /// @dev Execute the bottom up message batch after the commitment is registered
    function execBottomUpMsgBatch(
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

    function deriveAppHash(AppHashBreakdown calldata breakdown) internal pure returns (bytes memory appHash) {
        bytes32 derived = keccak256(abi.encode(breakdown));

        appHash = new bytes(32);
        assembly {
            mstore(add(appHash, 32), derived)
        }
        return appHash;
    }

    function validateAppHash(uint64 checkpointHeight, AppHashBreakdown calldata breakdown) internal view whenNotPaused {
        SubnetActorCheckpointingStorage storage checkpointStorage = LibCheckpointingStorage.getStorage();

        bytes memory expectedAppHash = checkpointStorage.appHash[checkpointHeight];
        bytes memory actual = deriveAppHash(breakdown);

        if (keccak256(expectedAppHash) != keccak256(actual)) revert AppHashNotEqual();
    }
}

// ================ INTERNAL UTIL ===================

struct SubnetActorCheckpointingStorage {
    /// @notice contains all committed subnet app hash against each checkpoint height
    mapping(uint64 => bytes) appHash;
    uint64 lastSubmissionHeight;
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
