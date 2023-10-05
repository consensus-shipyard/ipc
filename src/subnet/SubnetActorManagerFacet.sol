// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity 0.8.19;

import {CollateralIsZero, EmptyAddress, MessagesNotSorted, NotEnoughBalanceForRewards, NoValidatorsInSubnet, NotValidator, NotAllValidatorsHaveLeft, SubnetNotActive, WrongCheckpointSource, NoRewardToWithdraw, NotStakedBefore, InconsistentPrevCheckpoint, InvalidSignatureErr} from "../errors/IPCErrors.sol";
import {IGateway} from "../interfaces/IGateway.sol";
import {ISubnetActor} from "../interfaces/ISubnetActor.sol";
import {BottomUpCheckpoint} from "../structs/Checkpoint.sol";
import {FvmAddress} from "../structs/FvmAddress.sol";
import {SubnetID, Validator, ValidatorSet} from "../structs/Subnet.sol";
import {CheckpointHelper} from "../lib/CheckpointHelper.sol";
import {CrossMsgHelper} from "../lib/CrossMsgHelper.sol";
import {MultisignatureChecker} from "../lib/LibMultisignatureChecker.sol";
import {ReentrancyGuard} from "../lib/LibReentrancyGuard.sol";
import {SubnetActorModifiers} from "../lib/LibSubnetActorStorage.sol";
import {SubnetIDHelper} from "../lib/SubnetIDHelper.sol";
import {LibValidatorSet, LibStaking} from "../lib/LibStaking.sol";
import {EnumerableSet} from "openzeppelin-contracts/utils/structs/EnumerableSet.sol";
import {Address} from "openzeppelin-contracts/utils/Address.sol";

contract SubnetActorManagerFacet is ISubnetActor, SubnetActorModifiers, ReentrancyGuard {
    using EnumerableSet for EnumerableSet.AddressSet;
    using SubnetIDHelper for SubnetID;
    using CheckpointHelper for BottomUpCheckpoint;
    using LibValidatorSet for ValidatorSet;
    using Address for address payable;

    event BottomUpCheckpointSubmitted(BottomUpCheckpoint checkpoint, address submitter);
    event BottomUpCheckpointExecuted(uint64 epoch, address submitter);
    event NextBottomUpCheckpointExecuted(uint64 epoch, address submitter);

    /// @notice methods that allows a validator to submit a checkpoint (batch of messages) and vote for it with it's own voting power.
    /// @param checkpoint - the batch messages data
    /// @param membershipRootHash - a root hash of the Merkle tree built from the validator public keys and their weight
    /// @param membershipWeight - the total weight of the membership
    function submitCheckpoint(
        BottomUpCheckpoint calldata checkpoint,
        bytes32 membershipRootHash,
        uint256 membershipWeight
    ) external {
        if (!LibStaking.isActiveValidator(msg.sender)) {
            revert NotValidator();
        }
        if (checkpoint.subnetID.toHash() != s.currentSubnetHash) {
            revert WrongCheckpointSource();
        }

        _commitCheckpoint({
            checkpoint: checkpoint,
            membershipRootHash: membershipRootHash,
            membershipWeight: membershipWeight
        });

        LibStaking.confirmChange(checkpoint.nextConfigurationNumber);
    }

    /// @notice Set the data of a validator
    function setMetadata(bytes calldata metadata) external {
        if (!LibStaking.hasStaked(msg.sender)) {
            revert NotStakedBefore();
        }
        LibStaking.setValidatorMetadata(msg.sender, metadata);
    }

    /// @notice method that allows a validator to join the subnet
    /// @param metadata The offchain data that should be associated with the validator
    function join(bytes calldata metadata) external payable notKilled {
        if (msg.value == 0) {
            revert CollateralIsZero();
        }

        LibStaking.setValidatorMetadata(msg.sender, metadata);
        LibStaking.deposit(msg.sender, msg.value);
    }

    /// @notice method that allows a validator to increase their stake
    function stake() external payable notKilled {
        if (msg.value == 0) {
            revert CollateralIsZero();
        }

        if (!LibStaking.hasStaked(msg.sender)) {
            revert NotStakedBefore();
        }

        LibStaking.deposit(msg.sender, msg.value);
    }

    /// @notice method that allows a validator to leave the subnet
    function leave() external notKilled {
        uint256 amount = LibStaking.totalValidatorCollateral(msg.sender);
        if (amount == 0) {
            revert NotValidator();
        }

        LibStaking.withdraw(msg.sender, amount);
    }

    /// @notice method that allows to kill the subnet when all validators left. It is not a privileged operation.
    function kill() external notKilled {
        if (LibStaking.totalValidators() != 0) {
            revert NotAllValidatorsHaveLeft();
        }

        s.killed = true;
        IGateway(s.ipcGatewayAddr).kill();
    }

    /// @notice Valdiator claims their released collateral
    function claim() external nonReentrant {
        LibStaking.claimCollateral(msg.sender);
    }

    /// @notice method that commits a checkpoint after reaching majority
    /// @param checkpoint - the batch messages data
    function _commitCheckpoint(
        BottomUpCheckpoint calldata checkpoint,
        bytes32 membershipRootHash,
        uint256 membershipWeight
    ) internal {
        s.committedCheckpoints[checkpoint.blockHeight] = checkpoint;
        s.prevExecutedCheckpointHash = checkpoint.toHash();

        IGateway(s.ipcGatewayAddr).createBottomUpCheckpoint({
            checkpoint: checkpoint,
            membershipRootHash: membershipRootHash,
            membershipWeight: membershipWeight
        });
    }

    /**
     * @notice Checks whether the checkpoint is valid for the provided signatories, signatures and hash. Reverts otherwise.
     * @dev Signatories in `signatories` and their signatures in `signatures` must be provided in the same order.
     * @param signatories The addresses of the signatories.
     * @param hash The hash of the checkpoint.
     * @param signatures The packed signatures of the checkpoint.
     */
    function validateCheckpoint(address[] memory signatories, bytes32 hash, bytes memory signatures) external view {
        uint256[] memory collaterals = s.validatorSet.getConfirmedCollaterals(signatories);

        uint256 threshold = (s.validatorSet.totalConfirmedCollateral * s.majorityPercentage) / 100;

        (bool valid, MultisignatureChecker.Error err) = MultisignatureChecker.isValidWeightedMultiSignature({
            signatories: signatories,
            weights: collaterals,
            threshold: threshold,
            hash: hash,
            signatures: signatures
        });

        if (!valid) {
            revert InvalidSignatureErr(uint8(err));
        }
    }
}
