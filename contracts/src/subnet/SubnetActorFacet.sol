// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity ^0.8.23;

import {InvalidXnetMessage, InvalidXnetMessageReason, WrongSubnet} from "../errors/IPCErrors.sol";
// import {IGateway} from "../interfaces/IGateway.sol";
import {LibDiamond} from "../lib/LibDiamond.sol";
import {LibGenesis} from "../lib/LibGenesis.sol";
import {ReentrancyGuard} from "../lib/LibReentrancyGuard.sol";
import {SubnetActorModifiersV2, SubnetGenesis} from "../lib/LibSubnetActorStorage.sol";
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

library LibSubnetActor {
    // The subnet actor storage
    struct SubnetActorStorage {
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
    }

    function diamondStorage() internal pure returns (SubnetActorStorage storage ds) {
        bytes32 position = keccak256("ipc.subnet.actor.storage");
        assembly {
            ds.slot := position
        }
    }
}

contract SubnetActorFacet is SubnetActorModifiersV2, ReentrancyGuard, Pausable {
    using SupplySourceHelper for SupplySource;
    using FvmAddressHelper for FvmAddress;
    using LibGenesis for SubnetGenesis;
    using SubnetIDHelper for SubnetID;

    event NewTopDownMessage(IpcEnvelope message);

    /// @notice The supplying token
    function supplySource() external view returns(SupplySource memory) {
        return s.supplySource;
    }

    function powerAllocationMode() external view returns(PowerAllocationMode) {
        return s.powerAllocMode;
    }

    function consensus() external view returns(Consensus) {
        return s.consensus;
    }

    /// @notice Handles a specific cross network messages from the gateway.
    function handleXnetCall(IpcEnvelope calldata msg) external onlyGateway {
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
        uint256 transferAmount = s.supplySource.lock({value: amount});

        if (!ISubnet(address(this)).bootstrapped()) {
            /// TODO: convert to to evm is actually a hack. `to` is much more general.
            s.genesis.deposit(to.extractEvmAddress(), transferAmount);
            return;
        }

        SubnetID memory self = s.id;

        IpcEnvelope memory crossMsg = CrossMsgHelper.createFundMsg({
            subnet: self,
            signer: msg.sender,
            to: to,
            value: transferAmount
        });

        commitTopDownMsg(self, crossMsg);
    }

    /// @notice commit topdown messages for their execution in the subnet. Adds the message to the subnet struct for future execution
    /// @param crossMessage - the cross message to be committed
    function commitTopDownMsg(SubnetID memory self, IpcEnvelope memory crossMessage) internal {
        SubnetID memory commonParent = crossMessage.to.subnetId.commonParent(self);
        if (!commonParent.equals(self)) {
            revert WrongSubnet();
        }

        uint64 topDownNonce = s.topDownNonce;
        crossMessage.nonce = topDownNonce;
        s.topDownNonce = topDownNonce + 1;

        emit NewTopDownMessage({message: crossMessage});
    }
}
