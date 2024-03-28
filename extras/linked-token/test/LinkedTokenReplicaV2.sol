// SPDX-License-Identifier: MIT
pragma solidity 0.8.23;

import {LinkedTokenReplica} from "../src/LinkedTokenReplica.sol";

contract LinkedTokenReplicaV2 is LinkedTokenReplica {
    function newFunctionReturns8() public returns (uint256) {
        return 8;
    }
}
