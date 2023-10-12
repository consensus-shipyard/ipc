// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity 0.8.19;

import {IGateway} from "../interfaces/IGateway.sol";
import {LibSubnetActorStorage, SubnetActorStorage} from "./LibSubnetActorStorage.sol";
import {GatewayActorStorage, LibGatewayActorStorage} from "../lib/LibGatewayActorStorage.sol";
import {LibMaxPQ, MaxPQ} from "./priority/LibMaxPQ.sol";
import {LibMinPQ, MinPQ} from "./priority/LibMinPQ.sol";
import {StakingReleaseQueue, StakingChangeLog, StakingChange, StakingChangeRequest, StakingOperation, StakingRelease, ValidatorSet, AddressStakingReleases, ParentValidatorsTracker, Validator} from "../structs/Subnet.sol";
import {NoRewardToWithdraw, WithdrawExceedingCollateral, NotValidator, CannotConfirmFutureChanges, NoCollateralToWithdraw, AddressShouldBeValidator, InvalidConfigurationNumber} from "../errors/IPCErrors.sol";
import {Address} from "openzeppelin-contracts/utils/Address.sol";

/// The util library for `StakingChangeLog`
library LibStakingChangeLog {
    event NewStakingChangeRequest(StakingOperation op, address validator, bytes payload, uint64 configurationNumber);

    /// @notice Validator request to update its metadata
    function metadataRequest(StakingChangeLog storage changes, address validator, bytes calldata metadata) internal {
        uint64 configurationNumber = recordChange({
            changes: changes,
            validator: validator,
            op: StakingOperation.SetMetadata,
            payload: metadata
        });

        emit NewStakingChangeRequest({
            op: StakingOperation.SetMetadata,
            validator: validator,
            payload: metadata,
            configurationNumber: configurationNumber
        });
    }

    /// @notice Perform upsert operation to the withdraw changes, return total value to withdraw
    /// @notice of the validator.
    /// Each insert will increment the configuration number by 1, update will not.
    function withdrawRequest(StakingChangeLog storage changes, address validator, uint256 amount) internal {
        bytes memory payload = abi.encode(amount);

        uint64 configurationNumber = recordChange({
            changes: changes,
            validator: validator,
            op: StakingOperation.Withdraw,
            payload: payload
        });

        emit NewStakingChangeRequest({
            op: StakingOperation.Withdraw,
            validator: validator,
            payload: payload,
            configurationNumber: configurationNumber
        });
    }

    /// @notice Perform upsert operation to the deposit changes
    function depositRequest(StakingChangeLog storage changes, address validator, uint256 amount) internal {
        bytes memory payload = abi.encode(amount);

        uint64 configurationNumber = recordChange({
            changes: changes,
            validator: validator,
            op: StakingOperation.Deposit,
            payload: payload
        });

        emit NewStakingChangeRequest({
            op: StakingOperation.Deposit,
            validator: validator,
            payload: payload,
            configurationNumber: configurationNumber
        });
    }

    /// @notice Perform upsert operation to the deposit changes
    function recordChange(
        StakingChangeLog storage changes,
        address validator,
        StakingOperation op,
        bytes memory payload
    ) internal returns (uint64 configurationNumber) {
        configurationNumber = changes.nextConfigurationNumber;

        changes.changes[configurationNumber] = StakingChange({op: op, validator: validator, payload: payload});

        changes.nextConfigurationNumber = configurationNumber + 1;
    }

    /// @notice Get the change at configuration number
    function getChange(
        StakingChangeLog storage changes,
        uint64 configurationNumber
    ) internal view returns (StakingChange storage) {
        return changes.changes[configurationNumber];
    }

    function purgeChange(StakingChangeLog storage changes, uint64 configurationNumber) internal {
        delete changes.changes[configurationNumber];
    }
}

