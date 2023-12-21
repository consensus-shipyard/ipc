// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity 0.8.19;

import {VALIDATOR_SECP256K1_PUBLIC_KEY_LENGTH} from "../constants/Constants.sol";
import {ERR_PERMISSIONED_AND_BOOTSTRAPPED, ERR_VALIDATOR_JOINED, ERR_VALIDATOR_NOT_JOINED} from "../errors/IPCErrors.sol";
import {InvalidBatchEpoch, MaxMsgsPerBatchExceeded, BatchWithNoMessages, InvalidFederationPayload, SubnetAlreadyBootstrapped, NotEnoughFunds, CollateralIsZero, CannotReleaseZero, NotOwnerOfPublicKey, EmptyAddress, NotEnoughBalance, NotEnoughCollateral, NotValidator, NotAllValidatorsHaveLeft, NotStakedBefore, InvalidSignatureErr, InvalidCheckpointEpoch, InvalidPublicKeyLength, MethodNotAllowed} from "../errors/IPCErrors.sol";
import {IGateway} from "../interfaces/IGateway.sol";
import {ISubnetActor} from "../interfaces/ISubnetActor.sol";
import {QuorumObjKind} from "../structs/Quorum.sol";
import {BottomUpCheckpoint, BottomUpMsgBatch, BottomUpMsgBatchInfo} from "../structs/CrossNet.sol";
import {Validator, ValidatorSet, PermissionMode} from "../structs/Subnet.sol";
import {Pausable} from "../lib/LibPausable.sol";
import {LibDiamond} from "../lib/LibDiamond.sol";
import {MultisignatureChecker} from "../lib/LibMultisignatureChecker.sol";
import {ReentrancyGuard} from "../lib/LibReentrancyGuard.sol";
import {SubnetActorModifiers} from "../lib/LibSubnetActorStorage.sol";
import {LibValidatorSet, LibStaking} from "../lib/LibStaking.sol";
import {EnumerableSet} from "openzeppelin-contracts/utils/structs/EnumerableSet.sol";
import {Address} from "openzeppelin-contracts/utils/Address.sol";
import {LibSubnetActor} from "../lib/LibSubnetActor.sol";

