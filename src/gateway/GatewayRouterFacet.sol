// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity 0.8.19;

import {ISubnetActor} from "../interfaces/ISubnetActor.sol";
import {GatewayActorModifiers} from "../lib/LibGatewayActorStorage.sol";
import {EMPTY_HASH, METHOD_SEND} from "../constants/Constants.sol";
import {CrossMsg, StorableMsg, ParentFinality, BottomUpCheckpoint, CheckpointInfo} from "../structs/Checkpoint.sol";
import {Status} from "../enums/Status.sol";
import {IPCMsgType} from "../enums/IPCMsgType.sol";
import {SubnetID, Subnet, Validator, ValidatorInfo, ValidatorSet} from "../structs/Subnet.sol";
import {IPCMsgType} from "../enums/IPCMsgType.sol";
import {Membership} from "../structs/Subnet.sol";
import {NotEnoughSubnetCircSupply, InvalidCheckpointEpoch, InvalidSignature, NotAuthorized, SignatureReplay, InvalidRetentionHeight, FailedRemoveIncompleteCheckpoint} from "../errors/IPCErrors.sol";
import {InvalidCheckpointSource, InvalidCrossMsgNonce, InvalidCrossMsgDstSubnet, CheckpointAlreadyExists, CheckpointInfoAlreadyExists, CheckpointAlreadyProcessed, FailedAddIncompleteCheckpoint, FailedAddSignatory} from "../errors/IPCErrors.sol";
import {NotEnoughBalance, NotRegisteredSubnet, SubnetNotActive, SubnetNotFound, InvalidSubnet, CheckpointNotCreated, ZeroMembershipWeight} from "../errors/IPCErrors.sol";
import {SubnetIDHelper} from "../lib/SubnetIDHelper.sol";
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
import {LibValidatorTracking, LibValidatorSet} from "../lib/LibStaking.sol";
import {Address} from "openzeppelin-contracts/utils/Address.sol";

