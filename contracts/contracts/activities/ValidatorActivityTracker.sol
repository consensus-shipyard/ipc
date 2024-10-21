// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity ^0.8.23;

import {EnumerableSet} from "@openzeppelin/contracts/utils/structs/EnumerableSet.sol";

import {ValidatorSummary, ActivitySummary} from "./Activity.sol";
import {SystemContract} from "../lib/LibGatewayActorStorage.sol";


/// The validator reward facet for the child subnet, i.e. for child subnet to track validators's activies
/// and create the commitment.
contract ValidatorActivityTracker is SystemContract {
    using EnumerableSet for EnumerableSet.AddressSet;

    /// @dev The starting height of validator's mining activities since the last purged block
    uint64 startHeight;
    /// @dev The list of validator who have participated in mining since `startHeight`
    EnumerableSet.AddressSet validators;
    /// Tracks the number of blocks a validator has committed since `startHeight`
    mapping(address => uint64) blocksCommitted;

    /// Validators claim their reward for doing work in the child subnet
    function recordValidatorActivity(address validator) external systemActorOnly {
        blocksCommitted[validator] += 1;

        if (!validators.contains(validator)) {
            validators.add(validator);
        }
    }

    /// Reads the current validator summary
    function getSummary() external view returns(ActivitySummary memory summary) {
        summary.blockRange = [startHeight, block.number];

        // prepare the activities
        uint256 num_validators = validators.length();

        summary.activities = new ValidatorSummary[](num_validators);
        for (uint256 i = 0; i < num_validators; ) {
            address validator = validators.at(i);
            bytes memory metadata = new bytes(0);

            summary.activities[i] = ValidatorSummary({
                validator: validator,
                blocksCommitted: blocksCommitted[validator],
                metadata: metadata    
            });

            unchecked {
                i++;
            }
        }
    }

    /// Reads the current validator summary and purge the data accordingly
    /// @dev Call this method only when bottom up checkpoint needs to be created
    function purge_activities() external systemActorOnly {
        // prepare the activities
        uint256 num_validators = validators.length();

        for (uint256 i = num_validators - 1; i >= 0; ) {
            address validator = validators.at(i);

            delete blocksCommitted[validator];
            validators.remove(validator);

            unchecked {
                if (i == 0) { break; }
                i--;
            }
        }

        startHeight = uint64(block.number);
    }
}
