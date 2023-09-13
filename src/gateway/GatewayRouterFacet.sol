// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity 0.8.19;

import {GatewayActorModifiers} from "../lib/LibGatewayActorStorage.sol";
import {EMPTY_HASH, METHOD_SEND} from "../constants/Constants.sol";
import {CrossMsg, StorableMsg, ParentFinality, BottomUpCheckpoint} from "../structs/Checkpoint.sol";
import {EpochVoteTopDownSubmission} from "../structs/EpochVoteSubmission.sol";
import {Status} from "../enums/Status.sol";
import {IPCMsgType} from "../enums/IPCMsgType.sol";
import {SubnetID, Subnet} from "../structs/Subnet.sol";
import {InconsistentPrevCheckpoint, NotEnoughSubnetCircSupply, InvalidCheckpointEpoch} from "../errors/IPCErrors.sol";
import {InvalidCheckpointSource, InvalidCrossMsgNonce, InvalidCrossMsgDstSubnet} from "../errors/IPCErrors.sol";
import {MessagesNotSorted, NotInitialized, NotEnoughBalance, NotRegisteredSubnet} from "../errors/IPCErrors.sol";
import {NotValidator, SubnetNotActive} from "../errors/IPCErrors.sol";
import {SubnetIDHelper} from "../lib/SubnetIDHelper.sol";
import {CheckpointHelper} from "../lib/CheckpointHelper.sol";
import {LibVoting} from "../lib/LibVoting.sol";
import {CrossMsgHelper} from "../lib/CrossMsgHelper.sol";
import {LibGateway} from "../lib/LibGateway.sol";
import {StorableMsgHelper} from "../lib/StorableMsgHelper.sol";
import {FvmAddress} from "../structs/FvmAddress.sol";
import {FvmAddressHelper} from "../lib/FvmAddressHelper.sol";
import {FilAddress} from "fevmate/utils/FilAddress.sol";

contract GatewayRouterFacet is GatewayActorModifiers {
    using FilAddress for address;
    using SubnetIDHelper for SubnetID;
    using CheckpointHelper for BottomUpCheckpoint;
    using CrossMsgHelper for CrossMsg;
    using FvmAddressHelper for FvmAddress;
    using StorableMsgHelper for StorableMsg;

    /// @notice commit the ipc parent finality into storage
    function commitParentFinality(
        ParentFinality calldata finality,
        FvmAddress[] calldata validators,
        uint256[] calldata weights
    ) external systemActorOnly {
        LibGateway.commitParentFinality(finality);

        LibGateway.setMembership(validators, weights);
    }

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

        checkpoint.setChildCheck({
            commit: commit,
            children: s.children,
            checks: s.checks,
            currentEpoch: nextCheckEpoch
        });

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
}
