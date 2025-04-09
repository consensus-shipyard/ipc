// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity ^0.8.23;

error AddressShouldBeValidator();
error AlreadyRegisteredSubnet();
error AlreadyInSet();
error CannotConfirmFutureChanges();
error CannotReleaseZero();
error CheckpointAlreadyExists();
error BatchAlreadyExists();
error MaxMsgsPerBatchExceeded();
error QuorumAlreadyProcessed();
error CheckpointNotCreated();
error BottomUpCheckpointAlreadySubmitted();
error BatchNotCreated();
error CollateralIsZero();
error EmptyAddress();
error FailedAddIncompleteQuorum();
error FailedAddSignatory();
error FailedRemoveIncompleteQuorum();
error GatewayCannotBeZero();
error InvalidActorAddress();
error InvalidCheckpointEpoch();
error CannotSubmitFutureCheckpoint();
error InvalidBatchEpoch();
error InvalidCheckpointSource();
error InvalidBatchSource();
error InvalidSubnetActor();
error InvalidCollateral();
error InvalidConfigurationNumber();
error InvalidXnetMessage(InvalidXnetMessageReason reason);
error InvalidMajorityPercentage();
error InvalidPowerScale();
error InvalidRetentionHeight();
error InvalidSignature();
error InvalidSignatureErr(uint8);
error InvalidSignatureLength();
error InvalidPublicKeyLength();
error InvalidSubmissionPeriod();
error InvalidSubnet();
error NoCollateralToWithdraw();
error NoValidatorsInSubnet();
error NotAllValidatorsHaveLeft();
error NotAuthorized(address);
error NotEmptySubnetCircSupply();
error NotEnoughBalance();
error NotEnoughBalanceForRewards();
error NotEnoughCollateral();
error NotEnoughFunds();
error NotEnoughFundsToRelease();
error NotEnoughSubnetCircSupply();
error NotEnoughValidatorsInSubnet();
error NotGateway();
error NotInSet();
error NotOwnerOfPublicKey();
error NotRegisteredSubnet();
error NotStakedBefore();
error NotSystemActor();
error NotValidator(address);
error OldConfigurationNumber();
error PQDoesNotContainAddress();
error PQEmpty();
error ParentFinalityAlreadyCommitted();
error PostboxNotExist();
error SignatureReplay();
error SubnetAlreadyKilled();
error SubnetNotActive();
error SubnetNotFound();
error PowerReductionMoreThanTotal(uint256 total, uint256 change);
error ZeroMembershipWeight();
error SubnetAlreadyBootstrapped();
error SubnetNotBootstrapped();
error FacetCannotBeZero();
error WrongGateway();
error CannotFindSubnet();
error UnknownSubnet();
error MethodNotAllowed(string reason);
error InvalidFederationPayload();
error DuplicatedGenesisValidator();
error NotEnoughGenesisValidators();
error ValidatorPowerChangeDenied();
error MissingActivityCommitment();
error ValidatorAlreadyClaimed();
error InvalidActivityProof();
error NotOwner();
error HasAlreadyVoted(address validator);
error ExpectingLivenessVote(uint64 expectedLivenessHeight, uint64 actualVoteHeight);
error InvalidLivenssSubmissionHeight(uint64 expectedLivenessHeight, uint64 actualVoteHeight);
error VoteNotProposed(bytes32 vote);
// topdown checkpoint
error InvalidTopdownCheckpointHeight(uint64 proposed, uint64 previous);
error InvalidTopdownMessageNonce(uint64 expected, uint64 actual);
error InvalidTopdownConfigNumber(uint64 expected, uint64 actual);
error NonSequential(string what, uint64 previous, uint64 current);

enum InvalidXnetMessageReason {
    Sender,
    DstSubnet,
    Nonce,
    Value,
    Kind,
    ReflexiveSend,
    NoRoute,
    IncompatibleSupplySource
}

string constant ERR_PERMISSIONED_AND_BOOTSTRAPPED = "Method not allowed if permissioned is enabled and subnet bootstrapped";
string constant ERR_VALIDATOR_JOINED = "Method not allowed if validator has already joined";
string constant ERR_VALIDATOR_NOT_JOINED = "Method not allowed if validator has not joined";
