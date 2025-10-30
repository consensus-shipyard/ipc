// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity ^0.8.23;

import {Asset} from "../structs/Subnet.sol";
import {ValidatorInfo} from "../structs/Subnet.sol";

/// @title ISubnetActor Checkpointing Interface
/// @notice Defines the interface for subnet checkpoint submission and querying
/// @dev Provides a flexible, implementation-agnostic interface for checkpoint handling
interface ISubnetActorCheckpointing {
    /// @notice Returns the block height of the last bottom-up checkpoint
    /// @dev Used to track checkpoint progress and ensure sequential submission
    ///
    /// @return The block height of the most recently accepted bottom-up checkpoint
    ///         Returns 0 if no checkpoint has been submitted yet
    function lastBottomUpCheckpointHeight() external view returns (uint256);

    /// @notice Submits a bottom-up checkpoint with polymorphic data handling
    /// @dev Accepts raw bytes to enable flexible checkpoint formats across different subnet implementations
    ///
    /// This design provides polymorphism at the interface level:
    /// - Subnets can define their own checkpoint data structures (e.g., CometBFT, custom consensus)
    /// - Serialization/deserialization is handled by the implementing subnet
    /// - Enables support for various consensus mechanisms without interface changes
    /// - The implementation is responsible for decoding and validating the raw data
    ///
    /// Example implementations might decode to:
    /// - CometBFT SignedHeader for Tendermint-based subnets
    /// - Custom proof structures for other consensus mechanisms
    /// - Aggregated signatures for BLS-based consensus
    ///
    /// @param rawData The serialized checkpoint data in subnet-specific format
    ///                Must be properly encoded according to the subnet's consensus rules
    function submitBottomUpCheckpoint(bytes calldata rawData) external;
}

/// @title Subnet actor interface
interface ISubnetActor is ISubnetActorCheckpointing {
    function supplySource() external view returns (Asset memory);

    /// @notice Returns the total amount of confirmed collateral across all validators.
    function getTotalCurrentPower() external view returns (uint256);

    /// @notice Obtain the active validator address by its position index in the validator list array.
    function getActiveValidatorAddressByIndex(uint256 index) external view returns (address);

    /// @notice Returns detailed information about a specific validator.
    /// @param validatorAddress The address of the validator to query information for.
    function getValidator(address validatorAddress) external view returns (ValidatorInfo memory validator);
}
