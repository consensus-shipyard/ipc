// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.23;

import {SupplySource} from "../../contracts/structs/Subnet.sol";
import {SupplySourceHelper} from "../../contracts/lib/SupplySourceHelper.sol";

/// @notice Helpers to deal with a supply source.
contract SupplySourceHelperMock {
    function performCall(
        SupplySource memory supplySource,
        address payable target,
        bytes memory data,
        uint256 value
    ) public returns (bool success, bytes memory ret) {
        return SupplySourceHelper.performCall(supplySource, target, data, value);
    }
}
