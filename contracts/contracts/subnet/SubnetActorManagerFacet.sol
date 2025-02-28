// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity ^0.8.23;

import {VALIDATOR_SECP256K1_PUBLIC_KEY_LENGTH} from "../constants/Constants.sol";
import {ERR_VALIDATOR_JOINED, ERR_VALIDATOR_NOT_JOINED} from "../errors/IPCErrors.sol";
import {InvalidFederationPayload, SubnetAlreadyBootstrapped, NotEnoughFunds, CollateralIsZero, CannotReleaseZero, NotOwnerOfPublicKey, EmptyAddress, NotEnoughBalance, NotEnoughCollateral, NotValidator, NotAllValidatorsHaveLeft, InvalidPublicKeyLength, MethodNotAllowed, SubnetNotBootstrapped} from "../errors/IPCErrors.sol";
import {IGateway} from "../interfaces/IGateway.sol";
import {Validator, ValidatorSet, Asset, SubnetID} from "../structs/Subnet.sol";
import {SubnetIDHelper} from "../lib/SubnetIDHelper.sol";
import {LibDiamond} from "../lib/LibDiamond.sol";
import {ReentrancyGuard} from "../lib/LibReentrancyGuard.sol";
import {SubnetActorModifiers} from "../lib/LibSubnetActorStorage.sol";
import {LibValidatorSet, LibPower} from "../lib/LibPower.sol";
import {EnumerableSet} from "@openzeppelin/contracts/utils/structs/EnumerableSet.sol";
import {Address} from "@openzeppelin/contracts/utils/Address.sol";
import {LibSubnetActor} from "../lib/LibSubnetActor.sol";
import {Pausable} from "../lib/LibPausable.sol";
import {AssetHelper} from "../lib/AssetHelper.sol";

