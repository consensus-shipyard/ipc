// SPDX-License-Identifier: UNLICENSED
pragma solidity 0.8.19;

import {Voting} from "./Voting.sol";
import {ConsensusType} from "./enums/ConsensusType.sol";
import {Status} from "./enums/Status.sol";
import {BottomUpCheckpoint, CrossMsg} from "./structs/Checkpoint.sol";
import {SubnetID} from "./structs/Subnet.sol";
import {EpochVoteSubmission} from "./structs/EpochVoteSubmission.sol";
import {ISubnetActor} from "./interfaces/ISubnetActor.sol";
import {IGateway} from "./interfaces/IGateway.sol";
import {AccountHelper} from "./lib/AccountHelper.sol";
import {CrossMsgHelper} from "./lib/CrossMsgHelper.sol";
import {ExecutableQueue} from "./structs/ExecutableQueue.sol";
import {ExecutableQueueHelper} from "./lib/ExecutableQueueHelper.sol";
import {EpochVoteBottomUpSubmission} from "./structs/EpochVoteSubmission.sol";
import {EpochVoteSubmissionHelper} from "./lib/EpochVoteSubmissionHelper.sol";
import {SubnetIDHelper} from "./lib/SubnetIDHelper.sol";
import {CheckpointHelper} from "./lib/CheckpointHelper.sol";
import {FilAddress} from "fevmate/utils/FilAddress.sol";
import {EnumerableSet} from "openzeppelin-contracts/utils/structs/EnumerableSet.sol";
import {ReentrancyGuard} from "openzeppelin-contracts/security/ReentrancyGuard.sol";
import {Address} from "openzeppelin-contracts/utils/Address.sol";
import {FvmAddress} from "./structs/FvmAddress.sol";
import {FvmAddressHelper} from "./lib/FvmAddressHelper.sol";

