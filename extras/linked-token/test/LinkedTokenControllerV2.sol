// SPDX-License-Identifier: MIT
pragma solidity 0.8.23;

import {LinkedTokenController} from "../src/LinkedTokenController.sol";

contract LinkedTokenControllerV2 is LinkedTokenController {
    function newFunctionReturns7() public returns (uint256) {
        return 7;
    }
}
