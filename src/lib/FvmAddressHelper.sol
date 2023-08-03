// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity 0.8.19;

import {FvmAddress, DelegatedAddress} from "../structs/FvmAddress.sol";

/// @title Helper library for Fil Address
library FvmAddressHelper {
    /// f1: SECP256K1 key address, 20 byte hash of PublicKey.
    uint8 public constant SECP256K1 = 1;
    uint8 public constant PAYLOAD_HASH_LEN = 20;

    /// For delegated FIL address
    uint8 public constant DELEGATED = 4;
    uint64 public constant EAM_ACTOR = 10;

    error NotDelegatedEvmAddress();

    /// @notice Creates a FvmAddress from address type
    function from(address addr) internal pure returns (FvmAddress memory fvmAddress) {
        bytes memory payload = abi.encode(
            DelegatedAddress({namespace: EAM_ACTOR, length: 20, buffer: abi.encodePacked(addr)})
        );

        fvmAddress = FvmAddress({addrType: DELEGATED, payload: payload});
    }

    function extractEvmAddress(FvmAddress memory fvmAddress) internal pure returns (address addr) {
        if (fvmAddress.addrType != DELEGATED) {
            revert NotDelegatedEvmAddress();
        }

        DelegatedAddress memory delegated = abi.decode(fvmAddress.payload, (DelegatedAddress));

        if (delegated.namespace != EAM_ACTOR) {
            revert NotDelegatedEvmAddress();
        }
        if (delegated.length != 20) {
            revert NotDelegatedEvmAddress();
        }
        if (delegated.buffer.length != 20) {
            revert NotDelegatedEvmAddress();
        }

        addr = _bytesToAddress(delegated.buffer);
    }

    function _bytesToAddress(bytes memory bys) private pure returns (address addr) {
        // solhint-disable-next-line
        assembly {
            addr := mload(add(bys, 20))
        }
    }
}
