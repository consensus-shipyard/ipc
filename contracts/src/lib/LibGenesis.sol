// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity ^0.8.23;

import {GenesisValidator, SubnetGenesis, ValidatorInfo, ValidatorSet} from "../structs/Subnet.sol";
import {EnumerableSet} from "openzeppelin-contracts/utils/structs/EnumerableSet.sol";

/// @notice Provides convinient util methods to handle genesis states of the subnet
library LibGenesis {
    using EnumerableSet for EnumerableSet.AddressSet;

    /// @notice Dumps the validator's genesis information
    function getValidatorInfo(SubnetGenesis storage self, address validator) internal view returns(GenesisValidator memory info){
        info = self.validatorInfo[validator];   
    }

    function addValidator(SubnetGenesis storage self, address validator) internal {
        self.validators.add(validator);   
    }

    function removeValidator(SubnetGenesis storage self, address validator) internal {
        self.validators.remove(validator);   
    }

    /// @notice Handles the genesis state when the subnet is bootstrapped. From this point onwards,
    ///         no genesis state of the subnet can be changed.
    /// @param validatorInfo The validator staking information from LibStaking
    function bootstrap(SubnetGenesis storage self, ValidatorSet storage validatorInfo) internal {
        finalizeValidatorInfo(self, validatorInfo);
    }

    // ============ Interal functions ==============

    /// @notice Finalizes the genesis validator information as the subnet is bootstrapped. After 
    ///         this point, the genesis validator info can no longer be changed.
    /// @param validatorInfo The validator staking information from LibStaking
    function finalizeValidatorInfo(SubnetGenesis storage self, ValidatorSet storage validatorInfo) internal {
        address[] memory validators = self.validators.values();
        
        for (uint256 i = 0; i < validators.length; ) {
            address addr = validators[i];

            ValidatorInfo memory info = validatorInfo.validators[addr];
            GenesisValidator memory genesis = GenesisValidator({
                collateral: info.totalCollateral,
                federatedPower: info.federatedPower,
                addr: addr,
                metadata: info.metadata
            });

            self.validatorInfo[addr] = genesis;

            unchecked {
                i++;
            }
        }
    }
}