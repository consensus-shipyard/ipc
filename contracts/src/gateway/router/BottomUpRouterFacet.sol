// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity 0.8.19;

import {IRelayerRewardDistributor} from "../../interfaces/ISubnetActor.sol";
import {GatewayActorModifiers} from "../../lib/LibGatewayActorStorage.sol";
import {BottomUpMsgBatch} from "../../structs/CrossNet.sol";
import {LibGateway} from "../../lib/LibGateway.sol";
import {BatchNotCreated, BatchAlreadyExists, InvalidBatchEpoch, NotEnoughSubnetCircSupply, SubnetNotActive, SubnetNotFound, InvalidBatchSource, MaxMsgsPerBatchExceeded, BatchWithNoMessages, InvalidCrossMsgDstSubnet, NotRegisteredSubnet, InvalidCrossMsgNonce} from "../../errors/IPCErrors.sol";
import {Subnet} from "../../structs/Subnet.sol";
import {LibQuorum} from "../../lib/LibQuorum.sol";
import {QuorumObjKind} from "../../structs/Quorum.sol";
import {Address} from "openzeppelin-contracts/utils/Address.sol";
import {IPCMsgType} from "../../enums/IPCMsgType.sol";

import {CrossMsg, SubnetID} from "../../structs/CrossNet.sol";
import {CrossMsgHelper} from "../../lib/CrossMsgHelper.sol";

import {SupplySourceHelper} from "../../lib/SupplySourceHelper.sol";
import {SupplySource} from "../../structs/Subnet.sol";
import {SubnetActorGetterFacet} from "../../subnet/SubnetActorGetterFacet.sol";

import {SubnetIDHelper} from "../../lib/SubnetIDHelper.sol";
import {StorableMsgHelper} from "../../lib/StorableMsgHelper.sol";
import {StorableMsg} from "../../structs/CrossNet.sol";

contract BottomUpRouterFacet is GatewayActorModifiers {
    using SubnetIDHelper for SubnetID;
    using CrossMsgHelper for CrossMsg;
    using SupplySourceHelper for SupplySource;
    using StorableMsgHelper for StorableMsg;

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
}
