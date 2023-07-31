// SPDX-License-Identifier: UNLICENSED
pragma solidity 0.8.19;

import {GatewayActorModifiers} from "../lib/LibGatewayActorStorage.sol";
import {CrossMsg} from "../structs/Checkpoint.sol";
import {Status} from "../enums/Status.sol";
import {FvmAddress} from "../structs/FvmAddress.sol";
import {SubnetID, Subnet} from "../structs/Subnet.sol";
import {AlreadyInitialized, AlreadyRegisteredSubnet, CannotReleaseZero, NotEnoughFunds, NotEnoughFundsToRelease, NotEmptySubnetCircSupply, NotRegisteredSubnet, ValidatorsAndWeightsLengthMismatch, ValidatorWeightIsZero} from "../errors/IPCErrors.sol";
import {LibGateway} from "../lib/LibGateway.sol";
import {FvmAddressHelper} from "../lib/FvmAddressHelper.sol";
import {SubnetIDHelper} from "../lib/SubnetIDHelper.sol";
import {CrossMsgHelper} from "../lib/CrossMsgHelper.sol";
import {FilAddress} from "fevmate/utils/FilAddress.sol";
import {ReentrancyGuard} from "../lib/LibReentrancyGuard.sol";
import {LibVoting} from "../lib/LibVoting.sol";

contract GatewayManagerFacet is GatewayActorModifiers, ReentrancyGuard {
    using FilAddress for address payable;
    using SubnetIDHelper for SubnetID;
    using FvmAddressHelper for FvmAddress;

    /// @notice initialize the contract with the genesis epoch
    /// @param genesisEpoch - genesis epoch to set
    function initGenesisEpoch(uint64 genesisEpoch) external systemActorOnly {
        if (s.initialized) {
            revert AlreadyInitialized();
        }

        LibVoting.initGenesisEpoch(genesisEpoch);
        s.initialized = true;
    }

    /// @notice register a subnet in the gateway. It is called by a subnet when it reaches the threshold stake
    function register() external payable {
        if (msg.value < s.minStake) {
            revert NotEnoughFunds();
        }

        SubnetID memory subnetId = s.networkName.createSubnetId(msg.sender);

        (bool registered, Subnet storage subnet) = LibGateway.getSubnet(subnetId);

        if (registered) {
            revert AlreadyRegisteredSubnet();
        }

        subnet.id = subnetId;
        subnet.stake = msg.value;
        subnet.status = Status.Active;
        subnet.genesisEpoch = block.number;

        s.subnetKeys.push(subnetId.toHash());

        s.totalSubnets += 1;
    }

    /// @notice addStake - add collateral for an existing subnet
    function addStake() external payable {
        if (msg.value <= 0) {
            revert NotEnoughFunds();
        }

        (bool registered, Subnet storage subnet) = LibGateway.getSubnet(msg.sender);

        if (!registered) {
            revert NotRegisteredSubnet();
        }

        subnet.stake += msg.value;

        if (subnet.status == Status.Inactive) {
            if (subnet.stake >= s.minStake) {
                subnet.status = Status.Active;
            }
        }
    }

    /// @notice release collateral for an existing subnet
    function releaseStake(uint256 amount) external nonReentrant {
        if (amount == 0) {
            revert CannotReleaseZero();
        }

        (bool registered, Subnet storage subnet) = LibGateway.getSubnet(msg.sender);

        if (!registered) {
            revert NotRegisteredSubnet();
        }
        if (subnet.stake < amount) {
            revert NotEnoughFundsToRelease();
        }

        subnet.stake -= amount;

        if (subnet.stake < s.minStake) {
            subnet.status = Status.Inactive;
        }

        payable(subnet.id.getActor()).sendValue(amount);
    }

    function releaseRewards(uint256 amount) external nonReentrant {
        if (amount == 0) {
            revert CannotReleaseZero();
        }

        (bool registered, Subnet storage subnet) = LibGateway.getSubnet(msg.sender);
        if (!registered) {
            revert NotRegisteredSubnet();
        }

        payable(subnet.id.getActor()).sendValue(amount);
    }

    /// @notice kill an existing subnet. It's balance must be empty
    function kill() external {
        (bool registered, Subnet storage subnet) = LibGateway.getSubnet(msg.sender);

        if (!registered) {
            revert NotRegisteredSubnet();
        }
        if (subnet.circSupply > 0) {
            revert NotEmptySubnetCircSupply();
        }

        uint256 stake = subnet.stake;

        s.totalSubnets -= 1;

        delete s.subnets[subnet.id.toHash()];

        payable(msg.sender).sendValue(stake);
    }

    /// @notice fund - commit a top-down message releasing funds in a child subnet. There is an associated fee that gets distributed to validators in the subnet as well
    /// @param subnetId - subnet to fund
    /// @param to - the address to send funds to
    function fund(SubnetID calldata subnetId, FvmAddress calldata to) external payable hasFee {
        CrossMsg memory crossMsg = CrossMsgHelper.createFundMsg(subnetId, msg.sender, to, msg.value - s.crossMsgFee);

        // commit top-down message.
        LibGateway.commitTopDownMsg(crossMsg);

        LibGateway.distributeRewards(subnetId.getActor(), s.crossMsgFee);
    }

    /// @notice release method locks funds in the current subnet and sends a cross message up the hierarchy to the parent gateway to release the funds
    function release(FvmAddress calldata to) external payable hasFee {
        CrossMsg memory crossMsg = CrossMsgHelper.createReleaseMsg(
            s.networkName,
            msg.sender,
            to,
            msg.value - s.crossMsgFee
        );

        LibGateway.commitBottomUpMsg(crossMsg);
    }

    /// @notice set up the top-down validators and their voting power
    /// @param validators - list of validator addresses
    /// @param weights - list of validators voting powers
    function setMembership(address[] memory validators, uint256[] memory weights) external systemActorOnly {
        if (validators.length != weights.length) {
            revert ValidatorsAndWeightsLengthMismatch();
        }
        // invalidate the previous validator set
        ++s.validatorNonce;

        uint256 totalValidatorsWeight = 0;

        // setup the new validator set
        uint256 validatorsLength = validators.length;
        for (uint256 validatorIndex = 0; validatorIndex < validatorsLength; ) {
            address validatorAddress = validators[validatorIndex];
            if (validatorAddress != address(0)) {
                uint256 validatorWeight = weights[validatorIndex];

                if (validatorWeight == 0) {
                    revert ValidatorWeightIsZero();
                }

                s.validatorSet[s.validatorNonce][validatorAddress] = validatorWeight;

                totalValidatorsWeight += validatorWeight;
            }

            // initial validators need to be conveniently funded with at least
            // 1 FIL for them to be able to commit the first few top-down messages.
            // They should use this FIL to fund their own addresses in the subnet
            // so they can keep committing top-down messages. If they don't do this,
            // they won't be able to send cross-net messages in their subnet.
            // Funds are only distributed in child subnets, where top-down checkpoints need
            // to be committed. This doesn't apply to the root.
            // TODO: Once account abstraction is conveniently supported, there will be
            // no need for this initial funding of validators.
            // if (block.number == 1 && !_networkName.isRoot())
            //     payable(validatorAddress).sendValue(INITIAL_VALIDATOR_FUNDS);

            unchecked {
                ++validatorIndex;
            }
        }
        s.totalWeight = totalValidatorsWeight;
    }
}
