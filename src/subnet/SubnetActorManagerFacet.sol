// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity 0.8.19;

import {CollateralIsZero, NotOwnerOfPublicKey, EmptyAddress, MessagesNotSorted, NotEnoughBalanceForRewards, NoValidatorsInSubnet, NotValidator, NotAllValidatorsHaveLeft, SubnetNotActive, WrongCheckpointSource, NoRewardToWithdraw, NotStakedBefore, InconsistentPrevCheckpoint, InvalidSignatureErr, HeightAlreadyExecuted, InvalidCheckpointEpoch, InvalidCheckpointMessagesHash} from "../errors/IPCErrors.sol";
import {IGateway} from "../interfaces/IGateway.sol";
import {ISubnetActor} from "../interfaces/ISubnetActor.sol";
import {BottomUpCheckpoint, CrossMsg} from "../structs/Checkpoint.sol";
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
    event SubnetBootstrapped(Validator[]);

    /** @notice Executes the checkpoint if it is valid.
     *  @dev It triggers the commitment of the checkpoint, the execution of related cross-net messages,
     *       and any other side-effects that need to be triggered by the checkpoint such as relayer reward book keeping.
     * @param checkpoint The executed bottom-up checkpoint
     * @param messages The list of executed cross-messages
     * @param signatories The addresses of the signatories
     * @param signatures The collected checkpoint signatures
     */
    function submitCheckpoint(
        BottomUpCheckpoint calldata checkpoint,
        CrossMsg[] calldata messages,
        address[] calldata signatories,
        bytes[] calldata signatures
    ) external {
        // validations
        if (!LibStaking.isActiveValidator(msg.sender)) {
            revert NotValidator();
        }
        if (checkpoint.blockHeight != s.lastBottomUpCheckpointHeight + s.bottomUpCheckPeriod) {
            revert InvalidCheckpointEpoch();
        }
        if (keccak256(abi.encode(messages)) != checkpoint.crossMessagesHash) {
            revert InvalidCheckpointMessagesHash();
        }
        bytes32 checkpointHash = keccak256(abi.encode(checkpoint));
        validateActiveQuorumSignatures({signatories: signatories, hash: checkpointHash, signatures: signatures});

        // effects
        s.committedCheckpoints[checkpoint.blockHeight] = checkpoint;
        s.lastBottomUpCheckpointHeight = checkpoint.blockHeight;

        // interactions
        IGateway(s.ipcGatewayAddr).commitBottomUpCheckpoint(checkpoint);
    }

    /// @notice method that allows a validator to join the subnet
    /// @param publicKey The offchain public key that should be associated with the validator
    function join(bytes calldata publicKey) external payable nonReentrant notKilled {
        if (msg.value == 0) {
            revert CollateralIsZero();
        }

        address convertedAddress = publicKeyToAddress(publicKey);
        if (convertedAddress != msg.sender) {
            revert NotOwnerOfPublicKey();
        }

        if (!s.bootstrapped) {
            // if the subnet has not been bootstrapped, join directly
            // without delays, and collect collateral to register
            // in the gateway

            // confirm validators deposit immediately
            LibStaking.setMetadataWithConfirm(msg.sender, publicKey);
            LibStaking.depositWithConfirm(msg.sender, msg.value);

            uint256 totalCollateral = LibStaking.getTotalConfirmedCollateral();

            if (totalCollateral >= s.minActivationCollateral && LibStaking.totalActiveValidators() >= s.minValidators) {
                s.bootstrapped = true;
                emit SubnetBootstrapped(s.genesisValidators);

                IGateway(s.ipcGatewayAddr).register{value: totalCollateral}();
            }
        } else {
            LibStaking.setValidatorMetadata(msg.sender, publicKey);
            LibStaking.deposit(msg.sender, msg.value);
        }
    }

    /// @notice method that allows a validator to increase their stake
    function stake() external payable notKilled {
        if (msg.value == 0) {
            revert CollateralIsZero();
        }

        if (!LibStaking.hasStaked(msg.sender)) {
            revert NotStakedBefore();
        }

        if (!s.bootstrapped) {
            LibStaking.depositWithConfirm(msg.sender, msg.value);
            return;
        }

        LibStaking.deposit(msg.sender, msg.value);
    }

    /// @notice method that allows a validator to leave the subnet
    function leave() external notKilled {
        uint256 amount = LibStaking.totalValidatorCollateral(msg.sender);
        if (amount == 0) {
            revert NotValidator();
        }

        if (!s.bootstrapped) {
            LibStaking.withdrawWithConfirm(msg.sender, amount);
            return;
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

    /**
     * @notice Checks whether the signatures are valid for the provided signatories and hash within the current validator set.
     *         Reverts otherwise.
     * @dev Signatories in `signatories` and their signatures in `signatures` must be provided in the same order.
     *       Having it public allows external users to perform sanity-check verification if needed.
     * @param signatories The addresses of the signatories.
     * @param hash The hash of the checkpoint.
     * @param signatures The packed signatures of the checkpoint.
     */
    function validateActiveQuorumSignatures(
        address[] memory signatories,
        bytes32 hash,
        bytes[] memory signatures
    ) public view {
        // This call reverts if at least one of the signatories (validator) is not in the active validator set.
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

    function publicKeyToAddress(bytes memory publicKey) internal pure returns (address) {
        bytes32 hashed = keccak256(publicKey);
        return address(uint160(bytes20(hashed)));
    }
}
