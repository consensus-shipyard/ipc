// SPDX-License-Identifier: UNLICENSED
pragma solidity 0.8.18;

import "../constants/Constants.sol";
import "fevmate/utils/FilAddress.sol";

/// @title Helper library for checking account type
/// @author LimeChain team
library AccountHelper {
    function isAccount(address _address) external view returns (bool) {
        uint256 size;

        assembly {
            size := extcodesize(_address)
        }

        return size == 0 && ADDRESS_CODEHASH == _address.codehash && ADDRESS_CODEHASH == keccak256(_address.code);
    }

    function isSystemActor(address _address) external pure returns (bool) {
        return _address == FilAddress.SYSTEM_ACTOR;
    }
}
