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
}
