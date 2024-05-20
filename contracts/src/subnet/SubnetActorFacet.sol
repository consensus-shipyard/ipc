// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity ^0.8.23;

/**
 * Subnet actor facet that tracks the general state of the subnet. It's required for any subnet actor.
 */

import {InvalidXnetMessage, InvalidXnetMessageReason, NotGateway, WrongSubnet} from "../errors/IPCErrors.sol";
// import {IGateway} from "../interfaces/IGateway.sol";
import {LibDiamond} from "../lib/LibDiamond.sol";
import {LibSubnetGenesis, SubnetGenesis} from "../lib/LibGenesis.sol";
import {ReentrancyGuard} from "../lib/LibReentrancyGuard.sol";
import {Pausable} from "../lib/LibPausable.sol";
import {SupplySource, SubnetID, PowerAllocationMode} from "../structs/Subnet.sol";
import {SupplySourceHelper} from "../lib/SupplySourceHelper.sol";
import {FvmAddressHelper} from "../lib/FvmAddressHelper.sol";
import {FvmAddress} from "../structs/FvmAddress.sol";
import {SubnetIDHelper} from "../lib/SubnetIDHelper.sol";
import {IpcEnvelope} from "../structs/CrossNet.sol";
import {Consensus} from "../enums/ConsensusType.sol";
import {CrossMsgHelper} from "../lib/CrossMsgHelper.sol";
import {ISubnet} from "../interfaces/ISubnet.sol";

/// @notice The getters for querying subnet state
library LibSubnetActorQuery {
    function ipcGatewayAddr() internal view returns(address) {
        return LibSubnetActorStorage.diamondStorage().ipcGatewayAddr;
    }

    function id() internal view returns(SubnetID memory) {
        return LibSubnetActorStorage.diamondStorage().id;
    }

    function supplySource() internal view returns(SupplySource memory) {
        return LibSubnetActorStorage.diamondStorage().supplySource;
    }

    function powerAllocationMode() external view returns(PowerAllocationMode) {
        return LibSubnetActorStorage.diamondStorage().powerAllocMode;
    }

    function consensus() external view returns(Consensus) {
        return LibSubnetActorStorage.diamondStorage().consensus;
    }
}

contract SubnetActorFacet is ReentrancyGuard, Pausable {
    using FvmAddressHelper for FvmAddress;
    using LibSubnetGenesis for SubnetGenesis;
    using SubnetIDHelper for SubnetID;

    /// @notice The supplying token
    function supplySource() external view returns(SupplySource memory) {
        return LibSubnetActorQuery.supplySource();
    }

    function powerAllocationMode() external view returns(PowerAllocationMode) {
        return LibSubnetActorQuery.powerAllocationMode();
    }

    function consensus() external view returns(Consensus) {
        return LibSubnetActorQuery.consensus();
    }

    /// @notice Handles a specific cross network messages from the gateway.
    function handleXnetCall(IpcEnvelope calldata envelope) external {
        LibSubnetActor.onlyGateway();
        revert("todo");
    }

    /// @notice credits the received value to the specified address in the specified child subnet.
    ///
    /// @dev There may be an associated fee that gets distributed to validators in the subnet. Currently this fee is zero,
    ///      i.e. funding a subnet is free.
    ///
    /// @param to: the address to which to credit funds in the subnet.
    /// @param amount: the amount to send
    function fund(FvmAddress calldata to, uint256 amount) external payable {
        if (amount == 0) {
            // prevent spamming if there's no value to fund.
            revert InvalidXnetMessage(InvalidXnetMessageReason.Value);
        }

        // Locks a specified amount into custody, adjusting for tokens with transfer fees. This operation
        // accommodates inflationary tokens, potentially reflecting a higher effective locked amount.
        // Operation reverts if the effective transferred amount is zero.
        uint256 transferAmount = LibSubnetActor.lockFund(amount);

        SubnetActorStorage storage s = LibSubnetActorStorage.diamondStorage();

        if (!ISubnet(address(this)).bootstrapped()) {
            /// TODO: convert to to evm is actually a hack. `to` is much more general.
            s.genesis.deposit(to.extractEvmAddress(), transferAmount);
            return;
        }

        LibSubnetActor.emitTopDownMsg(
            CrossMsgHelper.createFundMsg({
                subnet: s.id,
                signer: msg.sender,
                to: to,
                value: transferAmount
            })
        );
    }
}

/// @notice Metadata handling and fund management for the subnet actor 
library LibSubnetActor {
    using SupplySourceHelper for SupplySource;
    using SubnetIDHelper for SubnetID;

    event NewTopDownMessage(IpcEnvelope message);

    function onlyGateway() internal view {
        SubnetActorStorage storage s = LibSubnetActorStorage.diamondStorage();
        if (msg.sender != s.ipcGatewayAddr) {
            revert NotGateway();
        }
    }

    /// @notice Lock certain amount of fund in the subnet
    function lockFund(uint256 amount) internal returns (uint256) {
        SubnetActorStorage storage s = LibSubnetActorStorage.diamondStorage();
        return s.supplySource.lock({value: amount});
    }

    /// @notice Obtains the applied bottom up nonce and increment by one. Returns the old value
    function getThenIncrAppliedBottomUpNonce() internal returns (uint64 appliedBottomUpNonce) {
        SubnetActorStorage storage s = LibSubnetActorStorage.diamondStorage();
        appliedBottomUpNonce = s.appliedBottomUpNonce;
        s.appliedBottomUpNonce = appliedBottomUpNonce + 1;
    }

    /// @notice Obtains the topdown nonce and increment by one. Returns the old value.
    function getThenIncrTopdownNonce() internal returns (uint64 topDownNonce) {
        SubnetActorStorage storage s = LibSubnetActorStorage.diamondStorage();
        topDownNonce = s.topDownNonce;
        s.topDownNonce = topDownNonce + 1;
    }

    /// @notice commit topdown messages for their execution in the subnet. Adds the message to the subnet struct for future execution
    /// @param crossMessage - the cross message to be committed
    function emitTopDownMsg(IpcEnvelope memory crossMessage) internal {
        SubnetID memory self = LibSubnetActorStorage.diamondStorage().id;

        SubnetID memory commonParent = crossMessage.to.subnetId.commonParent(self);
        if (!commonParent.equals(self)) {
            revert WrongSubnet();
        }

        crossMessage.nonce = getThenIncrTopdownNonce();

        emit NewTopDownMessage({message: crossMessage});
    }
}

/// ============== Internal ==============

// The subnet actor storage
struct SubnetActorStorage {
    /// @notice Address of the IPC gateway for the subnet
    address ipcGatewayAddr;
    /// @notice The topdown nonce
    uint64 topDownNonce;
    /// @notice The genesis state tracked by this contract
    SubnetGenesis genesis;
    /// immutable params
    Consensus consensus;
    /// @notice ID of the self
    SubnetID id;
    /// @notice The power allocation mode
    PowerAllocationMode powerAllocMode;
    /// @notice subnet supply strategy.
    SupplySource supplySource;
    /// @notice ID of the parent subnet
    SubnetID parentId;
    /// @notice The applied bottom up nonce, i.e. number of bottom up messages executed
    uint64 appliedBottomUpNonce;
}

library LibSubnetActorStorage {
    function diamondStorage() internal pure returns (SubnetActorStorage storage ds) {
        bytes32 position = keccak256("ipc.subnet.actor.storage");
        assembly {
            ds.slot := position
        }
    }
}