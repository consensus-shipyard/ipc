// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity ^0.8.23;

import {SubnetID} from "../structs/Subnet.sol";

// Event to be emitted within the subnet when a new activity summary has been recorded.
event ActivityRollupRecorded(uint64 checkpointHeight, FullActivityRollup rollup);

// Carries a set of reports summarising various aspects of the activity that took place in the subnet between the
// previous checkpoint and the checkpoint this summary is committed into. If this is the first checkpoint, the summary
// contains information about the subnet's activity since genesis.
// In the future we'll be having more kinds of activity reports here.
struct FullActivityRollup {
    /// A report of consensus-level activity that took place in the subnet between the previous checkpoint
    /// and the checkpoint this summary is committed into.
    /// @dev If there is a configuration change applied at this checkpoint, this carries information
    /// about the _old_ validator set.
    Consensus.FullSummary consensus;
}

// Compressed representation of the activity summary that can be embedded in checkpoints to propagate up the hierarchy.
struct CompressedActivityRollup {
    Consensus.CompressedSummary consensus;
}

/// Namespace for consensus-level activity summaries.
library Consensus {
    type MerkleHash is bytes32;

    // Aggregated stats for consensus-level activity.
    struct AggregatedStats {
        /// The total number of unique validators that have mined within this period.
        uint64 totalActiveValidators;
        /// The total number of blocks committed by all validators during this period.
        uint64 totalNumBlocksCommitted;
    }

    // The full activity summary for consensus-level activity.
    struct FullSummary {
        AggregatedStats stats;
        /// The breakdown of activity per validator.
        ValidatorData[] data;
    }

    // The compresed representation of the activity summary for consensus-level activity suitable for embedding in a checkpoint.
    struct CompressedSummary {
        AggregatedStats stats;
        /// The commitment for the validator details, so that we don't have to transmit them in full.
        MerkleHash dataRootCommitment;
    }

    struct ValidatorData {
        /// @dev The validator whose activity we're reporting about, identified by the Ethereum address corresponding
        /// to its secp256k1 pubkey.
        address validator;
        /// @dev The number of blocks committed by this validator during the summarised period.
        uint64 blocksCommitted;
    }

    /// The payload for validators to claim rewards
    struct ValidatorClaim {
        ValidatorData data;
        MerkleHash[] proof;
    }
}
