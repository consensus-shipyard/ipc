// SPDX-License-Identifier: UNLICENSED
pragma solidity 0.8.19;

import {IGateway} from "../interfaces/IGateway.sol";
import {GatewayActorStorage, LibGatewayActorStorage} from "../lib/LibGatewayActorStorage.sol";
import {ISubnetActor} from "../interfaces/ISubnetActor.sol";
import {SubnetID, Subnet} from "../structs/Subnet.sol";
import {BottomUpCheckpoint, CrossMsg} from "../structs/Checkpoint.sol";
import {CrossMsg, BottomUpCheckpoint, TopDownCheckpoint, StorableMsg} from "../structs/Checkpoint.sol";
import {NotRegisteredSubnet, InvalidActorAddress, EpochAlreadyExecuted, EpochNotVotable, ValidatorAlreadyVoted} from "../errors/IPCErrors.sol";
import {EpochVoteTopDownSubmission} from "../structs/EpochVoteSubmission.sol";
import {ExecutableQueue} from "../structs/ExecutableQueue.sol";
import {AccountHelper} from "./AccountHelper.sol";
import {Address} from "openzeppelin-contracts/utils/Address.sol";
import {ExecutableQueue} from "../structs/ExecutableQueue.sol";
import {EpochVoteSubmission} from "../structs/EpochVoteSubmission.sol";
import {VoteExecutionStatus} from "../enums/VoteExecutionStatus.sol";
import {ExecutableQueueHelper} from "../lib/ExecutableQueueHelper.sol";
import {EpochVoteSubmissionHelper} from "../lib/EpochVoteSubmissionHelper.sol";
import {FilAddress} from "fevmate/utils/FilAddress.sol";
import {CheckpointHelper} from "../lib/CheckpointHelper.sol";
import {AccountHelper} from "../lib/AccountHelper.sol";
import {CrossMsgHelper} from "../lib/CrossMsgHelper.sol";
import {ExecutableQueue} from "../structs/ExecutableQueue.sol";
import {EpochVoteSubmission} from "../structs/EpochVoteSubmission.sol";
import {VoteExecutionStatus} from "../enums/VoteExecutionStatus.sol";
import {ExecutableQueueHelper} from "../lib/ExecutableQueueHelper.sol";
import {EpochVoteSubmissionHelper} from "../lib/EpochVoteSubmissionHelper.sol";
import {SubnetIDHelper} from "../lib/SubnetIDHelper.sol";
import {LibVoting} from "../lib/LibVoting.sol";

library LibGateway {
    using FilAddress for address;
    using FilAddress for address payable;
    using AccountHelper for address;
    using SubnetIDHelper for SubnetID;
    using CrossMsgHelper for CrossMsg;
    using CheckpointHelper for BottomUpCheckpoint;
    using CheckpointHelper for TopDownCheckpoint;
    using ExecutableQueueHelper for ExecutableQueue;
    using EpochVoteSubmissionHelper for EpochVoteTopDownSubmission;
    using ExecutableQueueHelper for ExecutableQueue;
    using EpochVoteSubmissionHelper for EpochVoteSubmission;

    /// @notice returns the current bottom-up checkpoint
    /// @return exists - whether the checkpoint exists
    /// @return epoch - the epoch of the checkpoint
    /// @return checkpoint - the checkpoint struct
    function getCurrentBottomUpCheckpoint()
        internal
        view
        returns (bool exists, uint64 epoch, BottomUpCheckpoint storage checkpoint)
    {
        GatewayActorStorage storage s = LibGatewayActorStorage.appStorage();
        epoch = LibVoting.getNextEpoch(block.number, s.bottomUpCheckPeriod);
        checkpoint = s.bottomUpCheckpoints[epoch];
        exists = !checkpoint.source.isEmpty();
    }

    /// @notice commit topdown messages for their execution in the subnet. Adds the message to the subnet struct for future execution
    /// @param crossMessage - the cross message to be committed
    function commitTopDownMsg(CrossMsg memory crossMessage) internal {
        GatewayActorStorage storage s = LibGatewayActorStorage.appStorage();
        SubnetID memory subnetId = crossMessage.message.to.subnetId.down(s.networkName);

        (bool registered, Subnet storage subnet) = getSubnet(subnetId);

        if (!registered) {
            revert NotRegisteredSubnet();
        }

        crossMessage.message.nonce = subnet.topDownNonce;
        subnet.topDownNonce += 1;
        subnet.circSupply += crossMessage.message.value;
        subnet.topDownMsgs.push(crossMessage);
    }

    /// @notice commit bottomup messages for their execution in the subnet. Adds the message to the checkpoint for future execution
    /// @param crossMessage - the cross message to be committed
    function commitBottomUpMsg(CrossMsg memory crossMessage) internal {
        GatewayActorStorage storage s = LibGatewayActorStorage.appStorage();
        (, , BottomUpCheckpoint storage checkpoint) = getCurrentBottomUpCheckpoint();

        crossMessage.message.nonce = s.bottomUpNonce;

        checkpoint.fee += s.crossMsgFee;
        checkpoint.crossMsgs.push(crossMessage);
        s.bottomUpNonce += 1;
    }

    /// @notice distribute rewards to validators in child subnet
    /// @param to - the address of the target subnet contract
    /// @param amount - the amount of rewards to distribute
    function distributeRewards(address to, uint256 amount) internal {
        if (amount == 0) {
            return;
        }
        // slither-disable-next-line unused-return
        Address.functionCall(to.normalize(), abi.encodeCall(ISubnetActor.reward, amount));
    }

    /// @notice returns the subnet created by a validator
    /// @param actor the validator that created the subnet
    /// @return found whether the subnet exists
    /// @return subnet -  the subnet struct
    function getSubnet(address actor) internal view returns (bool found, Subnet storage subnet) {
        GatewayActorStorage storage s = LibGatewayActorStorage.appStorage();
        if (actor == address(0)) {
            revert InvalidActorAddress();
        }
        SubnetID memory subnetId = s.networkName.createSubnetId(actor);

        return getSubnet(subnetId);
    }

    /// @notice returns the subnet with the given id
    /// @param subnetId the id of the subnet
    /// @return found whether the subnet exists
    /// @return subnet -  the subnet struct
    function getSubnet(SubnetID memory subnetId) internal view returns (bool found, Subnet storage subnet) {
        GatewayActorStorage storage s = LibGatewayActorStorage.appStorage();
        subnet = s.subnets[subnetId.toHash()];
        found = !subnet.id.isEmpty();
    }
}
