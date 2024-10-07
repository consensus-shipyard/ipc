// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity ^0.8.23;

/// The commitment for the validator rewards that child subnet submits to the parent subnet
/// together with a bottom up checkpoint
struct ValidatorRewardCommitment {
    /// The commitment for the parent subnet
    bytes32 commitment;

    // TODO: add relayed rewarder commitment
}

/// The summary for a single validator 
struct ValidatorSummary {
    /// @dev The validator whose activity we're reporting about.
    address validator;
    /// @dev The number of blocks committed by each validator in the position they appear in the validators array.
    /// If there is a configuration change applied at this checkpoint, this carries information about the _old_ validator set.
    uint64 blocksCommitted;
    /// @dev Other metadata
    bytes metadata;
}

/// A summary of validator's activity in the child subnet. This is submitted to the parent for reward distribution.
struct ActivitySummary {
    /// @dev The block range the activity summary spans; these are the local heights of the start and the end, inclusive.
    uint256[2] blockRange;
    ValidatorSummary[] activities;
}

library LibActivitySummary {
    function numValidators(ActivitySummary calldata self) internal pure returns(uint64) {
        return uint64(self.activities.length);
    }

    function commitment(ActivitySummary calldata self) internal pure returns(bytes32) {
        return keccak256(abi.encode(self));
    }

    function containsValidator(ActivitySummary calldata self, address validator) internal pure returns(bool) {
        uint256 len = self.activities.length;
        for (uint256 i = 0; i < len; ) {
            if (self.activities[i].validator == validator) {
                return true;
            }

            unchecked {
                i++;
            }
        }

        return false;
    }
}