// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity 0.8.19;

import {GatewayActorModifiers} from "../lib/LibGatewayActorStorage.sol";
import {EMPTY_HASH, METHOD_SEND} from "../constants/Constants.sol";
import {CrossMsg, StorableMsg, ParentFinality, BottomUpCheckpoint, CheckpointInfo} from "../structs/Checkpoint.sol";
import {Status} from "../enums/Status.sol";
import {IPCMsgType} from "../enums/IPCMsgType.sol";
import {SubnetID, Subnet} from "../structs/Subnet.sol";
import {IPCMsgType} from "../enums/IPCMsgType.sol";
import {Membership} from "../structs/Validator.sol";
import {InconsistentPrevCheckpoint, NotEnoughSubnetCircSupply, InvalidCheckpointEpoch, InvalidSignature, NotAuthorized, SignatureReplay, InvalidRetentionHeight, FailedRemoveIncompleteCheckpoint} from "../errors/IPCErrors.sol";
import {InvalidCheckpointSource, InvalidCrossMsgNonce, InvalidCrossMsgDstSubnet, CheckpointAlreadyExists, CheckpointInfoAlreadyExists, IncompleteCheckpointExists, CheckpointAlreadyProcessed, FailedAddIncompleteCheckpoint, FailedAddSignatory, FailedAddSignature} from "../errors/IPCErrors.sol";
import {MessagesNotSorted, NotEnoughBalance, NotRegisteredSubnet} from "../errors/IPCErrors.sol";
import {NotValidator, SubnetNotActive, CheckpointNotCreated, CheckpointMembershipNotCreated, ZeroMembershipWeight} from "../errors/IPCErrors.sol";
import {SubnetIDHelper} from "../lib/SubnetIDHelper.sol";
import {CheckpointHelper} from "../lib/CheckpointHelper.sol";
import {CrossMsgHelper} from "../lib/CrossMsgHelper.sol";
import {LibGateway} from "../lib/LibGateway.sol";
import {StorableMsgHelper} from "../lib/StorableMsgHelper.sol";
import {FvmAddress} from "../structs/FvmAddress.sol";
import {FvmAddressHelper} from "../lib/FvmAddressHelper.sol";
import {FilAddress} from "fevmate/utils/FilAddress.sol";
import {ECDSA} from "openzeppelin-contracts/utils/cryptography/ECDSA.sol";
import {MerkleProof} from "openzeppelin-contracts/utils/cryptography/MerkleProof.sol";
import {EnumerableSet} from "openzeppelin-contracts/utils/structs/EnumerableSet.sol";
import {StakingChangeRequest, ParentValidatorsTracker} from "../structs/Subnet.sol";
import {LibValidatorTracking} from "../lib/LibStaking.sol";

