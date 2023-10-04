// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity 0.8.19;

import {FvmAddress} from "../structs/FvmAddress.sol";

struct Validator {
    uint256 weight;
    FvmAddress addr;
}

struct Membership {
    Validator[] validators;
    uint64 configurationNumber;
    uint256 totalWeight;
}
