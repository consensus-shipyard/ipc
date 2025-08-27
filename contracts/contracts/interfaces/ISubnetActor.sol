// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity ^0.8.23;

import {Asset} from "../structs/Subnet.sol";
import {ValidatorInfo} from "../structs/Subnet.sol";

/// @title Subnet actor interface
interface ISubnetActor {
    function supplySource() external view returns (Asset memory);

    /// @notice Returns the total amount of confirmed collateral across all validators.
    function getTotalCurrentPower() external view returns (uint256);

    /// @notice Obtain the active validator address by its position index in the validator list array.
    function getActiveValidatorAddressByIndex(uint256 index) external view returns (address);

    /// @notice Returns detailed information about a specific validator.
    /// @param validatorAddress The address of the validator to query information for.
    function getValidator(address validatorAddress) external view returns (ValidatorInfo memory validator);
}
