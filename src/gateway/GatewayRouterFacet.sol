// SPDX-License-Identifier: UNLICENSED
pragma solidity 0.8.19;

import {GatewayActorModifiers} from "../lib/LibGatewayActorStorage.sol";
import {EMPTY_HASH, BURNT_FUNDS_ACTOR, METHOD_SEND} from "../constants/Constants.sol";
import {CrossMsg, BottomUpCheckpoint, TopDownCheckpoint, StorableMsg} from "../structs/Checkpoint.sol";
import {EpochVoteTopDownSubmission} from "../structs/EpochVoteSubmission.sol";
import {Status} from "../enums/Status.sol";
import {IPCMsgType} from "../enums/IPCMsgType.sol";
import {SubnetID, Subnet} from "../structs/Subnet.sol";
import {InconsistentPrevCheckpoint, CannotSendCrossMsgToItself, NotEnoughSubnetCircSupply, InvalidCheckpointEpoch, InvalidCheckpointSource, InvalidCrossMsgNonce, InvalidCrossMsgDestinationSubnet} from "../errors/IPCErrors.sol";
import {InvalidCrossMsgFromSubnetId, MessagesNotSorted, NotInitialized, NotEnoughBalance, NotEnoughFunds, NotRegisteredSubnet, NotValidator, PostboxNotExist, SubnetNotActive} from "../errors/IPCErrors.sol";
import {SubnetIDHelper} from "../lib/SubnetIDHelper.sol";
import {CheckpointHelper} from "../lib/CheckpointHelper.sol";
import {LibVoting} from "../lib/LibVoting.sol";
import {CrossMsgHelper} from "../lib/CrossMsgHelper.sol";
import {LibGateway} from "../lib/LibGateway.sol";
import {StorableMsgHelper} from "../lib/StorableMsgHelper.sol";
import {FilAddress} from "fevmate/utils/FilAddress.sol";
import {Address} from "openzeppelin-contracts/utils/Address.sol";

