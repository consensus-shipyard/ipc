// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity ^0.8.23;

import {SubnetGenesis} from "./LibSubnetActorStorage.sol";
import {EnumerableMap} from "openzeppelin-contracts/utils/structs/EnumerableMap.sol";

/// @title Lib Genesis
/// @notice Handles the subnet genesis states and util functions
library LibGenesis {
    using EnumerableMap for EnumerableMap.AddressToUintMap;

    /// @notice Deposit into the genesis balance of the address
    function deposit(SubnetGenesis storage self, address addr, uint256 amount) internal {
        (bool exists, uint256 existingAmount) = self.balances.tryGet(addr);

        if (exists) {
            self.balances.set(addr, existingAmount + amount);
        } else {
            self.balances.set(addr, amount);
        }

        self.circSupply += amount;
    }
}
