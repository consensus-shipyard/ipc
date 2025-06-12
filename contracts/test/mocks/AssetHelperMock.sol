// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.23;

import {Asset} from "../../contracts/structs/Subnet.sol";
import {AssetHelper} from "../../contracts/lib/AssetHelper.sol";

/// @notice Helpers to deal with a supply source.
contract AssetHelperMock {
    function performCall(
        Asset memory supplySource,
        address payable target,
        bytes memory data,
        uint256 value
    ) public returns (bool success, bytes memory ret) {
        return AssetHelper.performCall(supplySource, target, data, value);
    }
}
