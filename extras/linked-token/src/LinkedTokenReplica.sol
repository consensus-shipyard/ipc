// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity 0.8.23;

import {ERC20} from "openzeppelin-contracts/token/ERC20/ERC20.sol";
import {LinkedToken} from "./LinkedToken.sol";

/**
 * @title IpcTokenController
 * @notice Contract to handle token transfer from L1, lock them and mint on L2.
 */
contract LinkedTokenReplica is LinkedToken, ERC20 {
    using SafeERC20 for IERC20;

    function _captureTokens(address holder, uint256 amount) internal override {
        _burn(holder, amount);
    }

    function _releaseTokens(address beneficiary, uint256 amount) internal override {
        _mint(beneficiary, amount);
    }
}
