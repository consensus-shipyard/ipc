// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity ^0.8.23;

import {EnumerableMap} from "openzeppelin-contracts/utils/structs/EnumerableMap.sol";
import {IGenesisComponent} from "../interfaces/IGenesis.sol";

struct SubnetGenesis {
    /// @notice The total circulation supply of the subnet
    uint256 circSupply;
    /// @notice The genesis balances of the address
    EnumerableMap.AddressToUintMap balances;
}

/// @title Lib Subnet Genesis
/// @notice Handles the subnet genesis states and util functions
library LibSubnetGenesis {
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
