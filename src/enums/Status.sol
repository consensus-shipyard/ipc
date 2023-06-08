// SPDX-License-Identifier: UNLICENSED
pragma solidity 0.8.18;

/// @title Subnet Status enum
/// @author LimeChain team
enum Status {
    Unset,
    Instantiated,
    Active,
    Inactive,
    Terminating,
    Killed
}
