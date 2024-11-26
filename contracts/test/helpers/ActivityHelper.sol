// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity ^0.8.23;

import {Consensus, CompressedActivityRollup, FullActivityRollup} from "../../contracts/structs/Activity.sol";

library ActivityHelper {
    function newCompressedActivityRollup(
        uint64 totalActiveValidators,
        uint64 totalNumBlocksCommitted,
        bytes32 detailsRootCommitment
    ) internal pure returns (CompressedActivityRollup memory compressed) {
        Consensus.CompressedSummary memory summary = newCompressedSummary(
            totalActiveValidators,
            totalNumBlocksCommitted,
            detailsRootCommitment
        );
        compressed.consensus = summary;
        return compressed;
    }

    function newCompressedSummary(
        uint64 totalActiveValidators,
        uint64 totalNumBlocksCommitted,
        bytes32 detailsRootCommitment
    ) internal pure returns (Consensus.CompressedSummary memory summary) {
        summary.stats.totalActiveValidators = totalActiveValidators;
        summary.stats.totalNumBlocksCommitted = totalNumBlocksCommitted;
        summary.dataRootCommitment = Consensus.MerkleHash.wrap(detailsRootCommitment);
    }

    function wrapBytes32Array(bytes32[] memory data) internal pure returns (Consensus.MerkleHash[] memory wrapped) {
        uint256 length = data.length;

        if (length == 0) {
            return wrapped;
        }

        wrapped = new Consensus.MerkleHash[](data.length);
        for (uint256 i = 0; i < length; ) {
            wrapped[i] = Consensus.MerkleHash.wrap(data[i]);
            unchecked {
                i++;
            }
        }

        return wrapped;
    }

    function dummyActivityRollup() internal pure returns (FullActivityRollup memory rollup) {
        Consensus.ValidatorData[] memory data = new Consensus.ValidatorData[](0);
        rollup = FullActivityRollup({
            consensus: Consensus.FullSummary({
                stats: Consensus.AggregatedStats({totalActiveValidators: 0, totalNumBlocksCommitted: 0}),
                data: data
            })
        });
        return rollup;
    }
}
