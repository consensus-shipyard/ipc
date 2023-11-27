// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity 0.8.19;

import {SubnetAlreadyBootstrapped, NotEnoughFunds, CollateralIsZero, CannotReleaseZero, NotOwnerOfPublicKey, EmptyAddress, NotEnoughBalance, NotEnoughBalanceForRewards, NotEnoughCollateral, NotValidator, NotAllValidatorsHaveLeft, NotStakedBefore, InvalidSignatureErr, InvalidCheckpointEpoch, InvalidCheckpointMessagesHash, InvalidPublicKeyLength} from "../errors/IPCErrors.sol";
import {IGateway} from "../interfaces/IGateway.sol";
import {ISubnetActor} from "../interfaces/ISubnetActor.sol";
import {BottomUpCheckpoint, CrossMsg} from "../structs/Checkpoint.sol";
import {SubnetID, Validator, ValidatorSet} from "../structs/Subnet.sol";
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
    using LibValidatorSet for ValidatorSet;
    using Address for address payable;

    event BottomUpCheckpointSubmitted(BottomUpCheckpoint checkpoint, address submitter);
    event BottomUpCheckpointExecuted(uint64 epoch, address submitter);
    event NextBottomUpCheckpointExecuted(uint64 epoch, address submitter);
    event SubnetBootstrapped(Validator[]);

    /** @notice submit a checkpoint for execution.
     *  @dev It triggers the commitment of the checkpoint and the execution of related cross-net messages,
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
        // the checkpoint height must be equal to the last bottom-up checkpoint height or
        // the next one
        if (
            checkpoint.blockHeight != s.lastBottomUpCheckpointHeight + s.bottomUpCheckPeriod &&
            checkpoint.blockHeight != s.lastBottomUpCheckpointHeight
        ) {
            revert InvalidCheckpointEpoch();
        }
        if (keccak256(abi.encode(messages)) != checkpoint.crossMessagesHash) {
            revert InvalidCheckpointMessagesHash();
        }
        bytes32 checkpointHash = keccak256(abi.encode(checkpoint));

        if (checkpoint.blockHeight == s.lastBottomUpCheckpointHeight + s.bottomUpCheckPeriod) {
            // validate signatures and quorum threshold, revert if validation fails
            validateActiveQuorumSignatures({signatories: signatories, hash: checkpointHash, signatures: signatures});

            // If the checkpoint height is the next expected height then this is a new checkpoint which must be executed
            // in the Gateway Actor, the checkpoint and the relayer must be stored, last bottom-up checkpoint updated.
            s.committedCheckpoints[checkpoint.blockHeight] = checkpoint;

            // slither-disable-next-line unused-return
            s.rewardedRelayers[checkpoint.blockHeight].add(msg.sender);

            s.lastBottomUpCheckpointHeight = checkpoint.blockHeight;

            // Execute messages.
            IGateway(s.ipcGatewayAddr).commitBottomUpCheckpoint(checkpoint, messages);

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
                s.rewardedRelayers[checkpoint.blockHeight].add(msg.sender);
            }
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
            rmAddressFromBalanceKey(msg.sender);
        }

        payable(msg.sender).sendValue(amount);
    }

    /// @notice method that allows a validator to join the subnet
    /// @param publicKey The off-chain 65 byte public key that should be associated with the validator
    function join(bytes calldata publicKey) external payable nonReentrant notKilled {
        if (msg.value == 0) {
            revert CollateralIsZero();
        }

        if (publicKey.length != 65) {
            // Taking 65 bytes because the FVM libraries have some assertions checking it, it's more convenient.
            revert InvalidPublicKeyLength();
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

            if (totalCollateral >= s.minActivationCollateral) {
                if (LibStaking.totalActiveValidators() >= s.minValidators) {
                    s.bootstrapped = true;
                    emit SubnetBootstrapped(s.genesisValidators);

                    // register adding the genesis circulating supply (if it exists)
                    IGateway(s.ipcGatewayAddr).register{value: totalCollateral + s.genesisCircSupply}(
                        s.genesisCircSupply
                    );
                }
            }
        } else {
            LibStaking.setValidatorMetadata(msg.sender, publicKey);
            LibStaking.deposit(msg.sender, msg.value);
        }
    }

    /// @notice method that allows a validator to increase its stake
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

    /// @notice method that allows a validator to unstake a part of its collateral from a subnet
    /// @dev `leave` must be used to unstake the entire stake.
    function unstake(uint256 amount) external notKilled {
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

    /// @notice method that allows a validator to leave the subnet
    /// @dev it also return the validators initial balance if the
    /// subnet was not yet bootstrapped.
    function leave() external notKilled nonReentrant {
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
                rmAddressFromBalanceKey(msg.sender);
                payable(msg.sender).sendValue(genesisBalance);
            }

            // interaction must be performed after checks and changes
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

    /// @notice Validator claims their released collateral
    function claim() external nonReentrant {
        LibStaking.claimCollateral(msg.sender);
    }

    /// @notice Relayer claims its reward
    function claimRewardForRelayer() external nonReentrant {
        LibStaking.claimRewardForRelayer(msg.sender);
    }

    /// @notice add a bootstrap node
    function addBootstrapNode(string memory netAddress) external {
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

    /// @notice reward the relayers for of the previous checkpoint after processing the one at height `height`.
    /// @dev The reward includes the fixed relayer reward and accumulated cross-message fees received from the gateway.
    /// @param height height of the checkpoint the relayers are rewarded for
    /// @param reward The sum of cross-message fees in the checkpoint
    function distributeRewardToRelayers(uint64 height, uint256 reward) external payable onlyGateway {
        if (reward == 0) {
            return;
        }
        uint64 previousHeight = height - s.bottomUpCheckPeriod;
        address[] memory relayers = s.rewardedRelayers[previousHeight].values();
        uint256 relayersLength = relayers.length;
        if (relayersLength == 0) {
            return;
        }
        if (reward < relayersLength) {
            // Reverting here would mean a single message with 1 atto reward
            // relayed by 2 validators would mean the checkpoint cannot be
            // submitted.
            return;
        }
        uint256 relayerReward = reward / relayersLength;

        for (uint256 i; i < relayersLength; ) {
            s.relayerRewards[relayers[i]] += relayerReward;
            unchecked {
                ++i;
            }
        }
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
        uint256 activeCollateral = s.validatorSet.getActiveCollateral();

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

    /**
     * @notice Hash a 65 byte public key and return the corresponding address.
     */
    function publicKeyToAddress(bytes calldata publicKey) internal pure returns (address) {
        assert(publicKey.length == 65);
        bytes32 hashed = keccak256(publicKey[1:]);
        return address(uint160(uint256(hashed)));
    }

    /// @notice Removes an address from the initial balance keys
    function rmAddressFromBalanceKey(address addr) internal {
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
