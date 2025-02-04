// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity ^0.8.23;

import {Asset} from "../structs/Subnet.sol";

/// @title Subnet actor interface
interface ISubnetActor {
    function supplySource() external view returns (Asset memory);
}
