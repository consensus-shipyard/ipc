// SPDX-License-Identifier: UNLICENSED
pragma solidity 0.8.18;

import "./Voting.sol";
import "./enums/ConsensusType.sol";
import "./enums/Status.sol";
import "./enums/VoteExecutionStatus.sol";
import "./structs/Checkpoint.sol";
import "./structs/Subnet.sol";
import "./interfaces/ISubnetActor.sol";
import "./interfaces/IGateway.sol";
import "./lib/AccountHelper.sol";
import "./lib/CheckpointHelper.sol";
import "./lib/CrossMsgHelper.sol";
import "./lib/SubnetIDHelper.sol";
import "./lib/ExecutableQueueHelper.sol";
import "./lib/EpochVoteSubmissionHelper.sol";
import "openzeppelin-contracts/utils/structs/EnumerableSet.sol";
import "openzeppelin-contracts/security/ReentrancyGuard.sol";
import "openzeppelin-contracts/utils/Address.sol";

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

    /// @notice minimum collateral validators need to stake in order to join the subnet. Values get clamped to this
    uint256 private constant MIN_COLLATERAL_AMOUNT = 1 ether;

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

    /// @notice Address of the IPC gateway for the subnet
    address public ipcGatewayAddr;

    /// @notice current status of the subnet
    Status public status;

    /// @notice Type of consensus algorithm.
    ConsensusType public consensus;

    /// @notice contains the last executed checkpoint hash
    bytes32 public prevExecutedCheckpointHash;

    /// @notice contains all committed bottom-up checkpoint at specific epoch
    mapping(uint64 => BottomUpCheckpoint) public committedCheckpoints;

    /// @notice List of validators in the subnet
    EnumerableSet.AddressSet private validators;

    /// @notice contains voted submissions for a given epoch
    mapping(uint64 => EpochVoteBottomUpSubmission) private epochVoteSubmissions;

    /// @notice validator address to stake amount
    mapping(address => uint256) public stake;

    mapping(address => string) public validatorNetAddresses;

    /// @notice ID of the parent subnet
    SubnetID private parentId;

    /// @notice genesis block
    bytes public genesis;

    /// @notice Human-readable name of the subnet.
    bytes32 public name;

    error NotGateway();
    error NotAccount();
    error CollateralIsZero();
    error CallerHasNoStake();
    error CollateralStillLockedInSubnet();
    error SubnetAlreadyKilled();
    error NotAllValidatorsHaveLeft();
    error NotValidator();
    error SubnetNotActive();
    error WrongCheckpointSource();
    error CheckpointNotChained();
    error NoRewardsSentForDistribution();
    error NoValidatorsInSubnet();
    error NotEnoughBalanceForRewards();
    error MessagesNotSorted();

    modifier onlyGateway() {
        if (msg.sender != ipcGatewayAddr) revert NotGateway();
        _;
    }

    modifier signableOnly() {
        if (!msg.sender.isAccount()) revert NotAccount();
        _;
    }

    modifier notKilled() {
        if (status == Status.Terminating || status == Status.Killed) revert SubnetAlreadyKilled();
        _;
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
        parentId = params.parentId;
        name = params.name;
        ipcGatewayAddr = params.ipcGatewayAddr;
        consensus = params.consensus;
        minActivationCollateral = params.minActivationCollateral < MIN_COLLATERAL_AMOUNT
            ? MIN_COLLATERAL_AMOUNT
            : params.minActivationCollateral;
        minValidators = params.minValidators;
        topDownCheckPeriod =
            params.topDownCheckPeriod < MIN_CHECKPOINT_PERIOD ? MIN_CHECKPOINT_PERIOD : params.topDownCheckPeriod;
        bottomUpCheckPeriod = submissionPeriod;
        status = Status.Instantiated;
        genesis = params.genesis;
        // NOTE: we currently use 0 as the genesisEpoch for subnets so checkpoints
        // are submitted directly from epoch 0.
        // In the future we can use the current epoch. This will be really
        // useful once we support the docking of subnets to new parents, etc.
        genesisEpoch = 0;
    }

    receive() external payable onlyGateway {}

    /// @notice method that allows a validator to join the subnet
    /// @param netAddr - the network address of the validator
    function join(string calldata netAddr) external payable signableOnly notKilled {
        uint256 validatorStake = msg.value;
        address validator = msg.sender;
        if (validatorStake == 0) revert CollateralIsZero();

        stake[validator] += validatorStake;
        totalStake += validatorStake;

        if (
            stake[validator] >= minActivationCollateral && !validators.contains(validator)
                && (consensus != ConsensusType.Delegated || validators.length() == 0)
        ) {
            validators.add(validator);
            validatorNetAddresses[validator] = netAddr;
        }

        if (status == Status.Instantiated) {
            if (totalStake >= minActivationCollateral) {
                IGateway(ipcGatewayAddr).register{value: totalStake}();
                status = Status.Active;
            }
        } else {
            IGateway(ipcGatewayAddr).addStake{value: validatorStake}();
        }

        if(status == Status.Inactive && totalStake >= minActivationCollateral) {
            status = Status.Active;
        }

    }

    /// @notice method that allows a validator to leave the subnet
    function leave() external nonReentrant signableOnly notKilled {
        if (stake[msg.sender] == 0) revert NotValidator();

        uint256 amount = stake[msg.sender];

        stake[msg.sender] = 0;
        totalStake -= amount;
        validators.remove(msg.sender);

        IGateway(ipcGatewayAddr).releaseStake(amount);

        if (status == Status.Active && totalStake < minActivationCollateral) {
            status = Status.Inactive;
        }

        payable(msg.sender).sendValue(amount);
    }

    /// @notice method that allows the subnet no be killed after all validators leave
    function kill() external signableOnly notKilled {
        if (address(this).balance > 0) revert CollateralStillLockedInSubnet();
        if (validators.length() != 0 || totalStake != 0) revert NotAllValidatorsHaveLeft();

        status = Status.Terminating;

        IGateway(ipcGatewayAddr).kill();

        if (totalStake == 0) {
            status = Status.Killed;
        }
    }

    /// @notice methods that allows a validator to submit a checkpoint (batch of messages) and vote for it with it's own voting power.
    /// @param checkpoint - the batch messages data
    function submitCheckpoint(BottomUpCheckpoint calldata checkpoint)
        external
        signableOnly
        validEpochOnly(checkpoint.epoch)
    {
        if (status != Status.Active) revert SubnetNotActive();
        if (!validators.contains(msg.sender)) revert NotValidator();
        if (checkpoint.source.toHash() != parentId.createSubnetId(address(this)).toHash()) {
            revert WrongCheckpointSource();
        }
        if (!CrossMsgHelper.isSorted(checkpoint.crossMsgs)) revert MessagesNotSorted();

        EpochVoteBottomUpSubmission storage voteSubmission = epochVoteSubmissions[checkpoint.epoch];

        // submit the vote
        bool shouldExecuteVote = _submitBottomUpVote(voteSubmission, checkpoint, msg.sender, stake[msg.sender]);

        if (shouldExecuteVote) {
            _commitCheckpoint(voteSubmission);
        } else {
            // try to get the next executable epoch from the queue
            (uint64 nextExecutableEpoch, bool isExecutableEpoch) = _getNextExecutableEpoch();

            if (isExecutableEpoch) {
                EpochVoteBottomUpSubmission storage nextVoteSubmission = epochVoteSubmissions[nextExecutableEpoch];

                _commitCheckpoint(nextVoteSubmission);
            }
        }
    }

    /// @notice method that distributes the rewards for the subnet to validators.
    function reward() public payable onlyGateway nonReentrant {
        uint256 validatorsLength = validators.length();
        uint256 balance = address(this).balance;

        if (msg.value == 0) revert NoRewardsSentForDistribution();
        if (validatorsLength == 0) revert NoValidatorsInSubnet();
        if (balance < validatorsLength) revert NotEnoughBalanceForRewards();

        uint256 rewardAmount = balance / validatorsLength;

        for (uint256 i = 0; i < validatorsLength;) {
            payable(validators.at(i)).sendValue(rewardAmount);
            unchecked {
                ++i;
            }
        }
    }

    /// @notice get the parent subnet id
    function getParent() external view returns (SubnetID memory) {
        return parentId;
    }

    /// @notice get validator count
    function validatorCount() external view returns (uint256) {
        return validators.length();
    }

    /// @notice get validator at index
    /// @param index - the index of the validator set
    function validatorAt(uint256 index) external view returns (address) {
        return validators.at(index);
    }

    /// @notice wheather a validator has voted for a checkpoint submission during an epoch
    /// @param epoch - the epoch to check
    /// @param submitter - the validator to check
    function hasValidatorVotedForSubmission(uint64 epoch, address submitter) external view returns (bool) {
        EpochVoteBottomUpSubmission storage voteSubmission = epochVoteSubmissions[epoch];

        return voteSubmission.vote.submitters[voteSubmission.vote.nonce][submitter];
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
            voteSubmission.vote, submissionHash, submitterAddress, submitterWeight, submission.epoch, totalStake
        );

        // store the submission only the first time
        if (voteSubmission.submissions[submissionHash].isEmpty()) {
            voteSubmission.submissions[submissionHash] = submission;
        }
    }

    /// @notice method that returns the most voted submission for a checkpoint
    function _getMostVotedSubmission(EpochVoteBottomUpSubmission storage voteSubmission)
        internal
        view
        returns (BottomUpCheckpoint storage)
    {
        return voteSubmission.submissions[voteSubmission.vote.mostVotedSubmission];
    }

    /// @notice method that commits a checkpoint after reaching majority
    /// @param voteSubmission - the last vote submission that reached majority for commit
    function _commitCheckpoint(EpochVoteBottomUpSubmission storage voteSubmission) internal {
        BottomUpCheckpoint storage checkpoint = _getMostVotedSubmission(voteSubmission);

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
