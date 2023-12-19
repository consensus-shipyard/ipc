// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity 0.8.19;

import {ISubnetActor} from "../interfaces/ISubnetActor.sol";
import {GatewayActorModifiers} from "../lib/LibGatewayActorStorage.sol";
import {METHOD_SEND} from "../constants/Constants.sol";
import {CrossMsg, StorableMsg, ParentFinality, BottomUpCheckpoint, BottomUpMsgBatch} from "../structs/CrossNet.sol";
import {QuorumObjKind} from "../structs/Quorum.sol";
import {Status} from "../enums/Status.sol";
import {IPCMsgType} from "../enums/IPCMsgType.sol";
import {SubnetID, Subnet, Validator, ValidatorInfo, ValidatorSet} from "../structs/Subnet.sol";
import {IPCMsgType} from "../enums/IPCMsgType.sol";
import {Membership, SupplySource} from "../structs/Subnet.sol";
import {BatchNotCreated, InvalidBatchEpoch, BatchAlreadyExists, NotEnoughSubnetCircSupply, InvalidCheckpointEpoch} from "../errors/IPCErrors.sol";
import {InvalidBatchSource, NotEnoughBalance, MaxMsgsPerBatchExceeded, BatchWithNoMessages, InvalidCheckpointSource, InvalidCrossMsgNonce, InvalidCrossMsgDstSubnet, CheckpointAlreadyExists} from "../errors/IPCErrors.sol";
import {NotRegisteredSubnet, SubnetNotActive, SubnetNotFound, InvalidSubnet, CheckpointNotCreated} from "../errors/IPCErrors.sol";
import {StorableMsgHelper} from "../lib/StorableMsgHelper.sol";
import {SubnetIDHelper} from "../lib/SubnetIDHelper.sol";
import {CrossMsgHelper} from "../lib/CrossMsgHelper.sol";
import {LibGateway} from "../lib/LibGateway.sol";
import {LibQuorum} from "../lib/LibQuorum.sol";
import {FilAddress} from "fevmate/utils/FilAddress.sol";
import {StakingChangeRequest, ParentValidatorsTracker} from "../structs/Subnet.sol";
import {LibValidatorTracking, LibValidatorSet} from "../lib/LibStaking.sol";
import {Address} from "openzeppelin-contracts/utils/Address.sol";
import {SubnetActorGetterFacet} from "../subnet/SubnetActorGetterFacet.sol";
import {SupplySourceHelper} from "../lib/SupplySourceHelper.sol";

