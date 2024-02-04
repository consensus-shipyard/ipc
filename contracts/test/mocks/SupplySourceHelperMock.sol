// SPDX-License-Identifier: UNLICENSED
pragma solidity 0.8.19;

import {SupplySource} from "../../src/structs/Subnet.sol";
import {SupplySourceHelper} from "../../src/lib/SupplySourceHelper.sol";

/// @notice Helpers to deal with a supply source.
contract SupplySourceHelperMock {
    function functionCallWithERC20Value(
        SupplySource memory supplySource,
        address target,
        bytes memory data,
        uint256 value
    ) public returns (bool success, bytes memory ret) {
        return SupplySourceHelper.functionCallWithERC20Value(supplySource, target, data, value);
    }
}
