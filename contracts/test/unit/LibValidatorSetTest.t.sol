// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity ^0.8.23;

import {Test} from "forge-std/Test.sol";
import {MaxPQ, LibMaxPQ} from "../../contracts/lib/priority/LibMaxPQ.sol";
import {LibValidatorSet} from "../../contracts/lib/LibPower.sol";
import {ValidatorSet} from "../../contracts/structs/Subnet.sol";

library LibValidatorSetTest {
    using LibValidatorSet for ValidatorSet;

    function confirmDeposit(ValidatorSet storage self, address validator, uint256 amount) internal {
        uint256 oldCollateral = self.getCurrentPower(validator);
        self.confirmPower(validator, oldCollateral + amount);
    }
}