library LibAddressStakingReleases {
    /// @notice Add new release to the storage. Caller makes sure the release.releasedAt is ordered
    /// @notice in ascending order. This method does not do checks on this.
    function push(AddressStakingReleases storage self, StakingRelease memory release) internal {
        uint16 length = self.length;
        uint16 nextIdx = self.startIdx + length;

        self.releases[nextIdx] = release;
        self.length = length + 1;
    }

    /// @notice Perform compaction on releases, i.e. aggregates the amount that can be released
    /// @notice and removes them from storage. Returns the total amount to release and the new
    /// @notice number of pending releases after compaction.
    function compact(AddressStakingReleases storage self) internal returns (uint256, uint16) {
        uint16 length = self.length;
        if (self.length == 0) {
            revert NoCollateralToWithdraw();
        }

        uint16 i = self.startIdx;
        uint16 newLength = length;
        uint256 amount = 0;
        while (i < length) {
            StakingRelease memory release = self.releases[i];

            // releases are ordered ascending by releaseAt, no need to check
            // further as they will still be locked.
            if (release.releaseAt > block.number) {
                break;
            }

            amount += release.amount;
            delete self.releases[i];

            unchecked {
                ++i;
                --newLength;
            }
        }

        self.startIdx = i;
        self.length = newLength;

        return (amount, newLength);
    }
}

/// The util library for `StakingReleaseQueue`
library LibStakingReleaseQueue {
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

        SubnetActorStorage storage s = LibSubnetActorStorage.appStorage();
        IGateway(s.ipcGatewayAddr).releaseStake(amount);
        payable(validator).transfer(amount);

        return amount;
    }
}

