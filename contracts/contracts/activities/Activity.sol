// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity ^0.8.23;

import {SubnetID} from "../structs/Subnet.sol";

// Event to be emitted within the subnet when a new activity summary has been recorded.
event ActivitySummaryRecorded(FullActivitySummary summary);

// Carries a set of reports summarising various aspects of the activity that took place in the subnet between the
// previous checkpoint and the checkpoint this summary is committed into. If this is the first checkpoint, the summary
// contains information about the subnet's activity since genesis.
// In the future we'll be having more kinds of activity reports here.
struct FullActivitySummary {
    /// A report of consensus-level activity that took place in the subnet between the previous checkpoint
    /// and the checkpoint this summary is committed into.
    /// @dev If there is a configuration change applied at this checkpoint, this carries information
    /// about the _old_ validator set.
    Consensus.Full consensus;
}

// Compressed representation of the activity summary that can be embedded in checkpoints to propagate up the hierarchy.
struct CompressedActivitySummary {
    Consensus.Compressed consensus;
}

/// Namespace for consensus-level activity summaries.
library Consensus {
    // Aggregated stats for consensus-level activity.
    struct Aggregated {
        /// The total number of unique validators that have mined within this period.
        uint64 totalActiveValidators;
        /// The total number of blocks committed by all validators during this period.
        uint64 totalNumBlocksCommitted;
    }

    // The full activity summary for consensus-level activity.
    struct Full {
        Aggregated aggregated;
        /// The breakdown of activity per validator.
        ValidatorDetail[] validatorDetails;
    }

    // The compresed representation of the activity summary for consensus-level activity suitable for embedding in a checkpoint.
    struct Compressed {
        Aggregated aggregated;
        /// The commitment for the validator details, so that we don't have to transmit them in full.
        bytes32 commitment;
    }

    struct ValidatorDetail {
        /// @dev The validator whose activity we're reporting about, identified by the Ethereum address corresponding
        /// to its secp256k1 pubkey.
        address validator;
        /// @dev The number of blocks committed by this validator during the summarised period.
        uint64 blocksCommitted;
    }
}

/// The proof required for validators to claim rewards
struct ValidatorClaimProof {
    ValidatorSummary summary;
    bytes32[] proof;
}

/// The proofs to batch claim validator rewards in a specific subnet
/// REVIEW(raulk): Delete this type. Make the method just take the subnet ID and the list of claim proofs.
struct BatchClaimProofs {
    SubnetID subnetId;
    ValidatorClaimProof[] proofs;
}
