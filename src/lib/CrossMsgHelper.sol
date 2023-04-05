// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.7;

import "../structs/Checkpoint.sol";
import "../constants/Constants.sol";
import "../lib/SubnetIDHelper.sol";

/// @title Helper library for manipulating StorableMsg struct
/// @author LimeChain team
library CrossMsgHelper {
    using SubnetIDHelper for SubnetID;

    function createReleaseMsg(
        SubnetID calldata subnet,
        address signer,
        uint256 value,
        uint64 nonce
    ) public pure returns (CrossMsg memory) {
        return
            CrossMsg({
                message: StorableMsg({
                    from: IPCAddress({
                        subnetId: subnet,
                        rawAddress: BURNT_FUNDS_ACTOR
                    }),
                    to: IPCAddress({
                        subnetId: subnet.getParentSubnet(),
                        rawAddress: signer
                    }),
                    value: value,
                    nonce: nonce,
                    method: 0,
                    params: EMPTY_BYTES
                }),
                wrapped: false
            });
    }

    function createFundMsg(
        SubnetID calldata subnet,
        address signer,
        uint256 value
    ) public pure returns (CrossMsg memory) {
        return
            CrossMsg({
                message: StorableMsg({
                    from: IPCAddress({
                        subnetId: subnet.getParentSubnet(),
                        rawAddress: signer
                    }),
                    to: IPCAddress({subnetId: subnet, rawAddress: signer}),
                    value: value,
                    nonce: 0,
                    method: 0,
                    params: EMPTY_BYTES
                }),
                wrapped: false
            });
    }

    function toHash(CrossMsg memory crossMsg) internal pure returns (bytes32) {
        return keccak256(abi.encode(crossMsg));
    }

    function toHash(
        CrossMsg[] memory crossMsgs
    ) internal pure returns (bytes32) {
        return keccak256(abi.encode(crossMsgs));
    }
}
