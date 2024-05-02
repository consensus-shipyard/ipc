// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity ^0.8.23;

import {MinPQ, LibMinPQ} from "./LibMinPQ.sol";
import {MaxPQ, LibMaxPQ} from "./LibMaxPQ.sol";
import {NotValidator, NotOwnerOfPublicKey, WithdrawExceedingCollateral, AddressShouldBeValidator, CannotConfirmFutureChanges} from "../../errors/IPCErrors.sol";
import {VALIDATOR_SECP256K1_PUBLIC_KEY_LENGTH} from "../../constants/Constants.sol";

/// @notice Subnet power change operations.
enum PowerOperation {
    NewPower,
    SetMetadata
}

/// @notice The change request to validator staking.
struct PowerChange {
    PowerOperation op;
    bytes payload;
    address validator;
}

/// @notice The change associated with its corresponding configuration number.
struct PowerChangeRequest {
    PowerChange change;
    uint64 configurationNumber;
}

/// @notice The collection of staking changes.
struct PowerChangeLog {
    /// @notice The next configuration number to assign to new changes.
    uint64 nextConfigurationNumber;
    /// @notice The starting configuration number stored.
    uint64 startConfigurationNumber;
    /// The details of the changes, mapping of configuration number to changes.
    mapping(uint64 => PowerChange) changes;
}

struct Validator {
    uint256 confirmedPower;
    uint256 unconfirmedPower;
    /// The metadata associated with the validator, i.e. off-chain network address.
    /// This information is not important to the protocol, off-chain should know how
    /// to parse or decode the bytes.
    bytes metadata;
}

/// @notice Proof of power is a generalisation of POS, which is using a generic power to rank the validators
/// @notice Keeping track of the list of validators.
/// @dev There are two types of validators:
///     - Active
///     - Waiting
/// Active validators are those that are producing blocks in the child subnet.
/// Waiting validators are those that do no have as high powers as Active validators.
///
/// The max number of active validators is limited by `activeLimit` and the size of waiting
/// validators is not bounded.
///
/// With each validator staking change, waiting validators can be promoted to active validators
/// and active validators can be knocked off.
struct ProofOfPower {
    /// The total number of active validators allowed.
    uint16 activeLimit;
    /// The total power confirmed.
    uint256 totalConfirmedPower;
    /// The mapping of each validator address to its information.
    mapping(address => Validator) validators;
    /// @notice The active validators tracked using min priority queue.
    MinPQ activeValidators;
    /// @notice The waiting validators tracked using max priority queue.
    MaxPQ waitingValidators;

