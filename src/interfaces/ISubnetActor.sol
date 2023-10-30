// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity 0.8.19;

import {BottomUpCheckpoint, CrossMsg} from "../structs/Checkpoint.sol";
import {FvmAddress} from "../structs/FvmAddress.sol";

/// @title Subnet Actor interface
/// @author LimeChain team
interface ISubnetActor {
    /// Called by peers looking to join a subnet.
    ///
    /// It implements the basic logic to onboard new peers to the subnet.
    function join(bytes calldata metadata) external payable;

    /// Called by peers looking to leave a subnet.
    function leave() external;

    /// Method that allows a validator to increase their stake
    function stake() external payable;

    /// Method that allows a validator to unstake their collateral from a subnet
    function unstake(uint256 amount) external;

    /// Unregister the subnet from the hierarchy, making it no longer discoverable.
    function kill() external;

    /// Validator claims released collateral
    function claim() external;

    /// Relayer claims a reward
    function claimRewardForRelayer() external;

    /// Executes the checkpoint if it is valid.
    /// It triggers the commitment of the checkpoint, the execution of related cross-net messages,
    /// and any other side-effects that need to be triggered by the checkpoint such as relayer reward book keeping.
    function submitCheckpoint(
        BottomUpCheckpoint calldata checkpoint,
        CrossMsg[] calldata messages,
        address[] calldata signatories,
        bytes[] calldata signatures
    ) external;

    /// reward the relayers for processing checkpoint at height `height`.
    /// The reword includes the fixed reward for a relayer defined in the contract and `amount` of fees from the cross-messages.
    function distributeRewardToRelayers(uint64 height, uint256 amount) external payable;
}