contract SubnetActorManagerFacet is SubnetActorModifiers, ReentrancyGuard, Pausable {
    using EnumerableSet for EnumerableSet.AddressSet;
    using SubnetIDHelper for SubnetID;
    using AssetHelper for Asset;
    using LibValidatorSet for ValidatorSet;
    using Address for address payable;

    event ValidatorGaterUpdated(address oldGater, address newGater);
    event NewBootstrapNode(string netAddress, address owner);

    /// @notice method to add some initial balance into a subnet that hasn't yet bootstrapped.
    /// @dev This balance is added to user addresses in genesis, and becomes part of the genesis
    /// circulating supply.
    function preFund(uint256 amount) external payable {
        if (amount == 0) {
            revert NotEnoughFunds();
        }

        if (s.bootstrapped) {
            revert SubnetAlreadyBootstrapped();
        }

        s.supplySource.lock(amount);

        if (s.genesisBalance[msg.sender] == 0) {
            s.genesisBalanceKeys.push(msg.sender);
        }

        s.genesisBalance[msg.sender] += amount;
        s.genesisCircSupply += amount;
    }

    /// @notice method to remove funds from the initial balance of a subnet.
    /// @dev This method can be used by users looking to recover part of their
    /// initial balance before the subnet bootstraps.
    /// @param amount The amount to remove.
    function preRelease(uint256 amount) external nonReentrant {
        if (amount == 0) {
            revert NotEnoughFunds();
        }

        if (s.bootstrapped) {
            revert SubnetAlreadyBootstrapped();
        }

        s.supplySource.transferFunds(payable(msg.sender), amount);

        if (s.genesisBalance[msg.sender] < amount) {
            revert NotEnoughBalance();
        }

        s.genesisBalance[msg.sender] -= amount;
        s.genesisCircSupply -= amount;

        if (s.genesisBalance[msg.sender] == 0) {
            LibSubnetActor.rmAddressFromBalanceKey(msg.sender);
        }
    }

    /// @notice Sets the validator gater contract implementation
    /// @param gater The addresse of validator gater implementation.
    function setValidatorGater(address gater) external notKilled {
        LibDiamond.enforceIsContractOwner();

        emit ValidatorGaterUpdated(s.validatorGater, gater);

        s.validatorGater = gater;
    }

    /// @notice Sets the federated power of validators.
    /// @dev method that allows the contract owner to set the validators' federated power.
    /// @param validators The addresses of validators.
    /// @param publicKeys The public keys of validators.
    /// @param powers The federated powers to be assigned to validators.
    function setFederatedPower(
        address[] calldata validators,
        bytes[] calldata publicKeys,
        uint256[] calldata powers
    ) external notKilled {
        LibDiamond.enforceIsContractOwner();

        LibSubnetActor.enforceFederatedValidation();

        if (validators.length != powers.length) {
            revert InvalidFederationPayload();
        }

        if (validators.length != publicKeys.length) {
            revert InvalidFederationPayload();
        }

        if (s.bootstrapped) {
            LibSubnetActor.postBootstrapSetFederatedPower({
                validators: validators,
                publicKeys: publicKeys,
                powers: powers
            });
        } else {
            LibSubnetActor.preBootstrapSetFederatedPower({
                validators: validators,
                publicKeys: publicKeys,
                powers: powers
            });
        }
    }

    /// @notice method that allows a validator to join the subnet.
    ///         If the total confirmed collateral of the subnet is greater
    ///         or equal to minimum activation collateral as a result of this operation,
    ///         then  subnet will be registered.
    /// @param publicKey The off-chain 65 byte public key that should be associated with the validator
    /// @param amount The amount of collateral provided as stake
    function join(bytes calldata publicKey, uint256 amount) external payable nonReentrant whenNotPaused notKilled {
        // Adding this check to prevent new validators from joining
        // after the subnet has been bootstrapped, if the subnet mode is not Collateral.
        // We will increase the functionality in the future to support explicit permissioning.
        if (s.bootstrapped) {
            LibSubnetActor.enforceCollateralValidation();
        }
        if (amount == 0) {
            revert CollateralIsZero();
        }

        if (LibPower.isValidator(msg.sender)) {
            revert MethodNotAllowed(ERR_VALIDATOR_JOINED);
        }

        if (publicKey.length != VALIDATOR_SECP256K1_PUBLIC_KEY_LENGTH) {
            // Taking 65 bytes because the FVM libraries have some assertions checking it, it's more convenient.
            revert InvalidPublicKeyLength();
        }

        address convertedAddress = LibSubnetActor.publicKeyToAddress(publicKey);
        if (convertedAddress != msg.sender) {
            revert NotOwnerOfPublicKey();
        }

        LibSubnetActor.gateValidatorPowerDelta(msg.sender, 0, amount);

        s.collateralSource.lock(amount);

        if (!s.bootstrapped) {
            // if the subnet has not been bootstrapped, join directly
            // without delays, and collect collateral to register
            // in the gateway

            // confirm validators deposit immediately
            LibPower.setMetadataWithConfirm(msg.sender, publicKey);
            LibPower.depositWithConfirm(msg.sender, amount);

            _patchGenesisValidators(msg.sender);

            LibSubnetActor.bootstrapSubnetIfNeeded();
        } else {
            // if the subnet has been bootstrapped, join with postponed confirmation.
            LibPower.setValidatorMetadata(msg.sender, publicKey);
            LibPower.deposit(msg.sender, amount);
        }
    }

    function _patchGenesisValidators(address validator) internal {
        // add to initial validators avoiding duplicates if it
        // is a genesis validator.
        bool alreadyValidator;
        uint256 length = s.genesisValidators.length;
        for (uint256 i; i < length; ) {
            if (s.genesisValidators[i].addr == validator) {
                alreadyValidator = true;
                break;
            }
            unchecked {
                ++i;
            }
        }
        if (!alreadyValidator) {
            uint256 collateral = s.validatorSet.validators[validator].currentPower;
            Validator memory val = Validator({
                addr: validator,
                weight: collateral,
                metadata: s.validatorSet.validators[validator].metadata
            });
            s.genesisValidators.push(val);
        }
    }

    /// @notice method that allows a validator to increase its stake.
    ///         If the total confirmed collateral of the subnet is greater
    ///         or equal to minimum activation collateral as a result of this operation,
    ///         then  subnet will be registered.
    /// @param amount The amount of collateral provided as stake
    function stake(uint256 amount) external payable whenNotPaused notKilled {
        // disabling validator changes for federated subnets (at least for now
        // until a more complex mechanism is implemented).
        LibSubnetActor.enforceCollateralValidation();
        if (amount == 0) {
            revert CollateralIsZero();
        }

        if (!LibPower.isValidator(msg.sender)) {
            revert MethodNotAllowed(ERR_VALIDATOR_NOT_JOINED);
        }

        uint256 collateral = LibPower.totalValidatorCollateral(msg.sender);
        LibSubnetActor.gateValidatorPowerDelta(msg.sender, collateral, collateral + amount);

        s.collateralSource.lock(amount);

        if (!s.bootstrapped) {
            LibPower.depositWithConfirm(msg.sender, amount);
            _patchGenesisValidators(msg.sender);
            LibSubnetActor.bootstrapSubnetIfNeeded();
        } else {
            LibPower.deposit(msg.sender, amount);
        }
    }

    /// @notice method that allows a validator to unstake a part of its collateral from a subnet.
    /// @dev `leave` must be used to unstake the entire stake.
    /// @param amount The amount to unstake.
    function unstake(uint256 amount) external nonReentrant whenNotPaused notKilled {
        // disabling validator changes for federated validation subnets (at least for now
        // until a more complex mechanism is implemented).
        LibSubnetActor.enforceCollateralValidation();

        if (amount == 0) {
            revert CannotReleaseZero();
        }

        uint256 collateral = LibPower.totalValidatorCollateral(msg.sender);

        if (collateral == 0) {
            revert NotValidator(msg.sender);
        }
        if (collateral <= amount) {
            revert NotEnoughCollateral();
        }

        LibSubnetActor.gateValidatorPowerDelta(msg.sender, collateral, collateral - amount);

        if (!s.bootstrapped) {
            LibPower.withdrawWithConfirm(msg.sender, amount);
            s.collateralSource.transferFunds(payable(msg.sender), amount);
        } else {
            LibPower.withdraw(msg.sender, amount);
        }
    }

    /// @notice method that allows a validator to leave the subnet.
    function leave() external nonReentrant whenNotPaused notKilled {
        // disabling validator changes for federated subnets (at least for now
        // until a more complex mechanism is implemented).
        // This means that initial validators won't be able to recover
        // their collateral ever (worth noting in the docs if this ends
        // up sticking around for a while).
        if (s.bootstrapped) {
            LibSubnetActor.enforceCollateralValidation();
        }

        // remove bootstrap nodes added by this validator
        uint256 amount = LibPower.totalValidatorCollateral(msg.sender);
        if (amount == 0) {
            revert NotValidator(msg.sender);
        }

        LibSubnetActor.gateValidatorPowerDelta(msg.sender, amount, 0);

        // slither-disable-next-line unused-return
        s.bootstrapOwners.remove(msg.sender);
        delete s.bootstrapNodes[msg.sender];

        if (!s.bootstrapped) {
            // check if the validator had some initial balance and return it if not bootstrapped
            uint256 genesisBalance = s.genesisBalance[msg.sender];
            if (genesisBalance != 0) {
                delete s.genesisBalance[msg.sender];
                s.genesisCircSupply -= genesisBalance;
                LibSubnetActor.rmAddressFromBalanceKey(msg.sender);
                s.collateralSource.transferFunds(payable(msg.sender), genesisBalance);
            }

            // interaction must be performed after checks and changes
            LibPower.withdrawWithConfirm(msg.sender, amount);
            s.collateralSource.transferFunds(payable(msg.sender), amount);
            return;
        }
        LibPower.withdraw(msg.sender, amount);
    }

    /// @notice method that allows to kill the subnet when all validators left.
    /// @dev It is not a privileged operation.
    function kill() external notKilled {
        if (LibPower.totalValidators() != 0) {
            revert NotAllValidatorsHaveLeft();
        }
        if (!s.bootstrapped) {
            revert SubnetNotBootstrapped();
        }
        s.killed = true;
        IGateway(s.ipcGatewayAddr).kill();
    }

    /// @notice Add a bootstrap node.
    /// @param netAddress The network address of the new bootstrap node.
    function addBootstrapNode(string calldata netAddress) external whenNotPaused {
        if (!s.validatorSet.isActiveValidator(msg.sender)) {
            revert NotValidator(msg.sender);
        }
        if (bytes(netAddress).length == 0) {
            revert EmptyAddress();
        }
        s.bootstrapNodes[msg.sender] = netAddress;
        // slither-disable-next-line unused-return
        s.bootstrapOwners.add(msg.sender);

        emit NewBootstrapNode(netAddress, msg.sender);
    }
}
