// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity ^0.8.23;

import {Consensus} from "../../contracts/activities/Activity.sol";

library ActivityHelper {
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
}
