// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.7;

import "./Checkpoint.sol";

struct PostBoxItem {
    CrossMsg crossMsg;
    address[] owners;
}
