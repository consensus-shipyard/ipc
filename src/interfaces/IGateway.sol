// SPDX-License-Identifier: UNLICENSED
pragma solidity 0.8.18;

import "../structs/Checkpoint.sol";
import "../structs/Subnet.sol";

/// @title Gateway interface
/// @author LimeChain team
interface IGateway {
    /// Register is called by subnet actors to put the required collateral
    /// and register the subnet to the hierarchy.
    function register() external payable;

    /// AddStake adds stake to the collateral of a subnet.
    function addStake() external payable;

    /// Release stake recovers some collateral of the subnet
    function releaseStake(uint256 amount) external;

    // Release rewards to the subnet actor
    function releaseRewards(uint256 amount) external;

    // Kill propagates the kill signal from a subnet actor to unregister it from th
    /// hierarchy.
    function kill() external;

    /// CommitChildCheck propagates the commitment of a checkpoint from a child subnet,
    /// process the cross-messages directed to the subnet.
    function commitChildCheck(BottomUpCheckpoint calldata bottomupCheckpoint) external;

    /// Fund injects new funds from an account of the parent chain to a subnet.
    ///
    /// This functions receives a transaction with the FILs that want to be injected in the subnet.
    /// - Funds injected are frozen.
    /// - A new fund cross-message is created and stored to propagate it to the subnet. It will be
    /// picked up by miners to include it in the next possible block.
    /// - The cross-message nonce is updated
    function fund(SubnetID memory subnetId) external payable;

    /// Release creates a new check message to release funds in parent chain
    ///
    /// This function burns the funds that will be released in the current subnet
    /// and propagates a new checkpoint message to the parent chain to signal
    /// the amount of funds that can be released for a specific address.
    function release() external payable;

    /// SendCross sends an arbitrary cross-message to other subnet in the hierarchy.
    ///
    /// If the message includes any funds they need to be burnt (like in Release)
    /// before being propagated to the corresponding subnet.
    /// The circulating supply in each subnet needs to be updated as the message passes through them.
    ///
    /// Params expect a raw message without any subnet context (the IPC address is
    /// included in the message by the actor). Only actors are allowed to send arbitrary
    /// cross-messages as a side-effect of their execution. For plain token exchanges
    /// fund and release have to be used.
    function sendCross(SubnetID memory destination, CrossMsg memory crossMsg) external payable;

    /// Whitelist a series of addresses as propagator of a cross net message.
    /// This is basically adding this list of addresses to the `PostBoxItem::owners`.
    /// Only existing owners can perform this operation.
    function whitelistPropagator(bytes32 msgCid, address[] calldata owners) external;

    /// Propagates the stored postbox item for the given cid
    function propagate(bytes32 msgCid) external payable;

    function submitTopDownCheckpoint(TopDownCheckpoint calldata topdownCheckpoint) external;

    function setMembership(address[] memory validatorsToSet, uint256[] memory weights) external;
}
