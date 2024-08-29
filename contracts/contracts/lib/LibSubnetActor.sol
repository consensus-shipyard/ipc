// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity ^0.8.23;

import {VALIDATOR_SECP256K1_PUBLIC_KEY_LENGTH} from "../constants/Constants.sol";
import {ERR_PERMISSIONED_AND_BOOTSTRAPPED} from "../errors/IPCErrors.sol";
import {NotEnoughGenesisValidators, DuplicatedGenesisValidator, NotOwnerOfPublicKey, MethodNotAllowed} from "../errors/IPCErrors.sol";
import {IGateway} from "../interfaces/IGateway.sol";
import {IValidatorGater} from "../interfaces/IValidatorGater.sol";
import {Validator, ValidatorSet, PermissionMode, SubnetID} from "../structs/Subnet.sol";
import {SubnetActorModifiers} from "../lib/LibSubnetActorStorage.sol";
import {LibValidatorSet, LibStaking} from "../lib/LibStaking.sol";
import {EnumerableSet} from "@openzeppelin/contracts/utils/structs/EnumerableSet.sol";
import {LibSubnetActorStorage, SubnetActorStorage} from "./LibSubnetActorStorage.sol";
import {SubnetIDHelper} from "../lib/SubnetIDHelper.sol";

library LibSubnetActor {
    using EnumerableSet for EnumerableSet.AddressSet;
    using SubnetIDHelper for SubnetID;

    event SubnetBootstrapped(Validator[]);

    /// @notice Ensures that the subnet is operating under Collateral-based permission mode.
    /// @dev Reverts if the subnet is not in Collateral mode.
    function enforceCollateralValidation() internal view {
        SubnetActorStorage storage s = LibSubnetActorStorage.appStorage();

        if (s.validatorSet.permissionMode != PermissionMode.Collateral) {
            revert MethodNotAllowed(ERR_PERMISSIONED_AND_BOOTSTRAPPED);
        }
        return;
    }

    /// @notice Ensures that the subnet is operating under Federated permission mode.
    /// @dev Reverts if the subnet is not in Federated mode.
    function enforceFederatedValidation() internal view {
        SubnetActorStorage storage s = LibSubnetActorStorage.appStorage();

        if (s.validatorSet.permissionMode != PermissionMode.Federated) {
            revert MethodNotAllowed(ERR_PERMISSIONED_AND_BOOTSTRAPPED);
        }
        return;
    }

    /// @notice Performs validator gating, i.e. checks if the validator power update is actually allowed.
    function validatorGating(address validator, uint256 powerDelta, bool isIncrease) internal {
        SubnetActorStorage storage s = LibSubnetActorStorage.appStorage();

        // zero address means no gating needed
        if (s.validatorGater == address(0)) {
            return;
        }

        uint256 oldPower = LibStaking.getPower(validator);
        uint256 newPower = 0;
        if (isIncrease) {
            newPower = oldPower + powerDelta;
        } else {
            newPower = oldPower - powerDelta;
        }

        SubnetID memory id = s.parentId.createSubnetId(address(this));
        IValidatorGater(s.validatorGater).interceptPowerDelta(id, validator, oldPower, newPower);
    }

    /// @notice Performs validator gating, i.e. checks if the validator power update is actually allowed.
    function validatorGating(SubnetID memory id, address validator, uint256 prevPower, uint256 newPower) internal {
        SubnetActorStorage storage s = LibSubnetActorStorage.appStorage();

        // zero address means no gating needed
        if (s.validatorGater == address(0)) {
            return;
        }

        IValidatorGater(s.validatorGater).interceptPowerDelta(id, validator, prevPower, newPower);
    }

    /// @dev This function is used to bootstrap the subnet,
    ///     if its total collateral is greater than minimum activation collateral.
    function bootstrapSubnetIfNeeded() internal {
        SubnetActorStorage storage s = LibSubnetActorStorage.appStorage();

        uint256 totalCollateral = LibStaking.getTotalConfirmedCollateral();

        if (totalCollateral >= s.minActivationCollateral) {
            if (LibStaking.totalActiveValidators() >= s.minValidators) {
                s.bootstrapped = true;
                emit SubnetBootstrapped(s.genesisValidators);

                // register adding the genesis circulating supply (if it exists)
                IGateway(s.ipcGatewayAddr).register{value: totalCollateral + s.genesisCircSupply}(s.genesisCircSupply);
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

    /// @notice method that allows the contract owner to set the validators' federated power before.
    /// @notice subnet has already been bootstrapped.
    /// @param validators The list of validators' addresses.
    /// @param publicKeys The list of validators' public keys.
    /// @param powers The list of power values of the validators.
    function preBootstrapSetFederatedPower(
        address[] calldata validators,
        bytes[] calldata publicKeys,
        uint256[] calldata powers
    ) internal {
        SubnetActorStorage storage s = LibSubnetActorStorage.appStorage();
        SubnetID memory subnet = s.parentId.createSubnetId(address(this));

        uint256 length = validators.length;

        if (length <= s.minValidators) {
            revert NotEnoughGenesisValidators();
        }

        for (uint256 i; i < length; ) {
            // check addresses
            address convertedAddress = publicKeyToAddress(publicKeys[i]);
            if (convertedAddress != validators[i]) {
                revert NotOwnerOfPublicKey();
            }

            // performing deduplication
            // validator should have no power when first added
            if (LibStaking.getPower(validators[i]) > 0) {
                revert DuplicatedGenesisValidator();
            }

            LibSubnetActor.validatorGating(subnet, validators[i], 0, powers[i]);
            LibStaking.setMetadataWithConfirm(validators[i], publicKeys[i]);
            LibStaking.setFederatedPowerWithConfirm(validators[i], powers[i]);

            s.genesisValidators.push(Validator({addr: validators[i], weight: powers[i], metadata: publicKeys[i]}));

            unchecked {
                ++i;
            }
        }

        s.bootstrapped = true;
        emit SubnetBootstrapped(s.genesisValidators);

        // register adding the genesis circulating supply (if it exists)
        IGateway(s.ipcGatewayAddr).register{value: s.genesisCircSupply}(s.genesisCircSupply);
    }

    /// @notice method that allows the contract owner to set the validators' federated power after
    /// @dev subnet has already been bootstrapped.
    /// @param validators The list of validators' addresses.
    /// @param publicKeys The list of validators' public keys.
    /// @param powers The list of power values of the validators.
    function postBootstrapSetFederatedPower(
        address[] calldata validators,
        bytes[] calldata publicKeys,
        uint256[] calldata powers
    ) internal {
        uint256 length = validators.length;

        SubnetActorStorage storage s = LibSubnetActorStorage.appStorage();
        SubnetID memory subnet = s.parentId.createSubnetId(address(this));

        for (uint256 i; i < length; ) {
            // check addresses
            address convertedAddress = publicKeyToAddress(publicKeys[i]);
            if (convertedAddress != validators[i]) {
                revert NotOwnerOfPublicKey();
            }

            LibSubnetActor.validatorGating(
                subnet,
                validators[i],
                LibStaking.getPower(validators[i]),
                powers[i]
            );

            // no need to do deduplication as set directly set the power, there wont be any addition of
            // federated power.
            LibStaking.setFederatedPower({validator: validators[i], metadata: publicKeys[i], amount: powers[i]});

            unchecked {
                ++i;
            }
        }
    }

    /// @notice Removes an address from the initial balance keys.
    /// @param addr The address to be removed from the genesis balance keys.
    function rmAddressFromBalanceKey(address addr) internal {
        SubnetActorStorage storage s = LibSubnetActorStorage.appStorage();

        uint256 length = s.genesisBalanceKeys.length;
        for (uint256 i; i < length; ) {
            if (s.genesisBalanceKeys[i] == addr) {
                s.genesisBalanceKeys[i] = s.genesisBalanceKeys[length - 1];
                s.genesisBalanceKeys.pop();
                // exit after removing the key
                break;
            }
            unchecked {
                ++i;
            }
        }
    }
}