/// @title Subnet Actor Contract
/// @author LimeChain team
contract SubnetActor is ISubnetActor, ReentrancyGuard, Voting {
    using EnumerableSet for EnumerableSet.AddressSet;
    using SubnetIDHelper for SubnetID;
    using CheckpointHelper for BottomUpCheckpoint;
    using FilAddress for address;
    using Address for address payable;
    using AccountHelper for address;
    using ExecutableQueueHelper for ExecutableQueue;
    using EpochVoteSubmissionHelper for EpochVoteSubmission;
    using CrossMsgHelper for CrossMsg;
    using FvmAddressHelper for FvmAddress;

    /// @notice minimum collateral validators need to stake in order to join the subnet. Values get clamped to this
    uint256 public constant MIN_COLLATERAL_AMOUNT = 1 ether;

    /// @notice The minimum collateral required to be a validator in this subnet
    uint256 public immutable minActivationCollateral;

    /// @notice Total collateral currently deposited in the GW from the subnet
    uint256 public totalStake;

    /// @notice number of blocks in a top-down epoch
    uint64 public immutable topDownCheckPeriod;

    /// @notice number of blocks in a bottom-up epoch
    uint64 public immutable bottomUpCheckPeriod;

    /// @notice Minimal number of validators required for the subnet to be able to validate new blocks.
    uint64 public immutable minValidators;

    /// @notice Sequence number that uniquely identifies a validator set.
    uint64 public configurationNumber;

    /// @notice Address of the IPC gateway for the subnet
    address public immutable ipcGatewayAddr;

    /// @notice current status of the subnet
    Status public status;

    /// @notice Type of consensus algorithm.
    ConsensusType public immutable consensus;

    /// @notice contains the last executed checkpoint hash
    bytes32 public prevExecutedCheckpointHash;

    /// @notice Human-readable name of the subnet.
    bytes32 public immutable name;

    // @notice Hash of the current subnet id
    bytes32 public immutable currentSubnetHash;

    /// @notice contains all committed bottom-up checkpoint at specific epoch
    mapping(uint64 => BottomUpCheckpoint) public committedCheckpoints;

    /// @notice List of validators in the subnet
    EnumerableSet.AddressSet private _validators;

    /// @notice contains voted submissions for a given epoch
    // slither-disable-next-line uninitialized-state
    mapping(uint64 => EpochVoteBottomUpSubmission) private _epochVoteSubmissions;

    /// @notice validator address to stake amount
    mapping(address => uint256) public stake;

    /// @notice validator address to accumulated rewards
    mapping(address => uint256) public accumulatedRewards;

    /// @notice validator address to validator net address
    mapping(address => string) public validatorNetAddresses;

    /// @notice validator address to validator worker address
    mapping(address => FvmAddress) public validatorWorkerAddresses;

    /// @notice ID of the parent subnet
    SubnetID private _parentId;

    /// @notice genesis block
    bytes public genesis;

    error NotGateway();
    error NotAccount();
    error WorkerAddressInvalid();
    error CollateralIsZero();
    error CallerHasNoStake();
    error CollateralStillLockedInSubnet();
    error SubnetAlreadyKilled();
    error NotAllValidatorsHaveLeft();
    error NotValidator();
    error SubnetNotActive();
    error WrongCheckpointSource();
    error CheckpointNotChained();
    error NoValidatorsInSubnet();
    error NotEnoughBalanceForRewards();
    error MessagesNotSorted();
    error NoRewardToWithdraw();
    error GatewayCannotBeZero();

    function _signableOnly() private view {
        if (!msg.sender.isAccount()) {
            revert NotAccount();
        }
    }

    function _onlyGateway() private view {
        if (msg.sender != ipcGatewayAddr) {
            revert NotGateway();
        }
    }

    function _notKilled() private view {
        if (status == Status.Killed) {
            revert SubnetAlreadyKilled();
        }
    }

    modifier onlyGateway() {
        _onlyGateway();
        _;
    }

    modifier signableOnly() {
        _signableOnly();
        _;
    }

    modifier notKilled() {
        _notKilled();
        _;
    }

    struct ValidatorInfo {
        address addr;
        uint256 weight;
        FvmAddress workerAddr;
        string netAddresses;
    }

    struct ValidatorSet {
        ValidatorInfo[] validators;
        uint64 configurationNumber;
    }

    struct ConstructParams {
        SubnetID parentId;
        bytes32 name;
        address ipcGatewayAddr;
        ConsensusType consensus;
        uint256 minActivationCollateral;
        uint64 minValidators;
        uint64 bottomUpCheckPeriod;
        uint64 topDownCheckPeriod;
        uint8 majorityPercentage;
        bytes genesis;
    }

    constructor(ConstructParams memory params) Voting(params.majorityPercentage, params.bottomUpCheckPeriod) {
        _parentId = params.parentId;
        name = params.name;
        if (params.ipcGatewayAddr == address(0)) {
            revert GatewayCannotBeZero();
        }
        ipcGatewayAddr = params.ipcGatewayAddr;
        consensus = params.consensus;
        minActivationCollateral = params.minActivationCollateral < MIN_COLLATERAL_AMOUNT
            ? MIN_COLLATERAL_AMOUNT
            : params.minActivationCollateral;
        minValidators = params.minValidators;
        topDownCheckPeriod = params.topDownCheckPeriod < MIN_CHECKPOINT_PERIOD
            ? MIN_CHECKPOINT_PERIOD
            : params.topDownCheckPeriod;
        bottomUpCheckPeriod = submissionPeriod;
        status = Status.Instantiated;
        genesis = params.genesis;
        currentSubnetHash = _parentId.createSubnetId(address(this)).toHash();
        // NOTE: we currently use 0 as the genesisEpoch for subnets so checkpoints
        // are submitted directly from epoch 0.
        // In the future we can use the current epoch. This will be really
        // useful once we support the docking of subnets to new parents, etc.
        _genesisEpoch = 0;
    }

    /* solhint-disable no-empty-blocks */
    receive() external payable onlyGateway {}

    /* solhint-enable no-empty-blocks */

    /// @notice method that allows a validator to join the subnet
    /// @param netAddr - the network address of the validator
    function join(string calldata netAddr, FvmAddress calldata workerAddr) external payable signableOnly notKilled {
        uint256 validatorStake = msg.value;
        address validator = msg.sender;
        if (validatorStake == 0) {
            revert CollateralIsZero();
        }

        stake[validator] += validatorStake;
        totalStake += validatorStake;

        if (stake[validator] >= minActivationCollateral) {
            if (!_validators.contains(validator)) {
                // slither-disable-next-line unused-return
                _validators.add(validator);
                validatorNetAddresses[validator] = netAddr;
                validatorWorkerAddresses[validator] = workerAddr;
            }
        }

        if (status == Status.Instantiated) {
            if (totalStake >= minActivationCollateral) {
                status = Status.Active;
                IGateway(ipcGatewayAddr).register{value: totalStake}();
            }
        } else {
            if (status == Status.Inactive) {
                if (totalStake >= minActivationCollateral) {
                    status = Status.Active;
                }
            }
            IGateway(ipcGatewayAddr).addStake{value: validatorStake}();
        }
    }

    /// @notice method that allows a validator to leave the subnet
    function leave() external nonReentrant signableOnly notKilled {
        uint256 amount = stake[msg.sender];

        if (amount == 0) {
            revert NotValidator();
        }

        stake[msg.sender] = 0;
        totalStake -= amount;
        // slither-disable-next-line unused-return
        _validators.remove(msg.sender);
        if (status == Status.Active) {
            if (totalStake < minActivationCollateral) {
                status = Status.Inactive;
            }
        }

        IGateway(ipcGatewayAddr).releaseStake(amount);

        payable(msg.sender).sendValue(amount);
    }

    /// @notice method that allows the subnet no be killed after all validators leave
    function kill() external signableOnly notKilled {
        if (_validators.length() != 0 || totalStake != 0) {
            revert NotAllValidatorsHaveLeft();
        }

        status = Status.Killed;

        IGateway(ipcGatewayAddr).kill();
    }

    /// @notice methods that allows a validator to submit a checkpoint (batch of messages) and vote for it with it's own voting power.
    /// @param checkpoint - the batch messages data
    function submitCheckpoint(
        BottomUpCheckpoint calldata checkpoint
    ) external signableOnly validEpochOnly(checkpoint.epoch) {
        if (status != Status.Active) {
            revert SubnetNotActive();
        }
        if (!_validators.contains(msg.sender)) {
            revert NotValidator();
        }
        if (checkpoint.source.toHash() != currentSubnetHash) {
            revert WrongCheckpointSource();
        }
        if (!CrossMsgHelper.isSorted(checkpoint.crossMsgs)) {
            revert MessagesNotSorted();
        }

        EpochVoteBottomUpSubmission storage voteSubmission = _epochVoteSubmissions[checkpoint.epoch];

        // submit the vote
        bool shouldExecuteVote = _submitBottomUpVote(voteSubmission, checkpoint, msg.sender, stake[msg.sender]);

        if (shouldExecuteVote) {
            _commitCheckpoint(voteSubmission);
        } else {
            // try to get the next executable epoch from the queue
            (uint64 nextExecutableEpoch, bool isExecutableEpoch) = _getNextExecutableEpoch();

            if (isExecutableEpoch) {
                EpochVoteBottomUpSubmission storage nextVoteSubmission = _epochVoteSubmissions[nextExecutableEpoch];

                _commitCheckpoint(nextVoteSubmission);
            }
        }
    }

    /// @notice method that distributes the rewards for the subnet to validators.
    function reward(uint256 amount) external onlyGateway {
        uint256 validatorsLength = _validators.length();

        if (validatorsLength == 0) {
            revert NoValidatorsInSubnet();
        }
        if (amount < validatorsLength) {
            revert NotEnoughBalanceForRewards();
        }

        uint256 rewardAmount = amount / validatorsLength;

        for (uint256 i = 0; i < validatorsLength; ) {
            accumulatedRewards[_validators.at(i)] += rewardAmount;
            unchecked {
                ++i;
            }
        }
    }

    /// @notice method that allows a validator to withdraw it's accumulated rewards using pull-based transfer
    function withdraw() external signableOnly {
        uint256 amount = accumulatedRewards[msg.sender];

        if (amount == 0) {
            revert NoRewardToWithdraw();
        }

        accumulatedRewards[msg.sender] = 0;

        IGateway(ipcGatewayAddr).releaseRewards(amount);

        payable(msg.sender).sendValue(amount);
    }

    /// @notice get the parent subnet id
    function getParent() external view returns (SubnetID memory) {
        return _parentId;
    }

    /// @notice get validator count
    function validatorCount() external view returns (uint256) {
        return _validators.length();
    }

    /// @notice get validator at index
    /// @param index - the index of the validator set
    function validatorAt(uint256 index) external view returns (address) {
        return _validators.at(index);
    }

    /// @notice get all the validators in the subnet.
    /// TODO: we can introduce pagination
    function getValidators() external view returns (address[] memory) {
        uint256 length = _validators.length();
        address[] memory result = new address[](length);

        for (uint256 i = 0; i < length; ) {
            result[i] = _validators.at(i);
            unchecked {
                ++i;
            }
        }

        return result;
    }

    /// @notice whether a validator has voted for a checkpoint submission during an epoch
    /// @param epoch - the epoch to check
    /// @param submitter - the validator to check
    function hasValidatorVotedForSubmission(uint64 epoch, address submitter) external view returns (bool) {
        EpochVoteBottomUpSubmission storage voteSubmission = _epochVoteSubmissions[epoch];

        return voteSubmission.vote.submitters[voteSubmission.vote.nonce][submitter];
    }

    /// @notice returns the committed bottom-up checkpoint at specific epoch
    /// @param epoch - the epoch to check
    /// @return exists - whether the checkpoint exists
    /// @return checkpoint - the checkpoint struct
    function bottomUpCheckpointAtEpoch(
        uint64 epoch
    ) public view returns (bool exists, BottomUpCheckpoint memory checkpoint) {
        checkpoint = committedCheckpoints[epoch];
        exists = !checkpoint.source.isEmpty();
    }

    /// @notice returns the historical committed bottom-up checkpoint hash
    /// @param epoch - the epoch to check
    /// @return exists - whether the checkpoint exists
    /// @return hash - the hash of the checkpoint
    function bottomUpCheckpointHashAtEpoch(uint64 epoch) external view returns (bool, bytes32) {
        (bool exists, BottomUpCheckpoint memory checkpoint) = bottomUpCheckpointAtEpoch(epoch);
        return (exists, checkpoint.toHash());
    }

    /// @notice submits a vote for a checkpoint
    /// @param voteSubmission - the vote submission data
    /// @param submitterAddress - the validator that submits the vote
    /// @param submitterWeight - the weight of the validator
    function _submitBottomUpVote(
        EpochVoteBottomUpSubmission storage voteSubmission,
        BottomUpCheckpoint calldata submission,
        address submitterAddress,
        uint256 submitterWeight
    ) internal returns (bool shouldExecuteVote) {
        bytes32 submissionHash = submission.toHash();

        shouldExecuteVote = _submitVote(
            voteSubmission.vote,
            submissionHash,
            submitterAddress,
            submitterWeight,
            submission.epoch,
            totalStake
        );

        // store the submission only the first time
        if (voteSubmission.submissions[submissionHash].isEmpty()) {
            voteSubmission.submissions[submissionHash] = submission;
        }
    }

    /// @notice method that commits a checkpoint after reaching majority
    /// @param voteSubmission - the last vote submission that reached majority for commit
    function _commitCheckpoint(EpochVoteBottomUpSubmission storage voteSubmission) internal {
        BottomUpCheckpoint storage checkpoint = voteSubmission.submissions[voteSubmission.vote.mostVotedSubmission];
        /// Ensures the checkpoints are chained. If not, should abort the current checkpoint.

        if (prevExecutedCheckpointHash != checkpoint.prevHash) {
            voteSubmission.vote.reset();
            executableQueue.remove(checkpoint.epoch);
            return;
        }

        _markSubmissionExecuted(checkpoint.epoch);

        committedCheckpoints[checkpoint.epoch] = checkpoint;
        prevExecutedCheckpointHash = checkpoint.toHash();

        IGateway(ipcGatewayAddr).commitChildCheck(checkpoint);
    }
}
