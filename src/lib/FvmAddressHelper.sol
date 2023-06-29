// SPDX-License-Identifier: UNLICENSED
pragma solidity 0.8.19;

import {FvmAddress} from "../structs/FvmAddress.sol";

/// @title Helper library for Fil Address
library FvmAddressHelper {
    /// f1: SECP256K1 key address, 20 byte hash of PublicKey.
    uint8 public constant SECP256K1 = 1;
    uint8 public constant PAYLOAD_HASH_LEN = 20;

    /// @notice Checks if two fil addresses are the same
    function isEqual(FvmAddress calldata f1, FvmAddress calldata f2) internal pure returns (bool) {
        if (f1.addrType != f2.addrType) {
            return false;
        }
        return f1.payload.length == f2.payload.length && keccak256(f1.payload) == keccak256(f2.payload);
    }

    /// @notice Checks if the fil addresses is valid
    function isValid(FvmAddress calldata filAddress) internal pure returns (bool) {
        require(filAddress.addrType == SECP256K1, "Addr not supported");
        return _isValidF1Address(filAddress.payload);
    }

    function _isValidF1Address(bytes calldata payload) private pure returns (bool) {
        return payload.length == PAYLOAD_HASH_LEN;
    }
}
