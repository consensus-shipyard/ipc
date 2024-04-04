// SPDX-License-Identifier: MIT
pragma solidity 0.8.23;

import {DelegationManager} from "./DelegationManager.sol";

interface ISlasher {
    function setDelegationManager(DelegationManager _delegation) external;
}