/// The util library for `ValidatorSet`
library LibValidatorSet {
    using LibMinPQ for MinPQ;
    using LibMaxPQ for MaxPQ;

    event ActiveValidatorCollateralUpdated(address validator, uint256 newCollateral);
    event WaitingValidatorCollateralUpdated(address validator, uint256 newCollateral);
    event NewActiveValidator(address validator, uint256 collateral);
    event NewWaitingValidator(address validator, uint256 collateral);
    event ActiveValidatorReplaced(address oldValidator, address newValidator);
    event ActiveValidatorLeft(address validator);
    event WaitingValidatorLeft(address validator);

    /// @notice Get the total confirmed collateral of the validators.
    function getTotalConfirmedCollateral(ValidatorSet storage validators) internal view returns (uint256 collateral) {
        collateral = validators.totalConfirmedCollateral;
    }

    /// @notice Get the total active validators.
    function totalActiveValidators(ValidatorSet storage validators) internal view returns (uint16 total) {
        total = validators.activeValidators.getSize();
    }

    /// @notice Get the confirmed collateral of the validator.
    function getConfirmedCollateral(
        ValidatorSet storage validators,
        address validator
    ) internal view returns (uint256 collateral) {
        collateral = validators.validators[validator].confirmedCollateral;
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

    /// @notice Get the confirmed collaterals of the validators.
    /// The function reverts if at least one validator is not in the active validator set.
    function getConfirmedCollaterals(
        ValidatorSet storage validators,
        address[] memory addresses
    ) internal view returns (uint256[] memory) {
        uint256 size = addresses.length;
        uint256[] memory activeCollaterals = new uint256[](size);

        for (uint256 i = 0; i < size; ) {
            if (!isActiveValidator(validators, addresses[i])) {
                revert NotValidator();
            }
            activeCollaterals[i] = validators.validators[addresses[i]].confirmedCollateral;
            unchecked {
                ++i;
            }
        }
        return activeCollaterals;
    }

    function isActiveValidator(ValidatorSet storage self, address validator) internal view returns (bool) {
        return self.activeValidators.contains(validator);
    }

    /// @notice Set validator data
    function setMetadata(ValidatorSet storage validators, address validator, bytes calldata metadata) internal {
        validators.validators[validator].metadata = metadata;
    }

    /***********************************************************************
     * Internal helper functions, should not be called by external functions
     ***********************************************************************/

    /// @notice Validator increases its total collateral by amount.
    function recordDeposit(ValidatorSet storage validators, address validator, uint256 amount) internal {
        validators.validators[validator].totalCollateral += amount;
    }

    /// @notice Validator reduces its total collateral by amount.
    function recordWithdraw(ValidatorSet storage validators, address validator, uint256 amount) internal {
        uint256 total = validators.validators[validator].totalCollateral;
        if (total < amount) {
            revert WithdrawExceedingCollateral();
        }

        total -= amount;
        validators.validators[validator].totalCollateral = total;
    }

    function confirmDeposit(ValidatorSet storage self, address validator, uint256 amount) internal {
        uint256 newCollateral = self.validators[validator].confirmedCollateral + amount;
        self.validators[validator].confirmedCollateral = newCollateral;

        self.totalConfirmedCollateral += amount;

        depositReshuffle({self: self, maybeActive: validator, newCollateral: newCollateral});
    }

    function confirmWithdraw(ValidatorSet storage self, address validator, uint256 amount) internal {
        uint256 newCollateral = self.validators[validator].confirmedCollateral - amount;

        if (newCollateral == 0) {
            delete self.validators[validator];
        } else {
            self.validators[validator].confirmedCollateral = newCollateral;
        }

        withdrawReshuffle({self: self, validator: validator, newCollateral: newCollateral});

        self.totalConfirmedCollateral -= amount;
    }

    /// @notice Reshuffles the active and waiting validators when a deposit is confirmed
    function depositReshuffle(ValidatorSet storage self, address maybeActive, uint256 newCollateral) internal {
        if (self.activeValidators.contains(maybeActive)) {
            self.activeValidators.increaseReheapify(self, maybeActive);
            emit ActiveValidatorCollateralUpdated(maybeActive, newCollateral);
            return;
        }

        // incoming address is not active validator
        uint16 activeLimit = self.activeLimit;
        uint16 activeSize = self.activeValidators.getSize();
        if (activeLimit > activeSize) {
            // we can still take more active validators, just insert to the pq.
            self.activeValidators.insert(self, maybeActive);
            emit NewActiveValidator(maybeActive, newCollateral);
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
        (address minAddress, uint256 minActiveCollateral) = self.activeValidators.min(self);
        if (minActiveCollateral < newCollateral) {
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
            emit WaitingValidatorCollateralUpdated(maybeActive, newCollateral);
            return;
        }

        self.waitingValidators.insert(self, maybeActive);
        emit NewWaitingValidator(maybeActive, newCollateral);
    }

    /// @notice Reshuffles the active and waiting validators when a withdraw is confirmed
    function withdrawReshuffle(ValidatorSet storage self, address validator, uint256 newCollateral) internal {
        if (self.waitingValidators.contains(validator)) {
            if (newCollateral == 0) {
                self.waitingValidators.deleteReheapify(self, validator);
                emit WaitingValidatorLeft(validator);
                return;
            }
            self.waitingValidators.decreaseReheapify(self, validator);
            emit WaitingValidatorCollateralUpdated(validator, newCollateral);
            return;
        }

        // sanity check
        if (!self.activeValidators.contains(validator)) {
            revert AddressShouldBeValidator();
        }

        // the validator is an active validator!

        if (newCollateral == 0) {
            self.activeValidators.deleteReheapify(self, validator);
            emit ActiveValidatorLeft(validator);

            if (self.waitingValidators.getSize() != 0) {
                (address toBePromoted, uint256 collateral) = self.waitingValidators.max(self);
                self.waitingValidators.pop(self);
                self.activeValidators.insert(self, toBePromoted);
                emit NewActiveValidator(toBePromoted, collateral);
            }

            return;
        }

        self.activeValidators.decreaseReheapify(self, validator);

        if (self.waitingValidators.getSize() == 0) {
            return;
        }

        (address mayBeDemoted, uint256 minActiveCollateral) = self.activeValidators.min(self);
        (address mayBePromoted, uint256 maxWaitingCollateral) = self.waitingValidators.max(self);
        if (minActiveCollateral < maxWaitingCollateral) {
            self.activeValidators.pop(self);
            self.waitingValidators.pop(self);
            self.activeValidators.insert(self, mayBePromoted);
            self.waitingValidators.insert(self, mayBeDemoted);

            emit ActiveValidatorReplaced(mayBeDemoted, mayBePromoted);
            return;
        }

        emit ActiveValidatorCollateralUpdated(validator, newCollateral);
    }
}

library LibStaking {
    using LibStakingReleaseQueue for StakingReleaseQueue;
    using LibStakingChangeLog for StakingChangeLog;
    using LibValidatorSet for ValidatorSet;
    using LibMaxPQ for MaxPQ;
    using LibMinPQ for MinPQ;
    using Address for address payable;

    uint64 public constant INITIAL_CONFIGURATION_NUMBER = 1;

    event ConfigurantionNumberConfirmed(uint64 number);
    event CollateralClaimed(address validator, uint256 amount);

    // =============== Getters =============

    /// @notice Checks if the validator is an active validator
    function isActiveValidator(address validator) internal view returns (bool) {
        SubnetActorStorage storage s = LibSubnetActorStorage.appStorage();
        return s.validatorSet.activeValidators.contains(validator);
    }

    /// @notice Checks if the validator is a waiting validator
    function isWaitingValidator(address validator) internal view returns (bool) {
        SubnetActorStorage storage s = LibSubnetActorStorage.appStorage();
        return s.validatorSet.waitingValidators.contains(validator);
    }

    /// @notice Checks if the validator has staked before
    function hasStaked(address validator) internal view returns (bool) {
        SubnetActorStorage storage s = LibSubnetActorStorage.appStorage();
        return s.validatorSet.validators[validator].totalCollateral > 0;
    }

    function totalActiveValidators() internal view returns (uint16) {
        SubnetActorStorage storage s = LibSubnetActorStorage.appStorage();
        return s.validatorSet.totalActiveValidators();
    }

    /// @notice Gets the total number of validators, including active and waiting
    function totalValidators() internal view returns (uint16) {
        SubnetActorStorage storage s = LibSubnetActorStorage.appStorage();
        return s.validatorSet.waitingValidators.getSize() + s.validatorSet.activeValidators.getSize();
    }

    function getTotalConfirmedCollateral() internal view returns (uint256) {
        SubnetActorStorage storage s = LibSubnetActorStorage.appStorage();
        return s.validatorSet.getTotalConfirmedCollateral();
    }

    /// @notice Gets the total collateral the validators has staked.
    function totalValidatorCollateral(address validator) internal view returns (uint256) {
        SubnetActorStorage storage s = LibSubnetActorStorage.appStorage();
        return s.validatorSet.validators[validator].totalCollateral;
    }

    // =============== Operations directly confirm =============

    /// @notice Set the validator metadata directly without queueing the request
    function setMetadataWithConfirm(address validator, bytes calldata metadata) internal {
        SubnetActorStorage storage s = LibSubnetActorStorage.appStorage();
        s.validatorSet.setMetadata(validator, metadata);
    }

    /// @notice Confirm the deposit directly without going through the confirmation process
    function depositWithConfirm(address validator, uint256 amount) internal {
        SubnetActorStorage storage s = LibSubnetActorStorage.appStorage();

        // record deposit that updates the total collateral
        s.validatorSet.recordDeposit(validator, amount);
        // confirm deposit that updates the confirmed collateral
        s.validatorSet.confirmDeposit(validator, amount);

        if (!s.bootstrapped) {
            // add to initial validators avoiding duplicates if it
            // is a genesis validator.
            bool alreadyValidator = false;
            uint256 length = s.genesisValidators.length;
            for (uint256 i = 0; i < length; ) {
                if (s.genesisValidators[i].addr == validator) {
                    alreadyValidator = true;
                    break;
                }
                unchecked {
                    ++i;
                }
            }
            if (!alreadyValidator) {
                uint256 collateral = s.validatorSet.validators[validator].confirmedCollateral;
                Validator memory val = Validator({
                    addr: validator,
                    weight: collateral,
                    metadata: s.validatorSet.validators[validator].metadata
                });
                s.genesisValidators.push(val);
            }
        }
    }

    /// @notice Confirm the withdraw directly without going through the confirmation process
    function withdrawWithConfirm(address validator, uint256 amount) internal {
        SubnetActorStorage storage s = LibSubnetActorStorage.appStorage();

        // record deposit that updates the total collateral
        s.validatorSet.recordWithdraw(validator, amount);
        // confirm deposit that updates the confirmed collateral
        s.validatorSet.confirmWithdraw(validator, amount);

        // release stake from gateway and transfer to user
        IGateway(s.ipcGatewayAddr).releaseStake(amount);
        payable(validator).transfer(amount);
    }

    // ================= Operations that are queued ==============

    /// @notice Set the validator metadata
    function setValidatorMetadata(address validator, bytes calldata metadata) internal {
        SubnetActorStorage storage s = LibSubnetActorStorage.appStorage();
        s.changeSet.metadataRequest(validator, metadata);
    }

    /// @notice Deposit the collateral
    function deposit(address validator, uint256 amount) internal {
        SubnetActorStorage storage s = LibSubnetActorStorage.appStorage();

        s.changeSet.depositRequest(validator, amount);
        s.validatorSet.recordDeposit(validator, amount);
    }

    /// @notice Withdraw the collateral
    function withdraw(address validator, uint256 amount) internal {
        SubnetActorStorage storage s = LibSubnetActorStorage.appStorage();

        s.changeSet.withdrawRequest(validator, amount);
        s.validatorSet.recordWithdraw(validator, amount);
    }

    // =============== Other functions ================

    /// @notice Claim the released collateral
    function claimCollateral(address validator) internal {
        SubnetActorStorage storage s = LibSubnetActorStorage.appStorage();
        uint256 amount = s.releaseQueue.claim(validator);
        emit CollateralClaimed(validator, amount);
    }

    /// @notice method that allows a relayer to withdraw it's accumulated rewards using pull-based transfer
    function claimRewardForRelayer(address relayer) external {
        SubnetActorStorage storage s = LibSubnetActorStorage.appStorage();
        uint256 amount = s.relayerRewards[relayer];

        if (amount == 0) {
            revert NoRewardToWithdraw();
        }

        s.relayerRewards[relayer] = 0;
        IGateway(s.ipcGatewayAddr).releaseRewardForRelayer(amount);

        payable(relayer).sendValue(amount);
    }

    /// @notice Confirm the changes in bottom up checkpoint submission, only call this in bottom up checkpoint execution.
    function confirmChange(uint64 configurationNumber) internal {
        SubnetActorStorage storage s = LibSubnetActorStorage.appStorage();
        StakingChangeLog storage changeSet = s.changeSet;

        if (configurationNumber >= changeSet.nextConfigurationNumber) {
            revert CannotConfirmFutureChanges();
        }

        uint64 start = changeSet.startConfigurationNumber;
        for (uint64 i = start; i <= configurationNumber; ) {
            StakingChange storage change = changeSet.getChange(i);
            address validator = change.validator;

            if (change.op == StakingOperation.SetMetadata) {
                s.validatorSet.validators[validator].metadata = change.payload;
            } else {
                uint256 amount = abi.decode(change.payload, (uint256));

                if (change.op == StakingOperation.Withdraw) {
                    s.validatorSet.confirmWithdraw(validator, amount);
                    s.releaseQueue.addNewRelease(validator, amount);
                } else {
                    s.validatorSet.confirmDeposit(validator, amount);
                    IGateway(s.ipcGatewayAddr).addStake{value: amount}();
                }
            }

            changeSet.purgeChange(i);
            unchecked {
                ++i;
            }
        }

        changeSet.startConfigurationNumber = configurationNumber + 1;

        emit ConfigurantionNumberConfirmed(configurationNumber);
    }
}

/// The library for tracking validator changes coming from the parent.
/// Should be used in the child gateway to store changes until they can be applied.
library LibValidatorTracking {
    using LibValidatorSet for ValidatorSet;
    using LibStakingChangeLog for StakingChangeLog;

    function storeChange(ParentValidatorsTracker storage self, StakingChangeRequest calldata changeRequest) internal {
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
        StakingChangeRequest[] calldata changeRequests
    ) internal {
        uint256 length = changeRequests.length;
        if (length == 0) {
            return;
        }

        for (uint256 i = 0; i < length; ) {
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
        }

        uint64 start = self.changes.startConfigurationNumber;

        for (uint64 i = start; i <= configurationNumber; ) {
            StakingChange storage change = self.changes.getChange(i);
            address validator = change.validator;

            if (change.op == StakingOperation.SetMetadata) {
                self.validators.validators[validator].metadata = change.payload;
            } else {
                uint256 amount = abi.decode(change.payload, (uint256));

                if (change.op == StakingOperation.Withdraw) {
                    self.validators.confirmWithdraw(validator, amount);
                } else {
                    self.validators.confirmDeposit(validator, amount);
                }
            }

            self.changes.purgeChange(i);
            unchecked {
                ++i;
            }
        }
        self.changes.startConfigurationNumber = configurationNumber + 1;
    }
}