contract GatewayRouterFacet is GatewayActorModifiers {
    using FilAddress for address;
    using SubnetIDHelper for SubnetID;
    using CheckpointHelper for BottomUpCheckpoint;
    using CrossMsgHelper for CrossMsg;
    using FvmAddressHelper for FvmAddress;
    using StorableMsgHelper for StorableMsg;
    using EnumerableSet for EnumerableSet.UintSet;
    using EnumerableSet for EnumerableSet.AddressSet;
    using LibValidatorTracking for ParentValidatorsTracker;

    event QuorumReached(uint64 height, bytes32 checkpoint, uint256 quorumWeight);
    event QuorumWeightUpdated(uint64 height, bytes32 checkpoint, uint256 newWeight);

    // TODO: Reimplement the function.
    /// @dev This function must be reimplemented according to the current checkpoint protocol.
    /// @notice submit a checkpoint in the gateway. Called from a subnet once the checkpoint is voted for and reaches majority
    function commitChildCheck(BottomUpCheckpoint calldata commit) external {
        if (commit.subnetID.getActor().normalize() != msg.sender) {
            revert InvalidCheckpointSource();
        }

        // slither-disable-next-line unused-return
        (, Subnet storage subnet) = LibGateway.getSubnet(msg.sender);
        if (subnet.status != Status.Active) {
            revert SubnetNotActive();
        }

        // get checkpoint for the current template being populated
        (bool checkpointExists, uint64 nextCheckEpoch, BottomUpCheckpoint storage checkpoint) = LibGateway
            .getCurrentBottomUpCheckpoint();

        // create a checkpoint template if it doesn't exists
        if (!checkpointExists) {
            checkpoint.subnetID = s.networkName;
            checkpoint.blockHeight = nextCheckEpoch;
        }

        CrossMsg[] memory messages = s.bottomUpMessages[commit.blockHeight];

        uint256 totalValue = 0;
        uint256 crossMsgLength = messages.length;
        for (uint256 i = 0; i < crossMsgLength; ) {
            totalValue += messages[i].message.value;
            unchecked {
                ++i;
            }
        }

        if (subnet.circSupply < totalValue) {
            revert NotEnoughSubnetCircSupply();
        }

        subnet.circSupply -= totalValue;

        subnet.prevCheckpoint = commit;

        _applyMessages(commit.subnetID, messages);

        // TODO: distribute rewards to validators for their service executing a checkpoint
    }

    /// @notice commit the ipc parent finality into storage
    /// @param finality - the parent finality
    function commitParentFinality(ParentFinality calldata finality) external systemActorOnly {
        LibGateway.commitParentFinality(finality);
    }

    /// @notice Store the validator change requests from parent.
    /// @param changeRequests - the validator changes
    function storeValidatorChanges(StakingChangeRequest[] calldata changeRequests) external systemActorOnly {
        s.validatorsTracker.batchStoreChange(changeRequests);
    }

    /// @notice THIS METHOD IS DEPRECATED. It will be replaced with validator changes. Keep now to ensure tests runs.
    /// @notice Update the membership.
    function updateMembership(
        uint64 n,
        FvmAddress[] calldata validators,
        uint256[] calldata weights
    ) external systemActorOnly {
        LibGateway.newMembership({n: n, validators: validators, weights: weights});
    }

    /// @notice apply cross messages
    function applyCrossMessages(CrossMsg[] calldata crossMsgs) external systemActorOnly {
        _applyMessages(SubnetID(0, new address[](0)), crossMsgs);
    }

    /// @notice executes a cross message if its destination is the current network, otherwise adds it to the postbox to be propagated further
    /// @param forwarder - the subnet that handles the cross message
    /// @param crossMsg - the cross message to be executed
    function _applyMsg(SubnetID memory forwarder, CrossMsg memory crossMsg) internal {
        if (crossMsg.message.to.subnetId.isEmpty()) {
            revert InvalidCrossMsgDstSubnet();
        }
        if (crossMsg.message.method == METHOD_SEND) {
            if (crossMsg.message.value > address(this).balance) {
                revert NotEnoughBalance();
            }
        }

        IPCMsgType applyType = crossMsg.message.applyType(s.networkName);

        // If the cross-message destination is the current network.
        if (crossMsg.message.to.subnetId.equals(s.networkName)) {
            // forwarder will always be empty subnet when we reach here from submitTopDownCheckpoint
            // so we check against it to not reach here in coverage

            if (applyType == IPCMsgType.BottomUp) {
                if (!forwarder.isEmpty()) {
                    (bool registered, Subnet storage subnet) = LibGateway.getSubnet(forwarder);
                    if (!registered) {
                        revert NotRegisteredSubnet();
                    }
                    if (subnet.appliedBottomUpNonce != crossMsg.message.nonce) {
                        revert InvalidCrossMsgNonce();
                    }

                    subnet.appliedBottomUpNonce += 1;
                }
            }

            if (applyType == IPCMsgType.TopDown) {
                if (s.appliedTopDownNonce != crossMsg.message.nonce) {
                    revert InvalidCrossMsgNonce();
                }
                s.appliedTopDownNonce += 1;
            }

            // slither-disable-next-line unused-return
            crossMsg.execute();
            return;
        }

        // when the destination is not the current network we add it to the postbox for further propagation
        bytes32 cid = crossMsg.toHash();

        s.postbox[cid] = crossMsg;
    }

    /// @notice applies a cross-net messages coming from some other subnet.
    /// The forwarder argument determines the previous subnet that submitted the checkpoint triggering the cross-net message execution.
    /// @param forwarder - the subnet that handles the messages
    /// @param crossMsgs - the cross-net messages to apply
    function _applyMessages(SubnetID memory forwarder, CrossMsg[] memory crossMsgs) internal {
        uint256 crossMsgsLength = crossMsgs.length;
        for (uint256 i = 0; i < crossMsgsLength; ) {
            _applyMsg(forwarder, crossMsgs[i]);
            unchecked {
                ++i;
            }
        }
    }

    /// @notice checks whether the provided checkpoint signature for the block at height `height` is valid and accumulates that it
    /// @dev If adding the signature leads to reaching the threshold, then the checkpoint is removed from `incompleteCheckpoints`
    /// @param height - the height of the block in the checkpoint
    /// @param membershipProof - a Merkle proof that the validator was in the membership at height `height` with weight `weight`
    /// @param weight - the weight of the validator
    /// @param signature - the signature of the checkpoint
    function addCheckpointSignature(
        uint64 height,
        bytes32[] memory membershipProof,
        uint256 weight,
        bytes memory signature
    ) external {
        if (height < s.bottomUpCheckpointRetentionHeight) {
            revert CheckpointAlreadyProcessed();
        }
        BottomUpCheckpoint memory checkpoint = s.bottomUpCheckpoints[height];
        if (checkpoint.blockHeight == 0) {
            revert CheckpointNotCreated();
        }

        CheckpointInfo storage checkpointInfo = s.bottomUpCheckpointInfo[height];
        if (checkpointInfo.threshold == 0) {
            revert CheckpointMembershipNotCreated();
        }

        bytes32 checkpointHash = checkpointInfo.hash;

        // slither-disable-next-line unused-return
        (address recoveredSignatory, ECDSA.RecoverError err, ) = ECDSA.tryRecover(checkpointHash, signature);
        if (err != ECDSA.RecoverError.NoError) {
            revert InvalidSignature();
        }

        // Check whether the validator has already sent a valid signature
        if (s.bottomUpSignatureSenders[height].contains(recoveredSignatory)) {
            revert SignatureReplay();
        }

        // The validator is allowed to send a signature if it was in the membership at the target height
        // Constructing leaf: https://github.com/OpenZeppelin/merkle-tree#leaf-hash
        bytes32 validatorLeaf = keccak256(bytes.concat(keccak256(abi.encode(recoveredSignatory, weight))));
        bool valid = MerkleProof.verify({proof: membershipProof, root: checkpointInfo.rootHash, leaf: validatorLeaf});
        if (!valid) {
            revert NotAuthorized(recoveredSignatory);
        }

        // All checks passed.
        // Adding signature and emitting events.

        bool ok = s.bottomUpSignatureSenders[height].add(recoveredSignatory);
        if (!ok) {
            revert FailedAddSignatory();
        }
        s.bottomUpSignatures[height][recoveredSignatory] = signature;
        checkpointInfo.currentWeight += weight;

        if (checkpointInfo.currentWeight >= checkpointInfo.threshold) {
            if (!checkpointInfo.reached) {
                checkpointInfo.reached = true;
                // checkpoint is completed since the threshold has been reached
                ok = s.incompleteCheckpoints.remove(checkpoint.blockHeight);
                if (!ok) {
                    revert FailedRemoveIncompleteCheckpoint();
                }
                emit QuorumReached({
                    height: height,
                    checkpoint: checkpointHash,
                    quorumWeight: checkpointInfo.currentWeight
                });
            } else {
                emit QuorumWeightUpdated({
                    height: height,
                    checkpoint: checkpointHash,
                    newWeight: checkpointInfo.currentWeight
                });
            }
        }
    }

    /// @notice creates a new bottom-up checkpoint
    /// @param checkpoint - a bottom-up checkpoint
    /// @param membershipRootHash - a root hash of the Merkle tree built from the validator public keys and their weight
    /// @param membershipWeight - the total weight of the membership
    function createBottomUpCheckpoint(
        BottomUpCheckpoint calldata checkpoint,
        bytes32 membershipRootHash,
        uint256 membershipWeight
    ) external systemActorOnly {
        if (checkpoint.blockHeight < s.bottomUpCheckpointRetentionHeight) {
            revert CheckpointAlreadyProcessed();
        }
        if (checkpoint.blockHeight % s.bottomUpCheckPeriod != 0) {
            revert InvalidCheckpointEpoch();
        }
        if (s.bottomUpCheckpoints[checkpoint.blockHeight].blockHeight > 0) {
            revert CheckpointAlreadyExists();
        }
        if (s.bottomUpCheckpointInfo[checkpoint.blockHeight].threshold > 0) {
            revert CheckpointInfoAlreadyExists();
        }

        if (membershipWeight == 0) {
            revert ZeroMembershipWeight();
        }

        uint256 threshold = LibGateway.weightNeeded(membershipWeight, s.majorityPercentage);

        // process the checkpoint
        s.bottomUpCheckpoints[checkpoint.blockHeight] = checkpoint;
        bool ok = s.incompleteCheckpoints.add(checkpoint.blockHeight);
        if (!ok) {
            revert FailedAddIncompleteCheckpoint();
        }
        s.bottomUpCheckpointInfo[checkpoint.blockHeight] = CheckpointInfo({
            hash: checkpoint.toHash(),
            rootHash: membershipRootHash,
            threshold: threshold,
            currentWeight: 0,
            reached: false
        });
    }

    /// @notice Set a new checkpoint retention height and garbage collect all checkpoints in range [`retentionHeight`, `newRetentionHeight`)
    /// @dev `retentionHeight` is the height of the first incomplete checkpointswe must keep to implement checkpointing.
    /// All checkpoints with a height less than `retentionHeight` are removed from the history, assuming they are committed to the parent.
    /// @param newRetentionHeight - the height of the oldest checkpoint to keep
    function pruneBottomUpCheckpoints(uint64 newRetentionHeight) external systemActorOnly {
        uint64 oldRetentionHeight = s.bottomUpCheckpointRetentionHeight;

        if (newRetentionHeight <= oldRetentionHeight) {
            revert InvalidRetentionHeight();
        }

        for (uint64 h = oldRetentionHeight; h < newRetentionHeight; ) {
            delete s.bottomUpCheckpoints[h];
            delete s.bottomUpCheckpointInfo[h];
            delete s.bottomUpSignatureSenders[h];
            delete s.bottomUpMessages[h];

            address[] memory validators = s.bottomUpSignatureSenders[h].values();
            uint256 n = validators.length;

            for (uint256 i = 0; i < n; ) {
                delete s.bottomUpSignatures[h][validators[i]];
                unchecked {
                    ++i;
                }
            }

            unchecked {
                ++h;
            }
        }

        s.bottomUpCheckpointRetentionHeight = newRetentionHeight;
    }
}
