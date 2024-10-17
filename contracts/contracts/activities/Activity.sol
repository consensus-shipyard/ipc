// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity ^0.8.23;

/// The summary for the child subnet activities that should be submitted to the parent subnet
/// together with a bottom up checkpoint
struct ActivitySummary {
    /// The total number of distintive validators that have mined
    uint64 totalActiveValidators;
    /// The activity commitment for validators
    bytes32 commitment;

    // TODO: add relayed rewarder commitment
}

/// The summary for a single validator
struct ValidatorSummary {
    /// @dev The child subnet checkpoint height associated with this summary
    uint64 checkpointHeight;
    /// @dev The validator whose activity we're reporting about.
    address validator;
    /// @dev The number of blocks committed by each validator in the position they appear in the validators array.
    /// If there is a configuration change applied at this checkpoint, this carries information about the _old_ validator set.
    uint64 blocksCommitted;
    /// @dev Other metadata
    bytes metadata;
}
