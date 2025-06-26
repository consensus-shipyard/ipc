// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity ^0.8.23;

import {Consensus, CompressedActivityRollup, FullActivityRollup} from "../../contracts/structs/Activity.sol";
import {BottomUpBatch} from "../../contracts/structs/BottomUpBatch.sol";
import {IpcEnvelope} from "../../contracts/structs/CrossNet.sol";
import {LibBottomUpBatch} from "../../contracts/lib/LibBottomUpBatch.sol";
import {MerkleTreeHelper} from "./MerkleTreeHelper.sol";

library BottomUpBatchHelper {
    /// @notice Constructs a commitment from a batch of bottom-up messages.
    /// @dev This function is intended for testing; in production, commitments are constructed off-chain.
    function makeCommitment(IpcEnvelope[] memory msgs) internal returns (BottomUpBatch.Commitment memory) {
        if (msgs.length == 0) {
            return BottomUpBatch.Commitment({totalNumMsgs: 0, msgsRoot: BottomUpBatch.MerkleHash.wrap(bytes32(0))});
        }

        if (msgs.length == 1) {
            return BottomUpBatch.Commitment({totalNumMsgs: 1, msgsRoot: LibBottomUpBatch.makeLeaf(msgs[0])});
        }

        (bytes32 root, ) = MerkleTreeHelper.createMerkleProofsForBottomUpBatch(msgs);
        return
            BottomUpBatch.Commitment({
                totalNumMsgs: uint64(msgs.length),
                msgsRoot: BottomUpBatch.MerkleHash.wrap(root)
            });
    }

    /// @notice Constructs Merkle inclusion proofs for each message in a bottom-up batch.
    /// @dev This function is for testing purposes only; in production, inclusion proofs are generated off-chain.
    function makeInclusions(IpcEnvelope[] memory msgs) internal returns (BottomUpBatch.Inclusion[] memory) {
        if (msgs.length == 0) {
            return new BottomUpBatch.Inclusion[](0);
        }

        if (msgs.length == 1) {
            BottomUpBatch.MerkleHash[] memory proof = new BottomUpBatch.MerkleHash[](0);
            BottomUpBatch.Inclusion[] memory inclusions = new BottomUpBatch.Inclusion[](1);
            inclusions[0] = BottomUpBatch.Inclusion({msg: msgs[0], proof: proof});
            return inclusions;
        }

        (, bytes32[][] memory proofs) = MerkleTreeHelper.createMerkleProofsForBottomUpBatch(msgs);
        uint256 len = proofs.length;
        BottomUpBatch.Inclusion[] memory inclusions = new BottomUpBatch.Inclusion[](len);
        for (uint256 i = 0; i < len; i++) {
            BottomUpBatch.MerkleHash[] memory proof = BottomUpBatchHelper.wrapBytes32Array(proofs[i]);
            inclusions[i] = BottomUpBatch.Inclusion({msg: msgs[i], proof: proof});
        }
        return inclusions;
    }

    function wrapBytes32Array(bytes32[] memory data) internal pure returns (BottomUpBatch.MerkleHash[] memory wrapped) {
        uint256 length = data.length;

        if (length == 0) {
            return wrapped;
        }

        wrapped = new BottomUpBatch.MerkleHash[](data.length);
        for (uint256 i = 0; i < length; ) {
            wrapped[i] = BottomUpBatch.MerkleHash.wrap(data[i]);
            unchecked {
                i++;
            }
        }

        return wrapped;
    }

    function makeEmptyBatch() internal pure returns (IpcEnvelope[] memory msgs) {
        msgs = new IpcEnvelope[](0);
    }

    function makeEmptyCommitment() internal returns (BottomUpBatch.Commitment memory) {
        return makeCommitment(makeEmptyBatch());
    }
}