    /// @notice Contains the list of changes to validator set. Configuration number is associated at each change.
    PowerChangeLog changeSet;
}

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

    /// @notice Updates the power of the validator
    function setPowerRequest(PowerChangeLog storage changes, address validator, uint256 power) internal {
        bytes memory payload = abi.encode(power);

        uint64 configurationNumber = recordChange({
            changes: changes,
            validator: validator,
            op: PowerOperation.NewPower,
            payload: payload
        });

        emit NewPowerChangeRequest({
            op: PowerOperation.NewPower,
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

library LibPowerQuery {
    using LibMinPQ for MinPQ;
    using LibMaxPQ for MaxPQ;

    // =============== Getters =============
    function getConfirmedPower(
        ProofOfPower storage self,
        address validator
    ) internal view returns(uint256) {
        return self.validators[validator].confirmedPower;
    }

    function getUnconfirmedPower(
        ProofOfPower storage self,
        address validator
    ) internal view returns(uint256) {
        return self.validators[validator].unconfirmedPower;
    }

    /// @notice Checks if the validator is an active validator
    function isActiveValidator(ProofOfPower storage self, address validator) internal view returns (bool) {
        return self.activeValidators.contains(validator);
    }

    /// @notice Checks if the validator is a waiting validator
    function isWaitingValidator(ProofOfPower storage self, address validator) internal view returns (bool) {
        return self.waitingValidators.contains(validator);
    }

    /// @notice Checks if the validator has power.
    /// @param validator The address to check for power.
    /// @return A boolean indicating whether the validator has power.
    function hasPower(ProofOfPower storage self, address validator) internal view returns (bool) {
        return self.validators[validator].unconfirmedPower != 0;
    }

    function listActiveValidators(ProofOfPower storage self) internal view returns (address[] memory addresses) {
        uint16 size = self.activeValidators.getSize();
        addresses = new address[](size);
        for (uint16 i = 1; i <= size; ) {
            addresses[i - 1] = self.activeValidators.getAddress(i);
            unchecked {
                ++i;
            }
        }
        return addresses;
    }

    /// @notice Get the total power of *active* validators.
    function totalConfirmedPowerOfActiveValidators(ProofOfPower storage self) internal view returns (uint256 power) {
        uint16 size = self.activeValidators.getSize();
        for (uint16 i = 1; i <= size; ) {
            address validator = self.activeValidators.getAddress(i);
            power += getConfirmedPower(self, validator);
            unchecked {
                ++i;
            }
        }
    }

    /// @notice Get the total confirmed power of the active validators.
    /// The function reverts if at least one validator is not in the active validator set.
    function totalConfirmedPowerOfActiveValidators(
        ProofOfPower storage self,
        address[] memory addresses
    ) internal view returns (uint256[] memory) {
        uint256 size = addresses.length;
        uint256[] memory activePowerTable = new uint256[](size);

        for (uint256 i; i < size; ) {
            if (!isActiveValidator(self, addresses[i])) {
                revert NotValidator(addresses[i]);
            }
            activePowerTable[i] = getConfirmedPower(self, addresses[i]);
            unchecked {
                ++i;
            }
        }
        return activePowerTable;
    }

    function totalActiveValidators(ProofOfPower storage self) internal view returns (uint16) {
        return self.activeValidators.getSize();
    }

    /// @notice Gets the total number of validators, including active and waiting
    function totalValidators(ProofOfPower storage self) internal view returns (uint16) {
        return self.waitingValidators.getSize() + self.activeValidators.getSize();
    }

    function getTotalConfirmedPower(ProofOfPower storage self) internal view returns (uint256) {
        return self.totalConfirmedPower;
    }

    function getConfigurationNumbers(ProofOfPower storage self) internal view returns(uint64, uint64) {
        return (self.changeSet.nextConfigurationNumber, self.changeSet.startConfigurationNumber);
    }
}

/// @notice Handles the proof of power with child subnet.
/// @dev This is a contract instead of a library so that hooks can be added for downstream use cases.
abstract contract PowerChangeInitiator {
    using LibPowerChangeLog for PowerChangeLog;
    using LibMaxPQ for MaxPQ;
    using LibMinPQ for MinPQ;

    event ActiveValidatorPowerUpdated(address validator, uint256 newPower);
    event WaitingValidatorPowerUpdated(address validator, uint256 newPower);
    event NewActiveValidator(address validator, uint256 power);
    event NewWaitingValidator(address validator, uint256 power);
    event ActiveValidatorReplaced(address oldValidator, address newValidator);
    event ActiveValidatorLeft(address validator);
    event WaitingValidatorLeft(address validator);

    uint64 internal constant INITIAL_CONFIGURATION_NUMBER = 1;

    event ConfigurationNumberConfirmed(uint64 number);

    /// @notice Hook for handling when the power of the validaor has changes
    function handlePowerChange(address validator, uint256 oldPower, uint256 newPower) internal virtual;

    /// @notice Set the metadata of a validator
    function setValidatorMetadata(ProofOfPower storage self, address validator, bytes calldata metadata) internal {
        self.changeSet.metadataRequest(validator, metadata);
    }

    /// @notice Increase the power of the validator
    function setNewPower(ProofOfPower storage self, address validator, uint256 power) internal {
        self.validators[validator].unconfirmedPower = power;
        self.changeSet.setPowerRequest(validator, power);
    }

    /// @notice Confirm the changes in bottom up checkpoint submission, only call this in bottom up checkpoint execution.
    function confirmChange(ProofOfPower storage self, uint64 configurationNumber) internal {
        PowerChangeLog storage changeSet = self.changeSet;

        if (configurationNumber >= changeSet.nextConfigurationNumber) {
            revert CannotConfirmFutureChanges();
        } else if (configurationNumber < changeSet.startConfigurationNumber) {
            return;
        }

        uint64 start = changeSet.startConfigurationNumber;
        for (uint64 i = start; i <= configurationNumber; ) {
            PowerChange storage change = changeSet.getChange(i);
            address validator = change.validator;

            if (change.op == PowerOperation.SetMetadata) {
                confirmMetadata(self, validator, change.payload);
            } else {
                uint256 newPower = abi.decode(change.payload, (uint256));
                confirmNewPower(self, validator, newPower);
            }

            changeSet.purgeChange(i);
            unchecked {
                ++i;
            }
        }

        changeSet.startConfigurationNumber = configurationNumber + 1;

        emit ConfigurationNumberConfirmed(configurationNumber);
    }

    /// @notice Confirm the metadata of a validator
    function confirmMetadata(ProofOfPower storage self, address validator, bytes memory metadata) internal {
        self.validators[validator].metadata = metadata;
    }

    function confirmNewPower(ProofOfPower storage self, address validator, uint256 newPower) internal {
        uint256 oldPower = self.validators[validator].confirmedPower;

        if (oldPower == newPower) {
            return;
        }

        self.validators[validator].confirmedPower = newPower;
        self.totalConfirmedPower = self.totalConfirmedPower - oldPower + newPower;

        if (newPower > oldPower) {
            increaseReshuffle({self: self, maybeActive: validator, newPower: newPower});
        } else {
            reduceReshuffle({self: self, validator: validator, newPower: newPower});
        }

        handlePowerChange(validator, oldPower, newPower);
    }

    function validatePublicKeys(
        address[] calldata validators,
        bytes[] calldata publicKeys
    ) internal pure {
        uint256 length = validators.length;
        for (uint256 i; i < length; ) {
            // check addresses
            address convertedAddress = publicKeyToAddress(publicKeys[i]);
            if (convertedAddress != validators[i]) {
                revert NotOwnerOfPublicKey();
            }

            unchecked {
                ++i;
            }
        }
    }

    /// @notice Converts a 65-byte public key to its corresponding address.
    /// @param publicKey The 65-byte public key to be converted.
    /// @return The address derived from the given public key.
    function publicKeyToAddress(bytes calldata publicKey) internal pure returns (address) {
        assert(publicKey.length == VALIDATOR_SECP256K1_PUBLIC_KEY_LENGTH);
        bytes32 hashed = keccak256(publicKey[1:]);
        return address(uint160(uint256(hashed)));
    }

    // ================ DO NOT CALL THESE METHODS OUTSIDE OF THIS LIB ===================

    /// @notice Reshuffles the active and waiting validators when an increase in power is confirmed
    function increaseReshuffle(ProofOfPower storage self, address maybeActive, uint256 newPower) internal {
        if (self.activeValidators.contains(maybeActive)) {
            self.activeValidators.increaseReheapify(self, maybeActive);
            emit ActiveValidatorPowerUpdated(maybeActive, newPower);
            return;
        }

        // incoming address is not active validator
        uint16 activeLimit = self.activeLimit;
        uint16 activeSize = self.activeValidators.getSize();
        if (activeLimit > activeSize) {
            // we can still take more active validators, just insert to the pq.
            self.activeValidators.insert(self, maybeActive);
            emit NewActiveValidator(maybeActive, newPower);
            return;
        }

        // now we have enough active validators, we need to check:
        // - if the incoming new collateral is more than the min active collateral,
        //     - yes:
        //        - pop the min active validator
        //        - remove the incoming validator from waiting validators
        //        - insert incoming validator into active validators
        //        - insert popped validator into waiting validators
        //     - no:
        //        - insert the incoming validator into waiting validators
        (address minAddress, uint256 minActivePower) = self.activeValidators.min(self);
        if (minActivePower < newPower) {
            self.activeValidators.pop(self);

            if (self.waitingValidators.contains(maybeActive)) {
                self.waitingValidators.deleteReheapify(self, maybeActive);
            }

            self.activeValidators.insert(self, maybeActive);
            self.waitingValidators.insert(self, minAddress);

            emit ActiveValidatorReplaced(minAddress, maybeActive);
            return;
        }

        if (self.waitingValidators.contains(maybeActive)) {
            self.waitingValidators.increaseReheapify(self, maybeActive);
            emit WaitingValidatorPowerUpdated(maybeActive, newPower);
            return;
        }

        self.waitingValidators.insert(self, maybeActive);
        emit NewWaitingValidator(maybeActive, newPower);
    }

    /// @notice Reshuffles the active and waiting validators when a power reduction is confirmed
    function reduceReshuffle(ProofOfPower storage self, address validator, uint256 newPower) internal {
        if (self.waitingValidators.contains(validator)) {
            if (newPower == 0) {
                self.waitingValidators.deleteReheapify(self, validator);
                emit WaitingValidatorLeft(validator);
                return;
            }
            self.waitingValidators.decreaseReheapify(self, validator);
            emit WaitingValidatorPowerUpdated(validator, newPower);
            return;
        }

        // sanity check
        if (!self.activeValidators.contains(validator)) {
            revert AddressShouldBeValidator();
        }

        // the validator is an active validator!

        if (newPower == 0) {
            self.activeValidators.deleteReheapify(self, validator);
            emit ActiveValidatorLeft(validator);

            if (self.waitingValidators.getSize() != 0) {
                (address toBePromoted, uint256 power) = self.waitingValidators.max(self);
                self.waitingValidators.pop(self);
                self.activeValidators.insert(self, toBePromoted);
                emit NewActiveValidator(toBePromoted, power);
            }

            return;
        }

        self.activeValidators.decreaseReheapify(self, validator);

        if (self.waitingValidators.getSize() == 0) {
            return;
        }

        (address mayBeDemoted, uint256 minActivePower) = self.activeValidators.min(self);
        (address mayBePromoted, uint256 maxWaitingPower) = self.waitingValidators.max(self);
        if (minActivePower < maxWaitingPower) {
            self.activeValidators.pop(self);
            self.waitingValidators.pop(self);
            self.activeValidators.insert(self, mayBePromoted);
            self.waitingValidators.insert(self, mayBeDemoted);

            emit ActiveValidatorReplaced(mayBeDemoted, mayBePromoted);
            return;
        }

        emit ActiveValidatorPowerUpdated(validator, newPower);
    }
}
