// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity 0.8.23;

import {
    SafeERC20
} from "openzeppelin-contracts/token/ERC20/utils/SafeERC20.sol";
import {IERC20} from "openzeppelin-contracts/token/ERC20/IERC20.sol";
import {LinkedTokenFacet} from "./LinkedTokenFacet.sol";
import {SubnetID} from "@ipc/src/structs/Subnet.sol";

import {LibLinkedToken} from "./lib/LibLinkedToken.sol";

contract LinkedTokenControllerFacet is LinkedTokenFacet {
    using SafeERC20 for IERC20;

    function _captureTokens(address holder, uint256 amount) internal override {
        IERC20 underlying = LibLinkedToken.getUnderlyingToken();
        underlying.safeTransferFrom({
            from: msg.sender,
            to: address(this),
            value: amount
        });
    }

    function _releaseTokens(address beneficiary, uint256 amount)
        internal
        override
    {
        IERC20 underlying = LibLinkedToken.getUnderlyingToken();
        underlying.safeTransfer(beneficiary, amount);
    }
}
