// SPDX-License-Identifier: UNLICENSED
pragma solidity 0.8.18;

/// @title Subnet Consensus Type enum
/// @author LimeChain team
enum ConsensusType {
    Delegated,
    PoW,
    Tendermint,
    Mir,
    FilecoinEC,
    Dummy
}
