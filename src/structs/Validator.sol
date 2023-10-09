// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity 0.8.19;

import {FvmAddress} from "../structs/FvmAddress.sol";

// These structs use `FvmAddress` instead of plain `address` because they need to
// trespass subnet boundaries. As a rule of thumb, we use `address` for everything
// that is treated exclusively in Solidity, and `FvmAddress` for things that involve
// the FVM or may be impacted by having heterogeneous runtimes.

struct Validator {
    uint256 weight;
    FvmAddress addr;
}

struct Membership {
    Validator[] validators;
    uint64 configurationNumber;
    uint256 totalWeight;
}
