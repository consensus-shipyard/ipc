// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity 0.8.19;

import {ADDRESS_CODEHASH} from "../constants/Constants.sol";
import {FilAddress} from "fevmate/utils/FilAddress.sol";

/// @title Helper library for checking account type
/// @author LimeChain team
library AccountHelper {
    function isAccount(address _address) external view returns (bool) {
        uint256 size;

        /* solhint-disable no-inline-assembly */
        assembly {
            size := extcodesize(_address)
        }
        /* solhint-enable no-inline-assembly */

        return size == 0 && ADDRESS_CODEHASH == _address.codehash && ADDRESS_CODEHASH == keccak256(_address.code);
    }

    function isSystemActor(address _address) external pure returns (bool) {
        return _address == FilAddress.SYSTEM_ACTOR;
    }
}
