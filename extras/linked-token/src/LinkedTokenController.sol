// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity 0.8.23;

import {
    SafeERC20
} from "openzeppelin-contracts/token/ERC20/utils/SafeERC20.sol";
import {IERC20} from "openzeppelin-contracts/token/ERC20/IERC20.sol";
import {LinkedToken} from "./LinkedToken.sol";
import {SubnetID} from "@ipc/src/structs/Subnet.sol";

contract LinkedTokenController is LinkedToken {
    using SafeERC20 for IERC20;

    constructor(
        address gateway,
        address underlyingToken,
        SubnetID memory linkedSubnet
    ) LinkedToken(gateway, underlyingToken, linkedSubnet) {}

    function _captureTokens(address holder, uint256 amount) internal override {
        _underlying.safeTransferFrom({
            from: msg.sender,
            to: address(this),
            value: amount
        });
    }

    function _releaseTokens(address beneficiary, uint256 amount)
        internal
        override
    {
        _underlying.safeTransfer(beneficiary, amount);
    }
}
