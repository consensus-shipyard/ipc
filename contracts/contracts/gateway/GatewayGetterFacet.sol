// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity ^0.8.23;

import {BottomUpMsgBatch, IpcEnvelope, ParentFinality} from "../structs/CrossNet.sol";
import {SubnetID, Subnet} from "../structs/Subnet.sol";
import {Membership} from "../structs/Subnet.sol";
import {LibGateway} from "../lib/LibGateway.sol";
import {LibPower} from "../lib/LibPower.sol";
import {GatewayActorStorage} from "../lib/LibGatewayActorStorage.sol";
import {SubnetIDHelper} from "../lib/SubnetIDHelper.sol";
import {EnumerableSet} from "@openzeppelin/contracts/utils/structs/EnumerableSet.sol";

contract GatewayGetterFacet {
    // slither-disable-next-line uninitialized-state
    GatewayActorStorage internal s;

    using SubnetIDHelper for SubnetID;
    using EnumerableSet for EnumerableSet.UintSet;
    using EnumerableSet for EnumerableSet.AddressSet;
    using EnumerableSet for EnumerableSet.Bytes32Set;

    /// @notice Returns the next and start configuration numbers in the validator changes.
    /// The configuration numbers are from changes made in the parent.
    function getValidatorConfigurationNumbers() external view returns (uint64, uint64) {
        return LibPower.getConfigurationNumbers();
    }

    /// @notice Returns code commit SHA where this contract is from.
    function getCommitSha() external view returns (bytes32) {
        return s.commitSha;
    }

    /// @notice Returns the current nonce for bottom-up message processing.
    function bottomUpNonce() external view returns (uint64) {
        return s.bottomUpNonce;
    }

    /// @notice Returns the total number of the registered subnets.
    function totalSubnets() external view returns (uint64) {
        return s.totalSubnets;
    }

    /// @notice Returns the period for bottom-up checkpointing.
    function bottomUpCheckPeriod() external view returns (uint256) {
        return s.bottomUpCheckPeriod;
    }

    /// @notice Returns the subnet identifier of the network.
    function getNetworkName() external view returns (SubnetID memory) {
        return s.networkName;
    }

    /// @notice Returns a specific bottom-up message batch based on an index.
    /// @param e The epoch number of the batch.
    function bottomUpMsgBatch(uint256 e) external view returns (BottomUpMsgBatch memory) {
        return s.bottomUpMsgBatches[e];
    }

    /// @notice Returns the parent chain finality information for a given block number.
    /// @param blockNumber The block number for which to retrieve parent-finality information.
    function getParentFinality(uint256 blockNumber) external view returns (ParentFinality memory) {
        return LibGateway.getParentFinality(blockNumber);
    }

    /// @notice Gets the most recent parent-finality information from the parent.
    function getLatestParentFinality() external view returns (ParentFinality memory) {
        return LibGateway.getLatestParentFinality();
    }

    /// @notice Returns the subnet with the given id.
    /// @param subnetId the id of the subnet.
    /// @return found whether the subnet exists.
    /// @return subnet -  the subnet struct.
    function getSubnet(SubnetID calldata subnetId) external view returns (bool, Subnet memory) {
        // slither-disable-next-line unused-return
        return LibGateway.getSubnet(subnetId);
    }

    /// @notice Returns information about a specific subnet using its hash identifier.
    /// @param h The hash identifier of the subnet to be queried.
    /// @return subnet The subnet information corresponding to the given hash.
    function subnets(bytes32 h) external view returns (Subnet memory subnet) {
        return s.subnets[h];
    }

    /// @notice Returns the length of the top-down message queue for a specified subnet.
    /// @param subnetId The identifier of the subnet for which the message queue length is queried.
    /// @return The current length of the top-down message queue, indicated by the subnet's top-down nonce.
    function getSubnetTopDownMsgsLength(SubnetID memory subnetId) external view returns (uint256) {
        // slither-disable-next-line unused-return
        (, Subnet storage subnet) = LibGateway.getSubnet(subnetId);
        // With every new message, the nonce is added by one, the current nonce should be equal to the top down message length.
        return subnet.topDownNonce;
    }

    /// @notice Returns the current applied top-down nonce for a specified subnet, indicating whether it's registered.
    /// @param subnetId The identifier of the subnet for which the top-down nonce is queried.
    /// @return A tuple containing a boolean indicating if the subnet is registered and the current top-down nonce.
    function getTopDownNonce(SubnetID calldata subnetId) external view returns (bool, uint64) {
        (bool registered, Subnet storage subnet) = LibGateway.getSubnet(subnetId);
        if (!registered) {
            return (false, 0);
        }
        return (true, subnet.topDownNonce);
    }

    /// @notice Returns the current applied bottom-up nonce for a specified subnet, indicating whether it's registered.
    /// @param subnetId The identifier of the subnet for which the bottom-up nonce is queried.
    /// @return A tuple containing a boolean indicating if the subnet is registered and the current applied bottom-up nonce.
    function getAppliedBottomUpNonce(SubnetID calldata subnetId) external view returns (bool, uint64) {
        (bool registered, Subnet storage subnet) = LibGateway.getSubnet(subnetId);
        if (!registered) {
            return (false, 0);
        }
        return (true, subnet.appliedBottomUpNonce);
    }

    /// @notice Returns the current applied top-down nonce of the gateway.
    function appliedTopDownNonce() external view returns (uint64) {
        return s.appliedTopDownNonce;
    }

    /// @notice Returns the storable message and its wrapped status from the postbox by a given identifier.
    /// @param id The unique identifier of the message in the postbox.
    function postbox(bytes32 id) external view returns (IpcEnvelope memory storableMsg) {
        return (s.postbox[id]);
    }

    function postboxMsgs() external view returns (bytes32[] memory) {
        return (s.postboxKeys.values());
    }

    /// @notice Returns the majority percentage required for certain consensus or decision-making processes.
    function majorityPercentage() external view returns (uint64) {
        return s.majorityPercentage;
    }

    /// @notice Returns the list of registered subnets.
    /// @return The list of the registered subnets.
    function listSubnets() external view returns (Subnet[] memory) {
        uint256 size = s.subnetKeys.length();
        Subnet[] memory out = new Subnet[](size);
        for (uint256 i; i < size; ) {
            bytes32 key = s.subnetKeys.at(i);
            out[i] = s.subnets[key];
            unchecked {
                ++i;
            }
        }
        return out;
    }

    /// @notice Returns the subnet keys.
    function getSubnetKeys() external view returns (bytes32[] memory) {
        return s.subnetKeys.values();
    }

    /// @notice Returns the last membership received from the parent.
    function getLastMembership() external view returns (Membership memory) {
        return s.lastMembership;
    }

    /// @notice Returns the last configuration number received from the parent.
    function getLastConfigurationNumber() external view returns (uint64) {
        return s.lastMembership.configurationNumber;
    }

    /// @notice Returns the current membership.
    function getCurrentMembership() external view returns (Membership memory) {
        return s.currentMembership;
    }

    /// @notice Returns the current configuration number.
    function getCurrentConfigurationNumber() external view returns (uint64) {
        return s.currentMembership.configurationNumber;
    }
}
