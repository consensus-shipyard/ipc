// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity 0.8.19;

import {BottomUpCheckpoint, CrossMsg, ParentFinality} from "../structs/Checkpoint.sol";
import {SubnetID} from "../structs/Subnet.sol";
import {FvmAddress} from "../structs/FvmAddress.sol";

/// @title Gateway interface
/// @author LimeChain team
interface IGateway {
    /// @notice Register is called by subnet actors to put the required collateral
    /// and register the subnet to the hierarchy.
    function register(uint256 genesisCircSupply) external payable;

    /// @notice AddStake adds stake to the collateral of a subnet.
    function addStake() external payable;

    /// @notice Release stake recovers some collateral of the subnet
    function releaseStake(uint256 amount) external;

    /// @notice Release reward for relayer
    function releaseRewardForRelayer(uint256 amount) external;

    /// @notice Kill propagates the kill signal from a subnet actor to unregister it from th
    /// hierarchy.
    function kill() external;

    /// @notice commitBottomUpCheckpoint propagates the commitment of a checkpoint from a child subnet and
    /// processes the cross-messages directed to the subnets.
    function commitBottomUpCheckpoint(
        BottomUpCheckpoint calldata bottomUpCheckpoint,
        CrossMsg[] calldata messages
    ) external;

    /// Fund injects new funds from an account of the parent chain to a subnet.
    ///
    /// This functions receives a transaction with the FILs that want to be injected in the subnet.
    /// - Funds injected are frozen.
    /// - A new fund cross-message is created and stored to propagate it to the subnet. It will be
    /// picked up by miners to include it in the next possible block.
    /// - The cross-message nonce is updated
    function fund(SubnetID calldata subnetId, FvmAddress calldata to) external payable;

    /// @notice Release creates a new check message to release funds in parent chain
    ///
    /// This function burns the funds that will be released in the current subnet
    /// and propagates a new checkpoint message to the parent chain to signal
    /// the amount of funds that can be released for a specific address.
    function release(FvmAddress calldata to) external payable;

    /// @notice SendUserXnetMessage sends an arbitrary cross-message to other subnet in the hierarchy.
    ///
    /// If the message includes any funds they need to be burnt (like in Release)
    /// before being propagated to the corresponding subnet.
    /// The circulating supply in each subnet needs to be updated as the message passes through them.
    ///
    /// Params expect a raw message without any subnet context (the IPC address is
    /// included in the message by the actor). Only actors are allowed to send arbitrary
    /// cross-messages as a side-effect of their execution. For plain token exchanges
    /// fund and release have to be used.
    function sendUserXnetMessage(CrossMsg memory crossMsg) external payable;

    /// @notice Propagates the stored postbox item for the given cid
    function propagate(bytes32 msgCid) external payable;

    /// @notice commit the ipc parent finality into storage
    function commitParentFinality(ParentFinality calldata finality) external;

    /// @notice creates a new bottom-up checkpoint
    function createBottomUpCheckpoint(
        BottomUpCheckpoint calldata checkpoint,
        bytes32 membershipRootHash,
        uint256 membershipWeight
    ) external;
}
