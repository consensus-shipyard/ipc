// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.7;

import "../lib/SubnetIDHelper.sol";
import "../structs/Checkpoint.sol";
import "../constants/Constants.sol";

/// @title Helper library for manipulating Checkpoint struct
/// @author LimeChain team
library CheckpointHelper {
    bytes32 constant EMPTY_TOPDOWNCHECKPOINT_HASH =
        keccak256(
            abi.encode(
                TopDownCheckpoint({epoch: 0, topDownMsgs: new CrossMsg[](0)})
            )
        );

    bytes32 constant EMPTY_BOTTOMUPCHECKPOINT_HASH =
        keccak256(
            abi.encode(
                BottomUpCheckpoint({
                    source: SubnetID(new address[](0)), 
                    epoch: 0,
                    fee: 0,
                    crossMsgs: new CrossMsg[](0),
                    children: new ChildCheck[](0),
                    prevHash: EMPTY_HASH
                })
            )
        );

    function toHash(
        BottomUpCheckpoint memory bottomupCheckpoint
    ) public pure returns (bytes32) {
        return keccak256(abi.encode(bottomupCheckpoint));
    }

    function toHash(
        TopDownCheckpoint memory topdownCheckpoint
    ) public pure returns (bytes32) {
        return keccak256(abi.encode(topdownCheckpoint));
    }

    function isEmpty(
        TopDownCheckpoint memory topdownCheckpoint
    ) public pure returns (bool) {
        return toHash(topdownCheckpoint) == EMPTY_TOPDOWNCHECKPOINT_HASH;
    }

    function isEmpty(
        BottomUpCheckpoint memory bottomUpCheckpoint
    ) public pure returns (bool) {
        return toHash(bottomUpCheckpoint) == EMPTY_BOTTOMUPCHECKPOINT_HASH;
    }
}