contract GatewayRouterFacet is GatewayActorModifiers {
    using FilAddress for address;
    using SubnetIDHelper for SubnetID;
    using CrossMsgHelper for CrossMsg;
    using StorableMsgHelper for StorableMsg;
    using EnumerableSet for EnumerableSet.UintSet;
    using EnumerableSet for EnumerableSet.AddressSet;
    using LibValidatorTracking for ParentValidatorsTracker;
    using LibValidatorSet for ValidatorSet;

    event QuorumReached(uint64 height, bytes32 checkpoint, uint256 quorumWeight);
    event QuorumWeightUpdated(uint64 height, bytes32 checkpoint, uint256 newWeight);

    /// @notice submit a verified checkpoint in the gateway to trigger side-effects and apply cross-messages.
    /// @dev this method is called by the corresponding subnet actor.
    /// Called from a subnet actor if the checkpoint is cryptographically valid.
    function commitBottomUpCheckpoint(BottomUpCheckpoint calldata checkpoint, CrossMsg[] calldata messages) external {
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
        if (subnet.status != Status.Active) {
            revert SubnetNotActive();
        }

        uint256 totalValue;
        uint256 totalFee;
        uint256 crossMsgLength = messages.length;
        for (uint256 i; i < crossMsgLength; ) {
            totalValue += messages[i].message.value;
            totalFee += messages[i].message.fee;
            unchecked {
                ++i;
            }
        }

        uint256 totalAmount = totalFee + totalValue;

        if (subnet.circSupply < totalAmount) {
            revert NotEnoughSubnetCircSupply();
        }

        subnet.circSupply -= totalAmount;

        // execute cross-messages
        _applyMessages(checkpoint.subnetID, messages);

        // reward relayers in the subnet for committing the previous checkpoint
        // slither-disable-next-line unused-return
        Address.functionCallWithValue({
            target: msg.sender,
            data: abi.encodeCall(ISubnetActor.distributeRewardToRelayers, (checkpoint.blockHeight, totalFee)),
            value: totalFee
        });
    }

    /// @notice commit the ipc parent finality into storage and returns the previous committed finality
    /// This is useful to understand if the finalities are consistent or if there have been reorgs.
    /// If there are no previous committed fainality, it will be default to zero values, i.e. zero height and block hash.
    /// @param finality - the parent finality
    function commitParentFinality(
        ParentFinality calldata finality
    ) external systemActorOnly returns (bool hasCommittedBefore, ParentFinality memory previousFinality) {
        previousFinality = LibGateway.commitParentFinality(finality);
        hasCommittedBefore = previousFinality.height != 0;
    }

    /// @notice Store the validator change requests from parent.
    /// @param changeRequests - the validator changes
    function storeValidatorChanges(StakingChangeRequest[] calldata changeRequests) external systemActorOnly {
        s.validatorsTracker.batchStoreChange(changeRequests);
    }

    /// @notice Apply all changes committed through the commitment of parent finality
    function applyFinalityChanges() external systemActorOnly returns (uint64) {
        // get the latest configuration number for the change set
        uint64 configurationNumber = s.validatorsTracker.changes.nextConfigurationNumber - 1;
        // return immediately if there are no changes to confirm by looking at next configNumber
        if (
            // nextConfiguration == startConfiguration (i.e. no changes)
            (configurationNumber + 1) == s.validatorsTracker.changes.startConfigurationNumber
        ) {
            // 0 flags that there are no changes
            return 0;
        }
        // confirm the change
        s.validatorsTracker.confirmChange(configurationNumber);

        // get the active validators
        address[] memory validators = s.validatorsTracker.validators.listActiveValidators();
        uint256 vLength = validators.length;
        Validator[] memory vs = new Validator[](vLength);
        for (uint256 i; i < vLength; ) {
            address addr = validators[i];
            ValidatorInfo storage info = s.validatorsTracker.validators.validators[addr];
            vs[i] = Validator({weight: info.confirmedCollateral, addr: addr, metadata: info.metadata});
            unchecked {
                ++i;
            }
        }

        // update membership with the applied changes
        LibGateway.updateMembership(Membership({configurationNumber: configurationNumber, validators: vs}));
        return configurationNumber;
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

        IPCMsgType applyType = crossMsg.message.applyType(s.networkName);

        // If the crossnet destination is the current network (network where the gateway is running).
        if (crossMsg.message.to.subnetId.equals(s.networkName)) {
            if (applyType == IPCMsgType.BottomUp) {
                // Load the subnet this message is coming from. Ensure that it exists and that the nonce expectation is met.
                (bool registered, Subnet storage subnet) = LibGateway.getSubnet(forwarder);
                if (!registered) {
                    revert NotRegisteredSubnet();
                }
                if (subnet.appliedBottomUpNonce != crossMsg.message.nonce) {
                    revert InvalidCrossMsgNonce();
                }
                subnet.appliedBottomUpNonce += 1;
            } else if (applyType == IPCMsgType.TopDown) {
                // There is no need to load the subnet, as a top-down application means that _we_ are the subnet.
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
        for (uint256 i; i < crossMsgsLength; ) {
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

        (bool exists, BottomUpCheckpoint storage checkpoint, CheckpointInfo storage checkpointInfo) = LibGateway
            .getBottomUpCheckpointWithInfo(height);
        if (!exists) {
            revert CheckpointNotCreated();
        }

        // slither-disable-next-line unused-return
        (address recoveredSignatory, ECDSA.RecoverError err, ) = ECDSA.tryRecover(checkpointInfo.hash, signature);
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
                    checkpoint: checkpointInfo.hash,
                    quorumWeight: checkpointInfo.currentWeight
                });
            } else {
                emit QuorumWeightUpdated({
                    height: height,
                    checkpoint: checkpointInfo.hash,
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
        if (LibGateway.bottomUpCheckpointExists(checkpoint.blockHeight)) {
            revert CheckpointAlreadyExists();
        }

        if (membershipWeight == 0) {
            revert ZeroMembershipWeight();
        }

        uint256 threshold = LibGateway.weightNeeded(membershipWeight, s.majorityPercentage);

        // process the checkpoint
        bool ok = s.incompleteCheckpoints.add(checkpoint.blockHeight);
        if (!ok) {
            revert FailedAddIncompleteCheckpoint();
        }

        CheckpointInfo memory info = CheckpointInfo({
            hash: keccak256(abi.encode(checkpoint)),
            rootHash: membershipRootHash,
            threshold: threshold,
            currentWeight: 0,
            reached: false
        });
        LibGateway.storeBottomUpCheckpointWithInfo(checkpoint, info);
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

            for (uint256 i; i < n; ) {
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