contract SubnetActorManagerFacet is ISubnetActor, SubnetActorModifiers, Pausable, ReentrancyGuard {
    using EnumerableSet for EnumerableSet.AddressSet;
    using LibValidatorSet for ValidatorSet;
    using Address for address payable;

    event BottomUpCheckpointSubmitted(BottomUpCheckpoint checkpoint, address submitter);
    event BottomUpCheckpointExecuted(uint256 epoch, address submitter);
    event NextBottomUpCheckpointExecuted(uint256 epoch, address submitter);

    /// @notice Pauses all contract functions with the `whenNotPaused` modifier.
    function pause() external {
        LibDiamond.enforceIsContractOwner();
        _pause();
    }

    /// @notice Unpauses all contract functions with the `whenNotPaused` modifier.
    function unpause() external {
        LibDiamond.enforceIsContractOwner();
        _unpause();
    }

    /// @notice Submits a checkpoint commitment for execution.
    ///  @dev   It triggers the commitment of the checkpoint and any other side-effects that
    ///         need to be triggered by the checkpoint such as relayer reward book keeping.
    /// @param checkpoint The executed bottom-up checkpoint.
    /// @param signatories The addresses of validators signing the checkpoint.
    /// @param signatures The signatures of validators on the checkpoint.
    function submitCheckpoint(
        BottomUpCheckpoint calldata checkpoint,
        address[] calldata signatories,
        bytes[] calldata signatures
    ) external whenNotPaused {
        // the checkpoint height must be equal to the last bottom-up checkpoint height or
        // the next one
        if (
            checkpoint.blockHeight != s.lastBottomUpCheckpointHeight + s.bottomUpCheckPeriod &&
            checkpoint.blockHeight != s.lastBottomUpCheckpointHeight
        ) {
            revert InvalidCheckpointEpoch();
        }
        bytes32 checkpointHash = keccak256(abi.encode(checkpoint));

        if (checkpoint.blockHeight == s.lastBottomUpCheckpointHeight + s.bottomUpCheckPeriod) {
            // validate signatures and quorum threshold, revert if validation fails
            validateActiveQuorumSignatures({signatories: signatories, hash: checkpointHash, signatures: signatures});

            // If the checkpoint height is the next expected height then this is a new checkpoint which must be executed
            // in the Gateway Actor, the checkpoint and the relayer must be stored, last bottom-up checkpoint updated.
            s.committedCheckpoints[checkpoint.blockHeight] = checkpoint;

            // slither-disable-next-line unused-return
            s.relayerRewards.checkpointRewarded[checkpoint.blockHeight].add(msg.sender);

            s.lastBottomUpCheckpointHeight = checkpoint.blockHeight;

            // Commit in gateway to distribute rewards
            IGateway(s.ipcGatewayAddr).commitCheckpoint(checkpoint);

            // confirming the changes in membership in the child
            LibStaking.confirmChange(checkpoint.nextConfigurationNumber);
        } else if (checkpoint.blockHeight == s.lastBottomUpCheckpointHeight) {
            // If the checkpoint height is equal to the last checkpoint height, then this is a repeated submission.
            // We should store the relayer, but not to execute checkpoint again.
            // In this case, we do not verify the signatures for this checkpoint again,
            // but we add the relayer to the list of all relayers for this checkpoint to be rewarded later.
            // The reason for comparing hashes instead of verifying signatures is the following:
            // once the checkpoint is executed, the active validator set changes
            // and can only be used to validate the next checkpoint, not another instance of the last one.
            bytes32 lastCheckpointHash = keccak256(abi.encode(s.committedCheckpoints[checkpoint.blockHeight]));
            if (checkpointHash == lastCheckpointHash) {
                // slither-disable-next-line unused-return
                s.relayerRewards.checkpointRewarded[checkpoint.blockHeight].add(msg.sender);
            }
        }
    }

    /// @notice Submits a batch of bottom-up messages for execution.
    /// @dev It triggers the execution of a cross-net message batch.
    /// @param batch The batch of bottom-up messages.
    /// @param signatories The addresses of validators signing the batch.
    /// @param signatures The signatures of validators on the batch.
    function submitBottomUpMsgBatch(
        BottomUpMsgBatch calldata batch,
        address[] calldata signatories,
        bytes[] calldata signatures
    ) external {
        // forbid the submission of batches from the past
        if (batch.blockHeight < s.lastBottomUpBatch.blockHeight) {
            revert InvalidBatchEpoch();
        }
        if (batch.msgs.length > s.maxMsgsPerBottomUpBatch) {
            revert MaxMsgsPerBatchExceeded();
        }
        // if the batch height is not max, we only supoprt batch submission in period epochs
        if (batch.msgs.length != s.maxMsgsPerBottomUpBatch && batch.blockHeight % s.bottomUpMsgBatchPeriod != 0) {
            revert InvalidBatchEpoch();
        }
        if (batch.msgs.length == 0) {
            revert BatchWithNoMessages();
        }

        bytes32 batchHash = keccak256(abi.encode(batch));

        if (batch.blockHeight == s.lastBottomUpBatch.blockHeight) {
            // If the batch info is equal to the last batch info, then this is a repeated submission.
            // We should store the relayer, but not to execute batch again following the same reward logic
            // used for checkpoints.
            if (batchHash == s.lastBottomUpBatch.hash) {
                // slither-disable-next-line unused-return
                s.relayerRewards.batchRewarded[batch.blockHeight].add(msg.sender);
            }
        } else {
            // validate signatures and quorum threshold, revert if validation fails
            validateActiveQuorumSignatures({signatories: signatories, hash: batchHash, signatures: signatures});

            // If the checkpoint height is the next expected height then this is a new batch,
            // and should be forwarded to the gateway for execution.
            s.lastBottomUpBatch = BottomUpMsgBatchInfo({blockHeight: batch.blockHeight, hash: batchHash});

            // slither-disable-next-line unused-return
            s.relayerRewards.batchRewarded[batch.blockHeight].add(msg.sender);

            // Execute messages.
            IGateway(s.ipcGatewayAddr).execBottomUpMsgBatch(batch);
        }
    }

    /// @notice method to add some initial balance into a subnet that hasn't yet bootstrapped.
    /// @dev This balance is added to user addresses in genesis, and becomes part of the genesis
    /// circulating supply.
    function preFund() external payable {
        if (msg.value == 0) {
            revert NotEnoughFunds();
        }

        if (s.bootstrapped) {
            revert SubnetAlreadyBootstrapped();
        }

        if (s.genesisBalance[msg.sender] == 0) {
            s.genesisBalanceKeys.push(msg.sender);
        }

        s.genesisBalance[msg.sender] += msg.value;
        s.genesisCircSupply += msg.value;
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

        if (s.genesisBalance[msg.sender] < amount) {
            revert NotEnoughBalance();
        }

        s.genesisBalance[msg.sender] -= amount;
        s.genesisCircSupply -= amount;

        if (s.genesisBalance[msg.sender] == 0) {
            LibSubnetActor.rmAddressFromBalanceKey(msg.sender);
        }

        payable(msg.sender).sendValue(amount);
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
            LibSubnetActor.postBootstrapSetFederatedPower(validators, publicKeys, powers);
        } else {
            LibSubnetActor.preBootstrapSetFederatedPower(validators, publicKeys, powers);
        }
    }

    /// @notice method that allows a validator to join the subnet.
    ///         If the total confirmed collateral of the subnet is greater
    ///         or equal to minimum activation collateral as a result of this operation,
    ///         then  subnet will be registered.
    /// @param publicKey The off-chain 65 byte public key that should be associated with the validator
    function join(bytes calldata publicKey) external payable nonReentrant whenNotPaused notKilled {
        // adding this check to prevent new validators from joining
        // after the subnet has been bootstrapped. We will increase the
        // functionality in the future to support explicit permissioning.
        if (s.bootstrapped) {
            LibSubnetActor.enforceCollateralValidation();
        }
        if (msg.value == 0) {
            revert CollateralIsZero();
        }

        if (LibStaking.isValidator(msg.sender)) {
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

        if (!s.bootstrapped) {
            // if the subnet has not been bootstrapped, join directly
            // without delays, and collect collateral to register
            // in the gateway

            // confirm validators deposit immediately
            LibStaking.setMetadataWithConfirm(msg.sender, publicKey);
            LibStaking.depositWithConfirm(msg.sender, msg.value);

            LibSubnetActor.bootstrapSubnetIfNeeded();
        } else {
            // if the subnet has been bootstrapped, join with postponed confirmation.
            LibStaking.setValidatorMetadata(msg.sender, publicKey);
            LibStaking.deposit(msg.sender, msg.value);
        }
    }

    /// @notice method that allows a validator to increase its stake.
    ///         If the total confirmed collateral of the subnet is greater
    ///         or equal to minimum activation collateral as a result of this operation,
    ///         then  subnet will be registered.
    function stake() external payable whenNotPaused notKilled {
        // disabling validator changes for federated subnets (at least for now
        // until a more complex mechanism is implemented).
        LibSubnetActor.enforceCollateralValidation();
        if (msg.value == 0) {
            revert CollateralIsZero();
        }

        if (!LibStaking.isValidator(msg.sender)) {
            revert MethodNotAllowed(ERR_VALIDATOR_NOT_JOINED);
        }

        if (!s.bootstrapped) {
            LibStaking.depositWithConfirm(msg.sender, msg.value);

            LibSubnetActor.bootstrapSubnetIfNeeded();
        } else {
            LibStaking.deposit(msg.sender, msg.value);
        }
    }

    /// @notice method that allows a validator to unstake a part of its collateral from a subnet.
    /// @dev `leave` must be used to unstake the entire stake.
    /// @param amount The amount to unstake.
    function unstake(uint256 amount) external whenNotPaused notKilled {
        // disbling validator changes for federated validation subnets (at least for now
        // until a more complex mechanism is implemented).
        LibSubnetActor.enforceCollateralValidation();

        if (amount == 0) {
            revert CannotReleaseZero();
        }

        uint256 collateral = LibStaking.totalValidatorCollateral(msg.sender);

        if (collateral == 0) {
            revert NotValidator(msg.sender);
        }
        if (collateral <= amount) {
            revert NotEnoughCollateral();
        }
        if (!s.bootstrapped) {
            LibStaking.withdrawWithConfirm(msg.sender, amount);
            return;
        }

        LibStaking.withdraw(msg.sender, amount);
    }

    /// @notice method that allows a validator to leave the subnet.
    function leave() external nonReentrant whenNotPaused notKilled {
        // disbling validator changes for federated subnets (at least for now
        // until a more complex mechanism is implemented).
        // This means that initial validators won't be able to recover
        // their collateral ever (worth noting in the docs if this ends
        // up sticking around for a while).
        if (s.bootstrapped) {
            LibSubnetActor.enforceCollateralValidation();
        }

        // remove bootstrap nodes added by this validator
        uint256 amount = LibStaking.totalValidatorCollateral(msg.sender);
        if (amount == 0) {
            revert NotValidator(msg.sender);
        }

        // slither-disable-next-line unused-return
        s.bootstrapOwners.remove(msg.sender);
        delete s.bootstrapNodes[msg.sender];

        if (!s.bootstrapped) {
            // check if the validator had some initial balance and return it if not bootstrapped
            uint256 genesisBalance = s.genesisBalance[msg.sender];
            if (genesisBalance != 0) {
                s.genesisBalance[msg.sender] == 0;
                s.genesisCircSupply -= genesisBalance;
                LibSubnetActor.rmAddressFromBalanceKey(msg.sender);
                payable(msg.sender).sendValue(genesisBalance);
            }

            // interaction must be performed after checks and changes
            LibStaking.withdrawWithConfirm(msg.sender, amount);
            return;
        }
        LibStaking.withdraw(msg.sender, amount);
    }

    /// @notice method that allows to kill the subnet when all validators left.
    /// @dev It is not a privileged operation.
    function kill() external notKilled {
        if (LibStaking.totalValidators() != 0) {
            revert NotAllValidatorsHaveLeft();
        }

        s.killed = true;
        IGateway(s.ipcGatewayAddr).kill();
    }

    /// @notice Validator claims their released collateral.
    function claim() external nonReentrant whenNotPaused {
        LibStaking.claimCollateral(msg.sender);
    }

    /// @notice Relayer claims its reward.
    function claimRewardForRelayer() external nonReentrant whenNotPaused {
        LibStaking.claimRewardForRelayer(msg.sender);
    }

    /// @notice Add a bootstrap node.
    /// @param netAddress The network address of the new bootstrap node.
    function addBootstrapNode(string memory netAddress) external whenNotPaused {
        if (!s.validatorSet.isActiveValidator(msg.sender)) {
            revert NotValidator(msg.sender);
        }
        if (bytes(netAddress).length == 0) {
            revert EmptyAddress();
        }
        s.bootstrapNodes[msg.sender] = netAddress;
        // slither-disable-next-line unused-return
        s.bootstrapOwners.add(msg.sender);
    }

    /// @notice Distributes rewards to relayers for a specific checkpoint.
    /// @param height The height of the checkpoint for which rewards are being distributed.
    /// @param reward The total amount of reward to be distributed.
    /// @param kind The type of object for which rewards are being distributed.
    function distributeRewardToRelayers(
        uint256 height,
        uint256 reward,
        QuorumObjKind kind
    ) external payable whenNotPaused onlyGateway {
        if (reward == 0) {
            return;
        }

        // get rewarded addresses
        address[] memory relayers = new address[](0);
        if (kind == QuorumObjKind.Checkpoint) {
            relayers = LibSubnetActor.checkpointRewardedAddrs(height);
        } else if (kind == QuorumObjKind.BottomUpMsgBatch) {
            // FIXME: The distribution of rewards for batches can't be done
            // as for checkpoints (due to how they are submitted). As
            // we are running out of time, we'll defer this for the future.
            revert MethodNotAllowed("rewards not defined for batches");
        } else {
            revert MethodNotAllowed("rewards not defined for object kind");
        }

        // comupte reward
        // we are not distributing equally, this logic should be decoupled
        // into different reward policies.
        uint256 relayersLength = relayers.length;
        if (relayersLength == 0) {
            return;
        }
        if (reward < relayersLength) {
            return;
        }
        uint256 relayerReward = reward / relayersLength;

        // distribute reward
        for (uint256 i; i < relayersLength; ) {
            s.relayerRewards.rewards[relayers[i]] += relayerReward;
            unchecked {
                ++i;
            }
        }
    }

    /// @notice Checks whether the signatures are valid for the provided signatories and hash within the current validator set.
    ///         Reverts otherwise.
    /// @dev Signatories in `signatories` and their signatures in `signatures` must be provided in the same order.
    ///       Having it public allows external users to perform sanity-check verification if needed.
    /// @param signatories The addresses of the signatories.
    /// @param hash The hash of the checkpoint.
    /// @param signatures The packed signatures of the checkpoint.
    function validateActiveQuorumSignatures(
        address[] memory signatories,
        bytes32 hash,
        bytes[] memory signatures
    ) public view {
        // This call reverts if at least one of the signatories (validator) is not in the active validator set.
        uint256[] memory collaterals = s.validatorSet.getTotalPowerOfValidators(signatories);
        uint256 activeCollateral = s.validatorSet.getTotalActivePower();

        uint256 threshold = (activeCollateral * s.majorityPercentage) / 100;

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
