// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity ^0.8.23;

import {PowerChangeLog, PowerChange, PowerOperation} from "../structs/Subnet.sol";

/// The util library for `PowerChangeLog`
library LibPowerChangeLog {
    event NewPowerChangeRequest(PowerOperation op, address validator, bytes payload, uint64 configurationNumber);

    /// @notice Validator request to update its metadata
    function metadataRequest(PowerChangeLog storage changes, address validator, bytes calldata metadata) internal {
        uint64 configurationNumber = recordChange({
            changes: changes,
            validator: validator,
            op: PowerOperation.SetMetadata,
            payload: metadata
        });

        emit NewPowerChangeRequest({
            op: PowerOperation.SetMetadata,
            validator: validator,
            payload: metadata,
            configurationNumber: configurationNumber
        });
    }

    /// @notice Records a request to set the new power of a validator
    function setPowerRequest(
        PowerChangeLog storage changes,
        address validator,
        uint256 power
    ) internal {
        bytes memory payload = abi.encode(power);

        uint64 configurationNumber = recordChange({
            changes: changes,
            validator: validator,
            op: PowerOperation.SetPower,
            payload: payload
        });

        emit NewPowerChangeRequest({
            op: PowerOperation.SetPower,
            validator: validator,
            payload: payload,
            configurationNumber: configurationNumber
        });
    }

    /// @notice Perform upsert operation to the deposit changes
    function recordChange(
        PowerChangeLog storage changes,
        address validator,
        PowerOperation op,
        bytes memory payload
    ) internal returns (uint64 configurationNumber) {
        configurationNumber = changes.nextConfigurationNumber;

        changes.changes[configurationNumber] = PowerChange({op: op, validator: validator, payload: payload});

        changes.nextConfigurationNumber = configurationNumber + 1;
    }

    /// @notice Get the change at configuration number
    function getChange(
        PowerChangeLog storage changes,
        uint64 configurationNumber
    ) internal view returns (PowerChange storage) {
        return changes.changes[configurationNumber];
    }

    function purgeChange(PowerChangeLog storage changes, uint64 configurationNumber) internal {
        delete changes.changes[configurationNumber];
    }
}
