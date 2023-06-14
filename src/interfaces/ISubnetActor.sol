// SPDX-License-Identifier: UNLICENSED
pragma solidity 0.8.18;

import "../structs/Checkpoint.sol";

/// @title Subnet Actor interface
/// @author LimeChain team
interface ISubnetActor {
    /// Called by peers looking to join a subnet.
    ///
    /// It implements the basic logic to onboard new peers to the subnet.
    function join(string calldata networkAddr) external payable;

    /// Called by peers looking to leave a subnet.
    function leave() external;

    /// Unregister the subnet from the hierarchy, making it no longer discoverable.
    function kill() external;

    /// SubmitCheckpoint accepts signed checkpoint votes for miners.
    ///
    /// This functions verifies that the checkpoint is valid before
    /// propagating it for commitment to the IPC gateway. It expects at least
    /// votes from 2/3 of miners with collateral.
    function submitCheckpoint(BottomUpCheckpoint calldata checkpoint) external;

    /// Tracks the accumulated rewards for each validator.
    function reward(uint256 amount) external;
}
