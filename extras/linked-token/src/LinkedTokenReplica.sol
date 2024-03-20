// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity 0.8.23;

import {IERC20} from "openzeppelin-contracts/token/ERC20/IERC20.sol";
import {ERC20} from "openzeppelin-contracts/token/ERC20/ERC20.sol";
import {
    SafeERC20
} from "openzeppelin-contracts/token/ERC20/utils/SafeERC20.sol";
import {LinkedToken} from "./LinkedToken.sol";
import {SubnetID} from "@ipc/src/structs/Subnet.sol";


import {ERC20Facet} from "@PieVaults/facets/ERC20/ERC20Facet.sol";

/**
 * @title IpcTokenController
 * @notice Contract to handle token transfer from L1, lock them and mint on L2.
 */
contract LinkedTokenReplica is LinkedTokenFacet, ERC20Facet {
    using SafeERC20 for IERC20;

    function _captureTokens(address holder, uint256 amount) internal override {
        _burn(holder, amount);
    }

    function _releaseTokens(address beneficiary, uint256 amount)
        internal
        override
    {
        _mint(beneficiary, amount);
    }
}
