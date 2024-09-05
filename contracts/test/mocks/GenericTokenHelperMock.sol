// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.23;

import {GenericToken} from "../../contracts/structs/Subnet.sol";
import {GenericTokenHelper} from "../../contracts/lib/GenericTokenHelper.sol";

/// @notice Helpers to deal with a supply source.
contract GenericTokenHelperMock {
    function performCall(
        GenericToken memory supplySource,
        address payable target,
        bytes memory data,
        uint256 value
    ) public returns (bool success, bytes memory ret) {
        return GenericTokenHelper.performCall(supplySource, target, data, value);
    }
}
