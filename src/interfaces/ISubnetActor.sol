// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity 0.8.19;

import {BottomUpCheckpoint} from "../structs/Checkpoint.sol";
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

    /// Unregister the subnet from the hierarchy, making it no longer discoverable.
    function kill() external;

    /// Valdiator claims their released collateral
    function claim() external;

    /// SubmitCheckpoint accepts signed checkpoint votes for validators.
    function submitCheckpoint(
        BottomUpCheckpoint calldata checkpoint,
        bytes32 membershipRootHash,
        uint256 membershipWeight
    ) external;
}
