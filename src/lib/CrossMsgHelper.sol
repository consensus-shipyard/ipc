// SPDX-License-Identifier: UNLICENSED
pragma solidity 0.8.18;

import "../structs/Checkpoint.sol";
import "../constants/Constants.sol";
import "../lib/SubnetIDHelper.sol";
import "openzeppelin-contracts/utils/Address.sol";
import "fevmate/utils/FilAddress.sol";

/// @title Helper library for manipulating StorableMsg struct
/// @author LimeChain team
library CrossMsgHelper {
    using SubnetIDHelper for SubnetID;
    using FilAddress for address;

    bytes32 public constant EMPTY_CROSS_MSG = keccak256(
        abi.encode(
            CrossMsg({
                message: StorableMsg({
                    from: IPCAddress({subnetId: SubnetID(0, new address[](0)), rawAddress: address(0)}),
                    to: IPCAddress({subnetId: SubnetID(0, new address[](0)), rawAddress: address(0)}),
                    value: 0,
                    nonce: 0,
                    method: METHOD_SEND,
                    params: EMPTY_BYTES
                }),
                wrapped: false
            })
        )
    );

    function createReleaseMsg(SubnetID calldata subnet, address signer, uint256 value)
        public
        pure
        returns (CrossMsg memory)
    {
        return CrossMsg({
            message: StorableMsg({
                from: IPCAddress({subnetId: subnet, rawAddress: BURNT_FUNDS_ACTOR}),
                to: IPCAddress({subnetId: subnet.getParentSubnet(), rawAddress: signer}),
                value: value,
                nonce: 0,
                method: METHOD_SEND,
                params: EMPTY_BYTES
            }),
            wrapped: false
        });
    }

    function createFundMsg(SubnetID calldata subnet, address signer, uint256 value)
        public
        pure
        returns (CrossMsg memory)
    {
        return CrossMsg({
            message: StorableMsg({
                from: IPCAddress({subnetId: subnet.getParentSubnet(), rawAddress: signer}),
                to: IPCAddress({subnetId: subnet, rawAddress: signer}),
                value: value,
                nonce: 0,
                method: METHOD_SEND,
                params: EMPTY_BYTES
            }),
            wrapped: false
        });
    }

    function toHash(CrossMsg memory crossMsg) internal pure returns (bytes32) {
        return keccak256(abi.encode(crossMsg));
    }

    function toHash(CrossMsg[] memory crossMsgs) public pure returns (bytes32) {
        return keccak256(abi.encode(crossMsgs));
    }

    function isEmpty(CrossMsg memory crossMsg) internal pure returns (bool) {
        return toHash(crossMsg) == EMPTY_CROSS_MSG;
    }

    function execute(CrossMsg calldata crossMsg) public returns (bytes memory) {
        uint256 value = crossMsg.message.value;
        address recipient = crossMsg.message.to.rawAddress.normalize();

        if (crossMsg.message.method == METHOD_SEND) {
            Address.sendValue(payable(recipient), value);
            return EMPTY_BYTES;
        }

        bytes memory params = crossMsg.message.params;

        if (crossMsg.wrapped) {
            params = abi.encode(crossMsg);
        }

        bytes memory data = abi.encodeWithSelector(crossMsg.message.method, params);

        if (value > 0) {
            return Address.functionCallWithValue(recipient, data, value);
        }

        return Address.functionCall(recipient, data);
    }

    // checks whether the cross messages are sorted in ascending order or not
    function isSorted(CrossMsg[] calldata crossMsgs) external pure returns (bool) {
        uint256 prevNonce = 0;
        uint256 length = crossMsgs.length;
        for (uint256 i = 0; i < length;) {
            uint256 nonce = crossMsgs[i].message.nonce;

            if (prevNonce >= nonce) {
                if (i > 0) {
                    return false;
                }
            }

            prevNonce = nonce;
            unchecked {
                ++i;
            }
        }

        return true;
    }
}
