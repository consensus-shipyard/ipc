// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.7;

import "./Checkpoint.sol";

/// @title postbox item struct
/// @author LimeChain team
struct PostBoxItem {
    CrossMsg crossMsg;
    address[] owners;
}
