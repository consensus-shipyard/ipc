// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.7;

import "../lib/SubnetIDHelper.sol";
import "../structs/Checkpoint.sol";

/// @title Helper library for manipulating Checkpoint struct
/// @author LimeChain team
library CheckpointHelper {
    bytes32 private constant EMPTY_CROSSMSG_HASH =
        keccak256(
            abi.encode(
                CrossMsgMeta({
                    nonce: 0,
                    value: 0,
                    fee: 0,
                    msgs: new CrossMsg[](0)
                })
            )
        );

    bytes32 public constant EMPTY_CHECKPOINT_DATA_HASH =
        keccak256(
            abi.encode(
                CheckData({
                    source: SubnetID(new address[](0)),
                    tipSet: new bytes(0),
                    epoch: 0,
                    prevHash: bytes32(0),
                    children: new ChildCheck[](0),
                    crossMsgs: CrossMsgMeta({
                        nonce: 0,
                        value: 0,
                        fee: 0,
                        msgs: new CrossMsg[](0)
                    })
                })
            )
        );

    function toHash(
        Checkpoint memory checkpoint
    ) public pure returns (bytes32) {
        return keccak256(abi.encode(checkpoint.data));
    }

    function hasCrossMsgMeta(
        Checkpoint memory checkpoint
    ) public pure returns (bool) {
        return
            keccak256(abi.encode(checkpoint.data.crossMsgs)) !=
            EMPTY_CROSSMSG_HASH;
    }
}