contract GatewayRouterFacet is GatewayActorModifiers {
    using FilAddress for address;
    using FilAddress for address payable;
    using SubnetIDHelper for SubnetID;
    using CrossMsgHelper for CrossMsg;
    using CheckpointHelper for BottomUpCheckpoint;
    using CheckpointHelper for TopDownCheckpoint;
    using StorableMsgHelper for StorableMsg;

    /// @notice submit a checkpoint in the gateway. Called from a subnet once the checkpoint is voted for and reaches majority
    function commitChildCheck(BottomUpCheckpoint calldata commit) external {
        if (!s.initialized) {
            revert NotInitialized();
        }
        if (commit.source.getActor().normalize() != msg.sender) {
            revert InvalidCheckpointSource();
        }

        // slither-disable-next-line unused-return
        (, Subnet storage subnet) = LibGateway.getSubnet(msg.sender);
        if (subnet.status != Status.Active) {
            revert SubnetNotActive();
        }
        if (subnet.prevCheckpoint.epoch >= commit.epoch) {
            revert InvalidCheckpointEpoch();
        }
        if (commit.prevHash != EMPTY_HASH) {
            if (commit.prevHash != subnet.prevCheckpoint.toHash()) {
                revert InconsistentPrevCheckpoint();
            }
        }

        // get checkpoint for the current template being populated
        (bool checkpointExists, uint64 nextCheckEpoch, BottomUpCheckpoint storage checkpoint) = LibGateway
            .getCurrentBottomUpCheckpoint();

        // create a checkpoint template if it doesn't exists
        if (!checkpointExists) {
            checkpoint.source = s.networkName;
            checkpoint.epoch = nextCheckEpoch;
        }

        checkpoint.setChildCheck(commit, s.children, s.checks, nextCheckEpoch);

        uint256 totalValue = 0;
        uint256 crossMsgLength = commit.crossMsgs.length;
        for (uint256 i = 0; i < crossMsgLength; ) {
            totalValue += commit.crossMsgs[i].message.value;
            unchecked {
                ++i;
            }
        }

        totalValue += commit.fee + checkpoint.fee; // add fee that is already in checkpoint as well. For example from release message interacting with the same checkpoint

        if (subnet.circSupply < totalValue) {
            revert NotEnoughSubnetCircSupply();
        }

        subnet.circSupply -= totalValue;

        subnet.prevCheckpoint = commit;

        _applyMessages(commit.source, commit.crossMsgs);

        LibGateway.distributeRewards(msg.sender, commit.fee);
    }

    /// @notice allows a validator to submit a batch of messages in a top-down commitment
    /// @param checkpoint - top-down checkpoint
    function submitTopDownCheckpoint(TopDownCheckpoint calldata checkpoint) external {
        // use this instead of the validEpochOnly modifier
        LibVoting.applyValidEpochOnly(checkpoint.epoch);

        uint256 validatorWeight = s.validatorSet[s.validatorNonce][msg.sender];

        if (!s.initialized) {
            revert NotInitialized();
        }
        if (validatorWeight == 0) {
            revert NotValidator();
        }
        if (!CrossMsgHelper.isSorted(checkpoint.topDownMsgs)) {
            revert MessagesNotSorted();
        }

        EpochVoteTopDownSubmission storage voteSubmission = s.epochVoteSubmissions[checkpoint.epoch];

        // submit the vote
        bool shouldExecuteVote = _submitTopDownVote(voteSubmission, checkpoint, msg.sender, validatorWeight);

        // slither-disable-next-line uninitialized-local
        CrossMsg[] memory topDownMsgs;

        if (shouldExecuteVote) {
            topDownMsgs = _markMostVotedSubmissionExecuted(voteSubmission);
        }

        // no messages executed in the current submission, let's get the next executable epoch from the queue to see if it can be executed already
        if (topDownMsgs.length == 0) {
            (uint64 nextExecutableEpoch, bool isExecutableEpoch) = LibVoting.getNextExecutableEpoch();

            if (isExecutableEpoch) {
                EpochVoteTopDownSubmission storage nextVoteSubmission = s.epochVoteSubmissions[nextExecutableEpoch];

                topDownMsgs = _markMostVotedSubmissionExecuted(nextVoteSubmission);
            }
        }

        //only execute the messages and update the last executed checkpoint when we have majority
        _applyMessages(SubnetID(0, new address[](0)), topDownMsgs);
    }

    /// @notice sends an arbitrary cross message from the current subnet to the destination subnet
    /// @param crossMsg - message to send
    function sendCrossMessage(CrossMsg calldata crossMsg) external payable hasFee {
        // There can be many semantics of the (rawAddress, msg.sender) pairs.
        // It depends on who is allowed to call sendCrossMessage method and what we want to get as a result.
        // They can be equal, we can propagate the real sender address only or both.
        // We are going to use the simplest implementation for now and define the appropriate interpretation later
        // based on the business requirements.
        if (crossMsg.message.value != msg.value) {
            revert NotEnoughFunds();
        }

        // We disregard the "to" of the message that will be verified in the _commitCrossMessage().
        // The caller is the one set as the "from" of the message
        if (!crossMsg.message.from.subnetId.equals(s.networkName)) {
            revert InvalidCrossMsgFromSubnetId();
        }

        // commit cross-message for propagation
        (bool shouldBurn, bool shouldDistributeRewards) = _commitCrossMessage(crossMsg);

        _crossMsgSideEffects(
            crossMsg.message.value,
            crossMsg.message.to.subnetId.down(s.networkName),
            shouldBurn,
            shouldDistributeRewards
        );
    }

    /// @notice propagates the populated cross net message for the given cid
    /// @param msgCid - the cid of the cross-net message
    function propagate(bytes32 msgCid) external payable hasFee {
        CrossMsg storage crossMsg = s.postbox[msgCid];

        (bool shouldBurn, bool shouldDistributeRewards) = _commitCrossMessage(crossMsg);
        // We must delete the message first to prevent potential re-entrancies,
        // and as the message is deleted and we don't have a reference to the object
        // anymore, we need to pull the data from the message to trigger the side-effects.
        uint256 v = crossMsg.message.value;
        SubnetID memory toSubnetId = crossMsg.message.to.subnetId.down(s.networkName);
        delete s.postbox[msgCid];

        _crossMsgSideEffects(v, toSubnetId, shouldBurn, shouldDistributeRewards);

        uint256 feeRemainder = msg.value - s.crossMsgFee;

        if (feeRemainder > 0) {
            payable(msg.sender).sendValue(feeRemainder);
        }
    }

    /// @notice marks a checkpoint as executed based on the last vote that reached majority
    /// @notice voteSubmission - the vote submission data
    /// @return the cross messages that should be executed
    function _markMostVotedSubmissionExecuted(
        EpochVoteTopDownSubmission storage voteSubmission
    ) internal returns (CrossMsg[] storage) {
        TopDownCheckpoint storage mostVotedSubmission = voteSubmission.submissions[
            voteSubmission.vote.mostVotedSubmission
        ];

        LibVoting.markSubmissionExecuted(mostVotedSubmission.epoch);

        return mostVotedSubmission.topDownMsgs;
    }

    /// @notice submits a vote for a checkpoint
    /// @param voteSubmission - the vote submission data
    /// @param submitterAddress - the validator that submits the vote
    /// @param submitterWeight - the weight of the validator
    /// @return shouldExecuteVote - flag if the checkpoint should be executed based on the vote
    function _submitTopDownVote(
        EpochVoteTopDownSubmission storage voteSubmission,
        TopDownCheckpoint calldata submission,
        address submitterAddress,
        uint256 submitterWeight
    ) internal returns (bool shouldExecuteVote) {
        bytes32 submissionHash = submission.toHash();

        shouldExecuteVote = LibVoting.submitVote(
            voteSubmission.vote,
            submissionHash,
            submitterAddress,
            submitterWeight,
            submission.epoch,
            s.totalWeight
        );

        // store the submission only the first time
        if (voteSubmission.submissions[submissionHash].isEmpty()) {
            voteSubmission.submissions[submissionHash] = submission;
        }
    }

    /// @notice Commit the cross message to storage. It outputs a flag signaling
    /// if the committed messages was bottom-up and some funds need to be
    /// burnt or if a top-down message fee needs to be distributed.
    ///
    /// It also validates that destination subnet ID is not empty
    /// and not equal to the current network.
    function _commitCrossMessage(
        CrossMsg memory crossMessage
    ) internal returns (bool shouldBurn, bool shouldDistributeRewards) {
        SubnetID memory to = crossMessage.message.to.subnetId;
        if (to.isEmpty()) {
            revert InvalidCrossMsgDestinationSubnet();
        }
        // destination is the current network, you are better off with a good old message, no cross needed
        if (to.equals(s.networkName)) {
            revert CannotSendCrossMsgToItself();
        }

        SubnetID memory from = crossMessage.message.from.subnetId;
        IPCMsgType applyType = crossMessage.message.applyType(s.networkName);

        // slither-disable-next-line uninitialized-local
        bool shouldCommitBottomUp;

        if (applyType == IPCMsgType.BottomUp) {
            shouldCommitBottomUp = !to.commonParent(from).equals(s.networkName);
        }

        if (shouldCommitBottomUp) {
            LibGateway.commitBottomUpMsg(crossMessage);

            return (shouldBurn = crossMessage.message.value > 0, shouldDistributeRewards = false);
        }

        if (applyType == IPCMsgType.TopDown) {
            ++s.appliedTopDownNonce;
        }

        LibGateway.commitTopDownMsg(crossMessage);

        return (shouldBurn = false, shouldDistributeRewards = true);
    }

    /// @notice transaction side-effects from the commitment of a cross-net message. It burns funds
    /// and propagates the corresponding rewards.
    /// @param v - the value of the committed cross-net message
    /// @param toSubnetId - the destination subnet of the committed cross-net message
    /// @param shouldBurn - flag if the message should burn funds
    /// @param shouldDistributeRewards - flag if the message should distribute rewards
    function _crossMsgSideEffects(
        uint256 v,
        SubnetID memory toSubnetId,
        bool shouldBurn,
        bool shouldDistributeRewards
    ) internal {
        if (shouldBurn) {
            payable(BURNT_FUNDS_ACTOR).sendValue(v);
        }

        if (shouldDistributeRewards) {
            LibGateway.distributeRewards(toSubnetId.getActor(), s.crossMsgFee);
        }
    }

    /// @notice executes a cross message if its destination is the current network, otherwise adds it to the postbox to be propagated further
    /// @param forwarder - the subnet that handles the cross message
    /// @param crossMsg - the cross message to be executed
    function _applyMsg(SubnetID memory forwarder, CrossMsg memory crossMsg) internal {
        if (crossMsg.message.to.subnetId.isEmpty()) {
            revert InvalidCrossMsgDestinationSubnet();
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
}
