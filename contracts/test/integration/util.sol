// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity ^0.8.23;

import {SubnetID} from "../../contracts/structs/Subnet.sol";
import {BottomUpBatch} from "../../contracts/structs/BottomUpBatch.sol";
import {CompressedActivityRollup} from "../../contracts/structs/Activity.sol";

import {GatewayDiamond} from "../../contracts/GatewayDiamond.sol";
import {GatewayFacetsHelper} from "../helpers/GatewayFacetsHelper.sol";

import {IpcEnvelope, BottomUpMsgBatch} from "../../contracts/structs/CrossNet.sol";
import {GatewayGetterFacet} from "../../contracts/gateway/GatewayGetterFacet.sol";

import {ActivityHelper} from "../helpers/ActivityHelper.sol";
import {BottomUpBatchHelper} from "../helpers/BottomUpBatchHelper.sol";


/// @notice A bottom-up checkpoint type.
struct BottomUpCheckpoint {
    /// @dev Child subnet ID, for replay protection from other subnets where the exact same validators operate.
    /// Alternatively it can be appended to the hash before signing, similar to how we use the chain ID.
    SubnetID subnetID;
    /// @dev The height of the child subnet at which this checkpoint was cut.
    /// Has to follow the previous checkpoint by checkpoint period.
    uint256 blockHeight;
    /// @dev The hash of the block.
    bytes32 blockHash;
    /// @dev The number of the membership (validator set) which is going to sign the next checkpoint.
    /// This one expected to be signed by the validators from the membership reported in the previous checkpoint.
    /// 0 could mean "no change".
    uint64 nextConfigurationNumber;
    /// @dev Batch of messages to execute.
    BottomUpBatch.Commitment msgs;
    /// @dev The activity rollup from child subnet to parent subnet.
    CompressedActivityRollup activity;
}

library XnetUtil {
    uint64 constant DEFAULT_CHECKPOINT_PERIOD = 10;

    using GatewayFacetsHelper for GatewayDiamond;
    
    function getNextEpoch(uint256 blockNumber, uint256 checkPeriod) internal pure returns (uint256) {
        return ((uint64(blockNumber) / checkPeriod) + 1) * checkPeriod;
    }

    function callCreateBottomUpCheckpointFromChildSubnet(
        SubnetID memory subnet,
        GatewayDiamond gw
    ) internal returns (BottomUpCheckpoint memory checkpoint, IpcEnvelope[] memory messages) {
        uint256 e = getNextEpoch(block.number, DEFAULT_CHECKPOINT_PERIOD);

        GatewayGetterFacet getter = gw.getter();

        BottomUpMsgBatch memory batch = getter.bottomUpMsgBatch(e);
        // require(batch.msgs.length == 1, "batch length incorrect");

        checkpoint = BottomUpCheckpoint({
            subnetID: subnet,
            blockHeight: batch.blockHeight,
            blockHash: keccak256("block1"),
            nextConfigurationNumber: 0,
            msgs: BottomUpBatchHelper.makeCommitment(batch.msgs),
            activity: ActivityHelper.newCompressedActivityRollup(1, 3, bytes32(uint256(0)))
        });

        return (checkpoint, batch.msgs);
    }

    function callCreateBottomUpCheckpointFromChildSubnet(
        SubnetID memory subnet,
        GatewayDiamond gw,
        IpcEnvelope[] memory msgs
    ) internal returns (BottomUpCheckpoint memory checkpoint) {
        uint256 e = getNextEpoch(block.number, DEFAULT_CHECKPOINT_PERIOD);

        checkpoint = BottomUpCheckpoint({
            subnetID: subnet,
            blockHeight: e,
            blockHash: keccak256("block1"),
            nextConfigurationNumber: 0,
            msgs: BottomUpBatchHelper.makeCommitment(msgs),
            activity: ActivityHelper.newCompressedActivityRollup(1, 3, bytes32(uint256(0)))
        });

        return checkpoint;
    }
}
