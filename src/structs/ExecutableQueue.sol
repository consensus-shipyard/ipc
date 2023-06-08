// SPDX-License-Identifier: UNLICENSED
pragma solidity 0.8.18;

struct ExecutableQueue {
    uint64 period; // number of blocks per epoch
    uint64 first; // next epoch
    uint64 last; // last epoch
    mapping(uint64 => bool) epochs; // epoch => exist
}