contract GatewayRouterFacet is GatewayActorModifiers {
    using FilAddress for address;
    using SubnetIDHelper for SubnetID;
    using CrossMsgHelper for CrossMsg;
    using StorableMsgHelper for StorableMsg;
    using LibValidatorTracking for ParentValidatorsTracker;
    using LibValidatorSet for ValidatorSet;
    using SupplySourceHelper for SupplySource;

    /// @notice submit a verified checkpoint in the gateway to trigger side-effects.
    /// @dev this method is called by the corresponding subnet actor.
    /// Called from a subnet actor if the checkpoint is cryptographically valid.
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
        // only active subnets can commit checkpoints
        if (subnet.status != Status.Active) {
            revert SubnetNotActive();
        }

        if (s.checkpointRelayerRewards) {
            // slither-disable-next-line unused-return
            Address.functionCallWithValue({
                target: msg.sender,
                data: abi.encodeCall(
                    ISubnetActor.distributeRewardToRelayers,
                    (checkpoint.blockHeight, 0, QuorumObjKind.Checkpoint)
                ),
                value: 0
            });
        }
    }

    /// @notice submit a batch of cross-net messages for execution.
    /// @dev this method is called by the corresponding subnet actor.
    /// Called from a subnet actor if the batch is valid.
    function execBottomUpMsgBatch(BottomUpMsgBatch calldata batch) external {
        if (batch.subnetID.getActor() != msg.sender) {
            revert InvalidBatchSource();
        }

        _checkMsgLength(batch);

        (bool subnetExists, Subnet storage subnet) = LibGateway.getSubnet(msg.sender);
        if (!subnetExists) {
            revert SubnetNotFound();
        }
        // cross-net messages can't be executed in inactive subnets.
        if (subnet.status != Status.Active) {
            revert SubnetNotActive();
        }

        uint256 totalValue;
        uint256 totalFee;
        uint256 crossMsgLength = batch.msgs.length;
        for (uint256 i; i < crossMsgLength; ) {
            totalValue += batch.msgs[i].message.value;
            totalFee += batch.msgs[i].message.fee;
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
        _applyMessages(subnet.id, batch.msgs);

        if (s.crossMsgRelayerRewards) {
            // reward relayers in the subnet for committing the previous checkpoint
            // slither-disable-next-line unused-return
            Address.functionCallWithValue({
                target: msg.sender,
                data: abi.encodeCall(
                    ISubnetActor.distributeRewardToRelayers,
                    (block.number, totalFee, QuorumObjKind.Checkpoint)
                ),
                value: totalFee
            });
        }
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

    /// @notice Applies top-down crossnet messages locally. This is invoked by IPC nodes when drawing messages from
    ///         their parent subnet for local execution. That's why the sender is restricted to the system sender,
    ///         because this method is implicitly invoked by the node during block production.
    function applyCrossMessages(CrossMsg[] calldata crossMsgs) external systemActorOnly {
        _applyMessages(s.networkName.getParentSubnet(), crossMsgs);
    }

    /// @notice executes a cross message if its destination is the current network, otherwise adds it to the postbox to be propagated further
    /// @param arrivingFrom - the immediate subnet from which this message is arriving
    /// @param crossMsg - the cross message to be executed
    function _applyMsg(SubnetID memory arrivingFrom, CrossMsg memory crossMsg) internal {
        if (crossMsg.message.to.subnetId.isEmpty()) {
            revert InvalidCrossMsgDstSubnet();
        }

        // If the crossnet destination is NOT the current network (network where the gateway is running),
        // we add it to the postbox for further propagation.
        if (!crossMsg.message.to.subnetId.equals(s.networkName)) {
            bytes32 cid = crossMsg.toHash();
            s.postbox[cid] = crossMsg;
            return;
        }

        // Now, let's find out the directionality of this message and act accordingly.
        // slither-disable-next-line uninitialized-local
        SupplySource memory supplySource;
        IPCMsgType applyType = crossMsg.message.applyType(s.networkName);
        if (applyType == IPCMsgType.BottomUp) {
            // Load the subnet this message is coming from. Ensure that it exists and that the nonce expectation is met.
            (bool registered, Subnet storage subnet) = LibGateway.getSubnet(arrivingFrom);
            if (!registered) {
                revert NotRegisteredSubnet();
            }
            if (subnet.appliedBottomUpNonce != crossMsg.message.nonce) {
                revert InvalidCrossMsgNonce();
            }
            subnet.appliedBottomUpNonce += 1;

            // The value carried in bottom-up messages needs to be treated according to the supply source
            // configuration of the subnet.
            supplySource = SubnetActorGetterFacet(subnet.id.getActor()).supplySource();
        } else if (applyType == IPCMsgType.TopDown) {
            // Note: there is no need to load the subnet, as a top-down application means that _we_ are the subnet.
            if (s.appliedTopDownNonce != crossMsg.message.nonce) {
                revert InvalidCrossMsgNonce();
            }
            s.appliedTopDownNonce += 1;

            // The value carried in top-down messages locally maps to the native coin, so we pass over the
            // native supply source.
            supplySource = SupplySourceHelper.native();
        }

        // slither-disable-next-line unused-return
        crossMsg.execute(supplySource);
    }

    /// @notice applies a cross-net messages coming from some other subnet.
    /// The forwarder argument determines the previous subnet that submitted the checkpoint triggering the cross-net message execution.
    /// @param arrivingFrom - the immediate subnet from which this message is arriving
    /// @param crossMsgs - the cross-net messages to apply
    function _applyMessages(SubnetID memory arrivingFrom, CrossMsg[] memory crossMsgs) internal {
        uint256 crossMsgsLength = crossMsgs.length;
        for (uint256 i; i < crossMsgsLength; ) {
            _applyMsg(arrivingFrom, crossMsgs[i]);
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

    /// @notice creates a new bottom-up checkpoint
    /// @param checkpoint - a bottom-up checkpoint
    /// @param membershipRootHash - a root hash of the Merkle tree built from the validator public keys and their weight
    /// @param membershipWeight - the total weight of the membership
    function createBottomUpCheckpoint(
        BottomUpCheckpoint calldata checkpoint,
        bytes32 membershipRootHash,
        uint256 membershipWeight
    ) external systemActorOnly {
        if (checkpoint.blockHeight % s.bottomUpCheckPeriod != 0) {
            revert InvalidCheckpointEpoch();
        }
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
    }

    /// @notice Set a new checkpoint retention height and garbage collect all checkpoints in range [`retentionHeight`, `newRetentionHeight`)
    /// @dev `retentionHeight` is the height of the first incomplete checkpointswe must keep to implement checkpointing.
    /// All checkpoints with a height less than `retentionHeight` are removed from the history, assuming they are committed to the parent.
    /// @param newRetentionHeight - the height of the oldest checkpoint to keep
    function pruneBottomUpCheckpoints(uint256 newRetentionHeight) external systemActorOnly {
        // we need to clean manually the checkpoints because Solidity does not support passing
        // a storage variable as an interface (so we can iterate and remove directly inside pruneQuorums)
        for (uint256 h = s.checkpointQuorumMap.retentionHeight; h < newRetentionHeight; ) {
            delete s.bottomUpCheckpoints[h];
            unchecked {
                ++h;
            }
        }

        LibQuorum.pruneQuorums(s.checkpointQuorumMap, newRetentionHeight);
    }

    /// @notice checks whether the provided batch signature for the block at height `height` is valid and accumulates that
    /// @param height - the height of the block in the checkpoint
    /// @param membershipProof - a Merkle proof that the validator was in the membership at height `height` with weight `weight`
    /// @param weight - the weight of the validator
    /// @param signature - the signature of the checkpoint
    function addBottomUpMsgBatchSignature(
        uint256 height,
        bytes32[] memory membershipProof,
        uint256 weight,
        bytes memory signature
    ) external {
        LibQuorum.isHeightAlreadyProcessed(s.bottomUpMsgBatchQuorumMap, height);

        // slither-disable-next-line unused-return
        (bool exists, ) = LibGateway.getBottomUpMsgBatch(height);
        if (!exists) {
            revert BatchNotCreated();
        }
        LibQuorum.addQuorumSignature({
            self: s.bottomUpMsgBatchQuorumMap,
            height: height,
            membershipProof: membershipProof,
            weight: weight,
            signature: signature
        });
    }

    /// @notice cuts a new message batch if the batch period is reached without
    /// the maximum number of messages being reached.
    /// @param batch - a bottom-up batch
    /// @param membershipRootHash - a root hash of the Merkle tree built from the validator public keys and their weight
    /// @param membershipWeight - the total weight of the membership
    function createBottomUpMsgBatch(
        BottomUpMsgBatch calldata batch,
        bytes32 membershipRootHash,
        uint256 membershipWeight
    ) external systemActorOnly {
        _checkMsgLength(batch);
        // We only externally trigger new batches if the maximum number
        // of messages for the batch hasn't been reached.
        // We also check that we are not trying to create a batch from
        // the future
        if (batch.blockHeight % s.bottomUpMsgBatchPeriod != 0 || block.number <= batch.blockHeight) {
            revert InvalidBatchEpoch();
        }

        if (LibGateway.bottomUpBatchMsgsExists(batch.blockHeight)) {
            revert BatchAlreadyExists();
        }

        LibQuorum.createQuorumInfo({
            self: s.bottomUpMsgBatchQuorumMap,
            objHeight: batch.blockHeight,
            objHash: keccak256(abi.encode(batch)),
            membershipRootHash: membershipRootHash,
            membershipWeight: membershipWeight,
            majorityPercentage: s.majorityPercentage
        });
        LibGateway.storeBottomUpMsgBatch(batch);
    }

    /// @notice Set a new batch retention height and garbage collect all batches in range [`retentionHeight`, `newRetentionHeight`)
    /// @param newRetentionHeight - the height of the oldest batch to keep
    function pruneBottomUpMsgBatches(uint256 newRetentionHeight) external systemActorOnly {
        for (uint256 h = s.bottomUpMsgBatchQuorumMap.retentionHeight; h < newRetentionHeight; ) {
            delete s.bottomUpMsgBatches[h];
            unchecked {
                ++h;
            }
        }

        LibQuorum.pruneQuorums(s.bottomUpMsgBatchQuorumMap, newRetentionHeight);
    }

    function _checkMsgLength(BottomUpMsgBatch memory batch) internal view {
        if (batch.msgs.length > s.maxMsgsPerBottomUpBatch) {
            revert MaxMsgsPerBatchExceeded();
        }
        if (batch.msgs.length == 0) {
            revert BatchWithNoMessages();
        }
    }
}
