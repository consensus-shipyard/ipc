// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity ^0.8.23;

import {IGateway} from "../interfaces/IGateway.sol";
import {LibSubnetActorStorage, SubnetActorStorage} from "./LibSubnetActorStorage.sol";
import {LibMaxPQ, MaxPQ} from "./priority/LibMaxPQ.sol";
import {LibMinPQ, MinPQ} from "./priority/LibMinPQ.sol";
import {LibPowerChangeLog} from "./LibPowerChangeLog.sol";
import {AssetHelper} from "./AssetHelper.sol";
import {PermissionMode, StakingReleaseQueue, PowerChangeLog, PowerChange, PowerChangeRequest, PowerOperation, StakingRelease, ValidatorSet, AddressStakingReleases, ParentValidatorsTracker, Validator, Asset} from "../structs/Subnet.sol";
import {PowerReductionMoreThanTotal, NotValidator, CannotConfirmFutureChanges, NoCollateralToWithdraw, AddressShouldBeValidator, InvalidConfigurationNumber} from "../errors/IPCErrors.sol";
import {Address} from "@openzeppelin/contracts/utils/Address.sol";

library LibAddressStakingReleases {
    /// @notice Add new release to the storage. Caller makes sure the release.releasedAt is ordered
    /// @notice in ascending order. This method does not do checks on this.
    function push(AddressStakingReleases storage self, StakingRelease memory release) internal {
        uint16 idx = self.totalReleases;

        self.releases[idx] = release;
        self.totalReleases = idx + 1;
    }

    /// @notice Perform compaction on releases, i.e. aggregates the amount that can be released
    /// @notice and removes them from storage. Returns the total amount to release and the new
    /// @notice number of pending releases after compaction.
    function compact(AddressStakingReleases storage self) internal returns (uint256, uint16) {
        uint16 toCollectIdx = self.toCollectIdx;
        uint16 totalReleases = self.totalReleases;

        if (toCollectIdx == totalReleases) {
            revert NoCollateralToWithdraw();
        }

        uint256 amount;
        for (; toCollectIdx < totalReleases;) {
            StakingRelease memory release = self.releases[toCollectIdx];

            // releases are ordered ascending by releaseAt, no need to check
            // further as they will still be locked.
            if (release.releaseAt > block.number) {
                break;
            }

            amount += release.amount;
            delete self.releases[toCollectIdx];

            unchecked {
                ++toCollectIdx;
            }
        }

        self.toCollectIdx = toCollectIdx;

        return (amount, totalReleases - toCollectIdx);
    }
}

/// The util library for `StakingReleaseQueue`
library LibStakingReleaseQueue {
    using Address for address payable;
    using LibAddressStakingReleases for AddressStakingReleases;

    event NewCollateralRelease(address validator, uint256 amount, uint256 releaseBlock);

    function setLockDuration(StakingReleaseQueue storage self, uint256 blocks) internal {
        self.lockingDuration = blocks;
    }

    /// @notice Set the amount and time for release collateral
    function addNewRelease(StakingReleaseQueue storage self, address validator, uint256 amount) internal {
        uint256 releaseAt = block.number + self.lockingDuration;
        StakingRelease memory release = StakingRelease({releaseAt: releaseAt, amount: amount});

        self.releases[validator].push(release);

        emit NewCollateralRelease({validator: validator, amount: amount, releaseBlock: releaseAt});
    }

    /// @notice Validator claim the available collateral that are released
    function claim(StakingReleaseQueue storage self, address validator) internal returns (uint256) {
        (uint256 amount, uint16 newLength) = self.releases[validator].compact();

        if (newLength == 0) {
            delete self.releases[validator];
        }

        return amount;
    }
}

