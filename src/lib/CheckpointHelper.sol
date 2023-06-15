// SPDX-License-Identifier: UNLICENSED
pragma solidity 0.8.18;

import "../lib/SubnetIDHelper.sol";
import "../structs/Checkpoint.sol";
import "../constants/Constants.sol";

/// @title Helper library for manipulating Checkpoint struct
/// @author LimeChain team
library CheckpointHelper {
    using SubnetIDHelper for SubnetID;

    bytes32 constant EMPTY_TOPDOWNCHECKPOINT_HASH =
        keccak256(abi.encode(TopDownCheckpoint({epoch: 0, topDownMsgs: new CrossMsg[](0)})));

    bytes32 constant EMPTY_BOTTOMUPCHECKPOINT_HASH = keccak256(
        abi.encode(
            BottomUpCheckpoint({
                source: SubnetID(0, new address[](0)),
                epoch: 0,
                fee: 0,
                crossMsgs: new CrossMsg[](0),
                children: new ChildCheck[](0),
                prevHash: EMPTY_HASH
            })
        )
    );

    function toHash(BottomUpCheckpoint memory bottomupCheckpoint) public pure returns (bytes32) {
        return keccak256(abi.encode(bottomupCheckpoint));
    }

    function toHash(TopDownCheckpoint memory topdownCheckpoint) public pure returns (bytes32) {
        return keccak256(abi.encode(topdownCheckpoint));
    }

    function isEmpty(TopDownCheckpoint memory topdownCheckpoint) public pure returns (bool) {
        return toHash(topdownCheckpoint) == EMPTY_TOPDOWNCHECKPOINT_HASH;
    }

    function isEmpty(BottomUpCheckpoint memory bottomUpCheckpoint) public pure returns (bool) {
        return toHash(bottomUpCheckpoint) == EMPTY_BOTTOMUPCHECKPOINT_HASH;
    }

    function setChildCheck(
        BottomUpCheckpoint storage checkpoint,
        BottomUpCheckpoint calldata commit,
        mapping(uint64 => mapping(bytes32 => uint256[2])) storage children,
        mapping(uint64 => mapping(bytes32 => mapping(bytes32 => bool))) storage checks,
        uint64 currentEpoch
    ) public {
        bytes32 commitSource = commit.source.toHash();
        bytes32 commitData = toHash(commit);

        uint256[2] memory child = children[currentEpoch][commitSource];
        uint256 childIndex = child[0]; // index at checkpoint.data.children for the given subnet
        bool childExists = child[1] == 1; // 0 - no, 1 - yes

        if (childExists == false) {
            checkpoint.children.push(ChildCheck({source: commit.source, checks: new bytes32[](0)}));
            childIndex = checkpoint.children.length - 1;
        }

        checkpoint.children[childIndex].checks.push(commitData);

        children[currentEpoch][commitSource][0] = childIndex;
        children[currentEpoch][commitSource][1] = 1;
        checks[currentEpoch][commitSource][commitData] = true;
    }
}
