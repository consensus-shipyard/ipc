// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.7;

import "../structs/Checkpoint.sol";

/// @title Gateway interface
/// @author LimeChain team
interface IGateway {
    /// Register is called by subnet actors to put the required collateral
    /// and register the subnet to the hierarchy.
    function register() external payable;

    /// AddStake adds stake to the collateral of a subnet.
    function addStake() external payable;

    /// Release stake recovers some collateral of the subnet
    function releaseStake(uint amount) external;

    // Kill propagates the kill signal from a subnet actor to unregister it from th
    /// hierarchy.
    function kill() external;

    /// CommitChildCheck propagates the commitment of a checkpoint from a child subnet,
    /// process the cross-messages directed to the subnet.
    function commitChildCheck(
        Checkpoint calldata checkpoint
    ) external returns (uint);
    
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
}