/// The util library for `ValidatorSet`
library LibValidatorSet {
    using LibMinPQ for MinPQ;
    using LibMaxPQ for MaxPQ;

    event ActiveValidatorCollateralUpdated(address validator, uint256 newPower);
    event WaitingValidatorCollateralUpdated(address validator, uint256 newPower);
    event NewActiveValidator(address validator, uint256 power);
    event NewWaitingValidator(address validator, uint256 power);
    event ActiveValidatorReplaced(address oldValidator, address newValidator);
    event ActiveValidatorLeft(address validator);
    event WaitingValidatorLeft(address validator);

    /// @notice Get the confirmed collateral of the validator.
    function getCurrentPower(
        ValidatorSet storage validators,
        address validator
    ) internal view returns (uint256 collateral) {
        collateral = validators.validators[validator].currentPower;
    }

    function listActiveValidators(ValidatorSet storage validators) internal view returns (address[] memory addresses) {
        uint16 size = validators.activeValidators.getSize();
        addresses = new address[](size);
        for (uint16 i = 1; i <= size; ) {
            addresses[i - 1] = validators.activeValidators.getAddress(i);
            unchecked {
                ++i;
            }
        }
        return addresses;
    }

    function listWaitingValidators(ValidatorSet storage validators) internal view returns (address[] memory addresses) {
        uint16 size = validators.waitingValidators.getSize();
        addresses = new address[](size);
        for (uint16 i = 1; i <= size; ) {
            addresses[i - 1] = validators.waitingValidators.getAddress(i);
            unchecked {
                ++i;
            }
        }
        return addresses;
    }

    /// @notice Get the total current power of *active* validators.
    function getTotalActivePower(ValidatorSet storage validators) internal view returns (uint256 power) {
        uint16 size = validators.activeValidators.getSize();
        for (uint16 i = 1; i <= size; ) {
            address validator = validators.activeValidators.getAddress(i);
            power += getCurrentPower(validators, validator);
            unchecked {
                ++i;
            }
        }
    }

    /// @notice Get the total power of the validators.
    /// The function reverts if at least one validator is not in the active validator set.
    function getTotalPowerOfValidators(
        ValidatorSet storage validators,
        address[] memory addresses
    ) internal view returns (uint256[] memory) {
        uint256 size = addresses.length;
        uint256[] memory activePowerTable = new uint256[](size);

        for (uint256 i; i < size; ) {
            if (!isActiveValidator(validators, addresses[i])) {
                revert NotValidator(addresses[i]);
            }
            activePowerTable[i] = getCurrentPower(validators, addresses[i]);
            unchecked {
                ++i;
            }
        }
        return activePowerTable;
    }

    function isActiveValidator(ValidatorSet storage self, address validator) internal view returns (bool) {
        return self.activeValidators.contains(validator);
    }

    /***********************************************************************
     * Internal helper functions, should not be called by external functions
     ***********************************************************************/
    
    /// @notice Set validator data
    function setMetadata(ValidatorSet storage validators, address validator, bytes calldata metadata) internal {
        validators.validators[validator].metadata = metadata;
    }

    /// @notice Increase the next power of a validator
    function increasePower(ValidatorSet storage validators, address validator, uint256 change) internal returns(uint256) {
        uint256 total = validators.validators[validator].nextPower;

        total += change;
        validators.validators[validator].nextPower = total;

        return total;
    }

    /// @notice Decrease the next power of a validator
    function decreasePower(ValidatorSet storage validators, address validator, uint256 change) internal returns(uint256) {
        uint256 total = validators.validators[validator].nextPower;
        if (total < change) {
            revert PowerReductionMoreThanTotal(total, change);
        }

        total -= change;
        validators.validators[validator].nextPower = total;

        return total;
    }

    /// @notice Update the validator's next power to a new value
    function setPower(ValidatorSet storage validators, address validator, uint256 newPower) internal returns(uint256) {
        validators.validators[validator].nextPower = newPower;
        return newPower;
    }

    /// @notice Set the validator federated power directly without queueing the request
    function setPowerWithConfirm(ValidatorSet storage validators, address validator, uint256 power) internal {
        setPower(validators, validator, power);
        confirmPower(validators, validator, power);
    }

    /// @notice Validator's power update was confirmed in the child subnet
    /// TODO: rename this to setPower and remove setPower when staking is shifted out of LibPower
    /// @return the old power of the validator
    function confirmPower(ValidatorSet storage self, address validator, uint256 power) internal returns(uint256) {
        uint256 oldPower = self.validators[validator].currentPower;
        self.validators[validator].currentPower = power;

        if (oldPower == power) {
            return oldPower;
        } else if (oldPower < power) {
            // oldPower < power, that mean validator power increased, should add (power - oldPower) to currentTotalPower
            // which is self.currentTotalPower + (power - oldPower)
            increaseReshuffle({self: self, maybeActive: validator, newPower: power});
        } else {
            // oldPower > power, that mean validator power dropped, should minus (oldPower - power)
            // which is self.currentTotalPower - (oldPower - power)
            reduceReshuffle({self: self, validator: validator, newPower: power});
        }

        self.currentTotalPower = self.currentTotalPower + power - oldPower;

        return oldPower;
    }

    /// @notice Reshuffles the active and waiting validators when an increase in power is confirmed
    function increaseReshuffle(ValidatorSet storage self, address maybeActive, uint256 newPower) internal {
        if (self.activeValidators.contains(maybeActive)) {
            self.activeValidators.increaseReheapify(self, maybeActive);
            emit ActiveValidatorCollateralUpdated(maybeActive, newPower);
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
            emit WaitingValidatorCollateralUpdated(maybeActive, newPower);
            return;
        }

        self.waitingValidators.insert(self, maybeActive);
        emit NewWaitingValidator(maybeActive, newPower);
    }

    /// @notice Reshuffles the active and waiting validators when a power reduction is confirmed
    function reduceReshuffle(ValidatorSet storage self, address validator, uint256 newPower) internal {
        if (self.waitingValidators.contains(validator)) {
            if (newPower == 0) {
                self.waitingValidators.deleteReheapify(self, validator);
                emit WaitingValidatorLeft(validator);
                return;
            }
            self.waitingValidators.decreaseReheapify(self, validator);
            emit WaitingValidatorCollateralUpdated(validator, newPower);
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

        emit ActiveValidatorCollateralUpdated(validator, newPower);
    }
}

library LibPower {
    using LibStakingReleaseQueue for StakingReleaseQueue;
    using LibPowerChangeLog for PowerChangeLog;
    using LibValidatorSet for ValidatorSet;
    using AssetHelper for Asset;
    using LibMaxPQ for MaxPQ;
    using LibMinPQ for MinPQ;
    using Address for address payable;

    uint64 internal constant INITIAL_CONFIGURATION_NUMBER = 1;

    event ConfigurationNumberConfirmed(uint64 number);
    event CollateralClaimed(address validator, uint256 amount);

    // =============== Getters =============
    function getCurrentPower(
        address validator
    ) internal view returns(uint256 power) {
        SubnetActorStorage storage s = LibSubnetActorStorage.appStorage();
        return s.validatorSet.getCurrentPower(validator);
    }

    /// @notice Checks if the validator is an active validator
    function isActiveValidator(address validator) internal view returns (bool) {
        SubnetActorStorage storage s = LibSubnetActorStorage.appStorage();
        return s.validatorSet.isActiveValidator(validator);
    }

    /// @notice Checks if the validator is a waiting validator
    function isWaitingValidator(address validator) internal view returns (bool) {
        SubnetActorStorage storage s = LibSubnetActorStorage.appStorage();
        return s.validatorSet.waitingValidators.contains(validator);
    }

    /// @notice Checks if the provided address is a validator (active or waiting) based on its total collateral.
    /// @param addr The address to check for validator status.
    /// @return A boolean indicating whether the address is a validator.
    function isValidator(address addr) internal view returns (bool) {
        SubnetActorStorage storage s = LibSubnetActorStorage.appStorage();

        // gas-opt: original check: nextPower > 0
        return s.validatorSet.validators[addr].nextPower != 0;
    }

    function totalActiveValidators() internal view returns (uint16) {
        SubnetActorStorage storage s = LibSubnetActorStorage.appStorage();
        return s.validatorSet.activeValidators.getSize();
    }

    /// @notice Gets the total number of validators, including active and waiting
    function totalValidators() internal view returns (uint16) {
        SubnetActorStorage storage s = LibSubnetActorStorage.appStorage();
        return s.validatorSet.waitingValidators.getSize() + s.validatorSet.activeValidators.getSize();
    }

    /// @notice Returns all active validators.
    function listActiveValidators() internal view returns (address[] memory addresses) {
        SubnetActorStorage storage s = LibSubnetActorStorage.appStorage();
        return s.validatorSet.listActiveValidators();
    }

    /// @notice Returns all waiting validators.
    function listWaitingValidators() internal view returns (address[] memory addresses) {
        SubnetActorStorage storage s = LibSubnetActorStorage.appStorage();
        return s.validatorSet.listWaitingValidators();
    }

    function getTotalCurrentPower() internal view returns (uint256) {
        SubnetActorStorage storage s = LibSubnetActorStorage.appStorage();
        return s.validatorSet.currentTotalPower;
    }

    /// @notice Gets the total collateral the validators has staked.
    function totalValidatorCollateral(address validator) internal view returns (uint256) {
        SubnetActorStorage storage s = LibSubnetActorStorage.appStorage();
        return s.validatorSet.validators[validator].nextPower;
    }

    // =============== Operations directly confirm =============

    /// @notice Set the validator federated power directly without queueing the request
    function setPowerWithConfirm(address validator, uint256 power) internal {
        SubnetActorStorage storage s = LibSubnetActorStorage.appStorage();
        s.validatorSet.setPowerWithConfirm(validator, power);
    }

    /// @notice Set the validator metadata directly without queueing the request
    function setMetadataWithConfirm(address validator, bytes calldata metadata) internal {
        SubnetActorStorage storage s = LibSubnetActorStorage.appStorage();
        s.validatorSet.setMetadata(validator, metadata);
    }

    /// @notice Confirm the deposit directly without going through the confirmation process
    function depositWithConfirm(address validator, uint256 amount) internal {
        SubnetActorStorage storage s = LibSubnetActorStorage.appStorage();

        uint256 newPower = s.validatorSet.increasePower(validator, amount);
        s.validatorSet.confirmPower(validator, newPower);
    }

    /// @notice Confirm the withdraw directly without going through the confirmation process
    /// and releasing from the gateway.
    /// @dev only use for non-bootstrapped subnets
    function withdrawWithConfirm(address validator, uint256 amount) internal {
        SubnetActorStorage storage s = LibSubnetActorStorage.appStorage();

        uint256 newPower = s.validatorSet.decreasePower(validator, amount);
        s.validatorSet.confirmPower(validator, newPower);
    }

    // ================= Operations that are queued ==============
    /// @notice Set the federated power of the validator
    function setFederatedPower(address validator, bytes calldata metadata, uint256 newPower) internal {
        SubnetActorStorage storage s = LibSubnetActorStorage.appStorage();

        s.validatorSet.setPower(validator, newPower);

        s.changeSet.setPowerRequest(validator, newPower);
        s.changeSet.metadataRequest(validator, metadata);
    }

    /// @notice Set the validator metadata
    function setValidatorMetadata(address validator, bytes calldata metadata) internal {
        SubnetActorStorage storage s = LibSubnetActorStorage.appStorage();
        s.changeSet.metadataRequest(validator, metadata);
    }

    /// @notice Deposit the collateral
    function deposit(address validator, uint256 amount) internal {
        SubnetActorStorage storage s = LibSubnetActorStorage.appStorage();

        uint256 nextPower = s.validatorSet.increasePower(validator, amount);
        s.changeSet.setPowerRequest(validator, nextPower);
    }

    /// @notice Withdraw the collateral
    function withdraw(address validator, uint256 amount) internal {
        SubnetActorStorage storage s = LibSubnetActorStorage.appStorage();

        uint256 nextPower = s.validatorSet.decreasePower(validator, amount);
        s.changeSet.setPowerRequest(validator, nextPower);
    }

    // =============== Other functions ================
    function getConfigurationNumbers() internal view returns(uint64, uint64) {
        SubnetActorStorage storage s = LibSubnetActorStorage.appStorage();
        return (s.changeSet.nextConfigurationNumber, s.changeSet.startConfigurationNumber);
    }

    /// @notice Claim the released collateral
    function claimCollateral(address validator) internal returns(uint256 amount) {
        SubnetActorStorage storage s = LibSubnetActorStorage.appStorage();
        amount = s.releaseQueue.claim(validator);
        emit CollateralClaimed(validator, amount);
    }

    /// @notice Handles the release of collateral or new deposit of collateral
    function handleCollateral(SubnetActorStorage storage s, address validator, uint256 oldCollateral, uint256 newCollateral) internal {
        if (s.validatorSet.permissionMode != PermissionMode.Collateral) {
            return;
        }

        if (oldCollateral == newCollateral) {
            return;
        }

        if (oldCollateral > newCollateral) {
            uint256 amount = oldCollateral - newCollateral;
            s.releaseQueue.addNewRelease(validator, amount);
            IGateway(s.ipcGatewayAddr).releaseStake(amount);
        } else {
            uint256 amount = newCollateral - oldCollateral;
            address gateway = s.ipcGatewayAddr;

            uint256 msgValue = s.collateralSource.makeAvailable(gateway, amount);
            IGateway(gateway).addStake{value: msgValue}(amount);
        }
    }

    /// @notice Confirm the changes in bottom up checkpoint submission, only call this in bottom up checkpoint execution.
    function confirmChange(uint64 configurationNumber) internal {
        SubnetActorStorage storage s = LibSubnetActorStorage.appStorage();

        if (configurationNumber >= s.changeSet.nextConfigurationNumber) {
            revert CannotConfirmFutureChanges();
        } else if (configurationNumber < s.changeSet.startConfigurationNumber) {
            return;
        }

        uint64 start = s.changeSet.startConfigurationNumber;
        for (uint64 i = start; i <= configurationNumber; ) {
            PowerChange storage change = s.changeSet.getChange(i);

            if (change.op == PowerOperation.SetMetadata) {
                s.validatorSet.validators[change.validator].metadata = change.payload;
            } else if (change.op == PowerOperation.SetPower) {
                (uint256 power) = abi.decode(change.payload, (uint256));
                address validator = change.validator;
                uint256 oldPower = s.validatorSet.confirmPower(validator, power);

                // TODO: Ideally lib power should not be aware of permission mode,
                // TODO: but the current subnet actor design is putting both
                // TODO: collateral and federated power update mode in one contract.
                // TODO: Unless the permission modes are break into different facets,
                // TODO: `handleCollateral` is needed.
                handleCollateral(s, validator, oldPower, power);

            } else {
                revert("Unrecognized power operation");
            }

            s.changeSet.purgeChange(i);
            unchecked {
                ++i;
            }
        }

        s.changeSet.startConfigurationNumber = configurationNumber + 1;
        emit ConfigurationNumberConfirmed(configurationNumber);
    }
}

/// The library for tracking validator changes coming from the parent.
/// Should be used in the child gateway to store changes until they can be applied.
library LibValidatorTracking {
    using LibValidatorSet for ValidatorSet;
    using LibPowerChangeLog for PowerChangeLog;

    function storeChange(ParentValidatorsTracker storage self, PowerChangeRequest calldata changeRequest) internal {
        uint64 configurationNumber = self.changes.recordChange({
            validator: changeRequest.change.validator,
            op: changeRequest.change.op,
            payload: changeRequest.change.payload
        });

        if (configurationNumber != changeRequest.configurationNumber) {
            revert InvalidConfigurationNumber();
        }
    }

    function batchStoreChange(
        ParentValidatorsTracker storage self,
        PowerChangeRequest[] calldata changeRequests
    ) internal {
        uint256 length = changeRequests.length;
        if (length == 0) {
            return;
        }

        for (uint256 i; i < length; ) {
            storeChange(self, changeRequests[i]);
            unchecked {
                ++i;
            }
        }
    }

    /// @notice Confirm the changes in for a finality commitment
    function confirmChange(ParentValidatorsTracker storage self, uint64 configurationNumber) internal {
        if (configurationNumber >= self.changes.nextConfigurationNumber) {
            revert CannotConfirmFutureChanges();
        } else if (configurationNumber < self.changes.startConfigurationNumber) {
            return;
        }

        uint64 start = self.changes.startConfigurationNumber;

        for (uint64 i = start; i <= configurationNumber; ) {
            PowerChange storage change = self.changes.getChange(i);
            address validator = change.validator;

            if (change.op == PowerOperation.SetMetadata) {
                self.validators.validators[validator].metadata = change.payload;
            } else if (change.op == PowerOperation.SetPower) {
                (uint256 power) = abi.decode(change.payload, (uint256));
                self.validators.confirmPower(validator, power);
            } else {
                revert("Unrecognized power operation");
            }

            self.changes.purgeChange(i);
            unchecked {
                ++i;
            }
        }
        self.changes.startConfigurationNumber = configurationNumber + 1;
    }
}
