// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity ^0.8.23;

import {GatewayActorModifiers} from "../../lib/LibGatewayActorStorage.sol";
import {IpcEnvelope} from "../../structs/CrossNet.sol";
import {BottomUpBatchRecorded} from "../../structs/BottomUpBatch.sol";
import {LibGateway} from "../../lib/LibGateway.sol";
import {LibQuorum} from "../../lib/LibQuorum.sol";
import {Subnet} from "../../structs/Subnet.sol";
import {QuorumObjKind} from "../../structs/Quorum.sol";
import {Address} from "@openzeppelin/contracts/utils/Address.sol";

import {InvalidBatchSource, NotEnoughBalance, InvalidCheckpointSource, CheckpointAlreadyExists} from "../../errors/IPCErrors.sol";
import {NotRegisteredSubnet, SubnetNotActive, SubnetNotFound, InvalidSubnet, CheckpointNotCreated} from "../../errors/IPCErrors.sol";
import {BatchNotCreated, InvalidBatchEpoch, BatchAlreadyExists, NotEnoughSubnetCircSupply, InvalidCheckpointEpoch} from "../../errors/IPCErrors.sol";

import {CrossMsgHelper} from "../../lib/CrossMsgHelper.sol";
import {IpcEnvelope, SubnetID, IpcMsgKind} from "../../structs/CrossNet.sol";
import {SubnetIDHelper} from "../../lib/SubnetIDHelper.sol";

import {ActivityRollupRecorded, FullActivityRollup} from "../../structs/Activity.sol";
import {StateCommitmentBreakDown} from "../../lib/cometbft/CometbftLightClient.sol";

contract CheckpointingFacet is GatewayActorModifiers {
    using SubnetIDHelper for SubnetID;
    using CrossMsgHelper for IpcEnvelope;

    /// @dev Emitted when a checkpoint is committed to gateway.
    event CheckpointCommitted(address indexed subnet, uint256 subnetHeight);
    event StateCommitmentCreated(uint64 checkpointHeight, StateCommitmentBreakDown breakdown);

    /// @notice submit a verified batch of committed cross-net messages for execution.
    /// @param msgs The batch of messages to be executed.
    function execBottomUpMsgBatch(IpcEnvelope[] calldata msgs) external {
        (bool subnetExists, Subnet storage subnet) = LibGateway.getSubnet(msg.sender);
        if (!subnetExists) {
            revert SubnetNotFound();
        }

        _execBottomUpMsgBatch(msgs, subnet);
    }

    function recordLightClientCommitments(
        StateCommitmentBreakDown calldata commitment,
        IpcEnvelope[] calldata msgs,
        FullActivityRollup calldata activity
    ) external systemActorOnly {
        emit StateCommitmentCreated(uint64(block.number), commitment);
        emit BottomUpBatchRecorded(uint64(block.number), msgs);
        emit ActivityRollupRecorded(uint64(block.number), activity);
    }

    function _execBottomUpMsgBatch(IpcEnvelope[] calldata msgs, Subnet storage subnet) internal {
        uint256 totalValue;
        uint256 crossMsgLength = msgs.length;

        for (uint256 i; i < crossMsgLength; ) {
            totalValue += msgs[i].value;
            unchecked {
                ++i;
            }
        }

        uint256 totalAmount = totalValue;

        if (subnet.circSupply < totalAmount) {
            revert NotEnoughSubnetCircSupply();
        }

        subnet.circSupply -= totalAmount;

        // execute cross-messages
        LibGateway.applyMessages(subnet.id, msgs);
    }
}
