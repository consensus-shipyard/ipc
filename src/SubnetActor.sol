// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.7;
import "./enums/ConsensusType.sol";
import "./enums/Status.sol";
import "./structs/Checkpoint.sol";
import "./structs/Subnet.sol";
import "./interfaces/ISubnetActor.sol";
import "./interfaces/IGateway.sol";
import "./lib/CheckpointMappingHelper.sol";
import "./lib/CheckpointHelper.sol";
import "./lib/SubnetIDHelper.sol";
import "openzeppelin-contracts/utils/structs/EnumerableSet.sol";
import "openzeppelin-contracts/security/ReentrancyGuard.sol";
import "openzeppelin-contracts/utils/Address.sol";

/// @title Subnet Actor Contract
/// @author LimeChain team
contract SubnetActor is ISubnetActor, ReentrancyGuard {
    using EnumerableSet for EnumerableSet.AddressSet;
    using SubnetIDHelper for SubnetID;
    using CheckpointHelper for Checkpoint;
    using CheckpointMappingHelper for mapping(int64 => Checkpoint);
    using Address for address payable;

    /// @notice Human-readable name of the subnet.
    string public name;
    /// @notice ID of the parent subnet
    SubnetID private parentId;
    /// @notice Address of the IPC gateway for the subnet
    address public ipcGatewayAddr;
    /// @notice Type of consensus algorithm.
    ConsensusType public consensus;
    /// @notice The minimum stake required to be a validator in this subnet
    uint256 public minValidatorStake;
    /// @notice Total collateral currently deposited in the SCA from the subnet
    uint256 public totalStake;
    /// @notice validator address to stake amount
    mapping(address => uint256) public stake;
    /// @notice current status of the subnet
    Status public status;
    /// @notice genesis block
    bytes public genesis;
    /// @notice number of blocks after which finality is reached
    int64 public finalityThreshold;
    /// @notice number of blocks between two checkpoints
    int64 public checkPeriod;
    /// @notice block number to corresponding checkpoint at that block
    mapping(int64 => Checkpoint) public checkpoints;
    /// @notice keccak256 hashed message data to set of validators who voted for the checkpoint
    mapping(bytes32 => EnumerableSet.AddressSet) private windowChecks;
    /// @notice List of validators in the subnet
    EnumerableSet.AddressSet private validators;
    /// @notice Minimal number of validators required for the subnet
    // to be able to validate new blocks.
    uint64 public minValidators;

    modifier onlyGateway() {
        require(
            msg.sender == ipcGatewayAddr,
            "only the IPC gateway can call this function"
        );
        _;
    }

    modifier mutateState() {
        _;
        if (status == Status.Instantiated && totalStake >= minValidatorStake) {
            status = Status.Active;
        } else if (status == Status.Active && totalStake < minValidatorStake) {
            status = Status.Inactive;
        } else if (
            status == Status.Inactive && totalStake >= minValidatorStake
        ) {
            status = Status.Active;
        } else if (status == Status.Terminating && totalStake == 0) {
            status = Status.Killed;
        }
    }

    constructor(
        SubnetID memory _parentId,
        string memory _name,
        address _ipcGatewayAddr,
        ConsensusType _consensus,
        uint256 _minValidatorStake,
        uint64 _minValidators,
        int64 _finalityThreshold,
        int64 _checkPeriod,
        bytes memory _genesis
    ) {
        require(
            _minValidatorStake > 0,
            "minValidatorStake must be greater than 0"
        );
        require(_minValidators > 0, "minValidators must be greater than 0");
        parentId = _parentId;
        name = _name;
        ipcGatewayAddr = _ipcGatewayAddr;
        consensus = _consensus;
        minValidatorStake = _minValidatorStake;
        minValidators = _minValidators;
        finalityThreshold = _finalityThreshold;
        checkPeriod = _checkPeriod;
        genesis = _genesis;
        status = Status.Instantiated;
    }

    receive() external payable onlyGateway {}

    function join() external payable mutateState {
        require(
            msg.value > 0,
            "a minimum collateral is required to join the subnet"
        );

        stake[msg.sender] += msg.value;
        totalStake += msg.value;
        if (
            stake[msg.sender] >= minValidatorStake &&
            !validators.contains(msg.sender) &&
            (consensus != ConsensusType.Delegated || validators.length() == 0)
        ) validators.add(msg.sender);

        if (status == Status.Instantiated) {
            if (totalStake >= minValidatorStake) {
                payable(ipcGatewayAddr).functionCallWithValue(
                    abi.encodeWithSignature("register()"),
                    totalStake
                );
            }
        } else {
            payable(ipcGatewayAddr).functionCallWithValue(
                abi.encodeWithSignature("addStake()"),
                msg.value
            );
        }
    }

    function leave() external mutateState nonReentrant {
        require(stake[msg.sender] != 0, "caller has no stake in subnet");

        uint256 amount = stake[msg.sender];

        stake[msg.sender] = 0;
        totalStake -= amount;
        validators.remove(msg.sender);

        if (status == Status.Terminating) return;

        IGateway(ipcGatewayAddr).releaseStake(amount);

        payable(msg.sender).sendValue(amount);
    }

    function kill() external mutateState {
        require(
            address(this).balance == 0,
            "there is still collateral in the subnet"
        );
        require(
            status != Status.Terminating && status != Status.Killed,
            "the subnet is already in a killed or terminating state"
        );
        require(
            validators.length() == 0 && totalStake == 0,
            "this subnet can only be killed when all validators have left"
        );

        status = Status.Terminating;

        IGateway(ipcGatewayAddr).kill();
    }

    function submitCheckpoint(Checkpoint calldata checkpoint) external {
        require(validators.contains(msg.sender), "not validator");
        require(
            status == Status.Active,
            "submitting checkpoints is not allowed while subnet is not active"
        );
        require(
            checkpoint.data.epoch % checkPeriod == 0,
            "epoch in checkpoint doesn't correspond with a signing window"
        );
        require(
            checkpoint.data.source.toHash() ==
                parentId.createSubnetId(address(this)).toHash(),
            "submitting checkpoint with the wrong source"
        );

        bytes32 prevHash = checkpoints.getPrevCheckpointHash(
            checkpoint.data.epoch,
            checkPeriod
        );

        require(
            checkpoint.data.prevHash == prevHash ||
                checkpoint.data.prevHash ==
                CheckpointHelper.EMPTY_CHECKPOINT_DATA_HASH,
            "checkpoint data hash is not the same as prevHash"
        );

        bytes32 messageHash = checkpoint.toHash();

        require(
            _recoverSigner(messageHash, checkpoint.signature) == msg.sender,
            "invalid signature"
        );

        EnumerableSet.AddressSet storage voters = windowChecks[messageHash];
        require(
            !voters.contains(msg.sender),
            "miner has already voted the checkpoint"
        );

        voters.add(msg.sender);

        uint sum = 0;
        for (uint i = 0; i < voters.length(); ) {
            sum += stake[voters.at(i)];
            unchecked {
                ++i;
            }
        }

        bool hasMajority = sum > (totalStake * 2 / 3);
        if (hasMajority == false) return;

        // store the commitment on vote majority
        require(
            checkpoints[checkpoint.data.epoch].signature.length == 0,
            "cannot submit checkpoint for epoch"
        );
        checkpoints[checkpoint.data.epoch] = checkpoint;
        //clear the votes
        address[] memory votersArray = voters.values();
        for (uint i = 0; i < votersArray.length; ) {
            (bool success) = voters.remove(votersArray[i]);
            require(success, "failed to remove voter");
            unchecked {
                ++i;
            }
        }

        IGateway(ipcGatewayAddr).commitChildCheck(checkpoint);
    }

    function reward() public payable onlyGateway nonReentrant {
        uint validatorLength = validators.length();
        require(validatorLength != 0, "no validators in subnet");
        require(
            address(this).balance >= validatorLength,
            "we need to distribute at least one wei to each validator"
        );

        uint rewardAmount = address(this).balance / validatorLength;

        for (uint i = 0; i < validatorLength; ) {
            payable(validators.at(i)).sendValue(rewardAmount);
            unchecked {
                ++i;
            }
        }
    }

    function getParent() external view returns (SubnetID memory) {
        return parentId;
    }

    function validatorCount() external view returns (uint) {
        return validators.length();
    }

    function validatorAt(uint index) external view returns (address) {
        return validators.at(index);
    }

    function windowCheckCount(bytes32 checkpointHash)
        external
        view
        returns (uint)
    {
        return windowChecks[checkpointHash].length();
    }

    function windowCheckAt(bytes32 checkpointHash, uint index)
        external
        view
        returns (address)
    {
        return windowChecks[checkpointHash].at(index);
    }

    function _recoverSigner(
        bytes32 _ethSignedMessageHash,
        bytes memory _signature
    ) internal pure returns (address) {
        (bytes32 r, bytes32 s, uint8 v) = _splitSignature(_signature);

        return ecrecover(_ethSignedMessageHash, v, r, s);
    }

    function _splitSignature(
        bytes memory sig
    ) internal pure returns (bytes32 r, bytes32 s, uint8 v) {
        require(sig.length == 65, "invalid signature length");

        assembly {
            /*
            First 32 bytes stores the length of the signature

            add(sig, 32) = pointer of sig + 32
            effectively, skips first 32 bytes of signature

            mload(p) loads next 32 bytes starting at the memory address p into memory
            */

            // first 32 bytes, after the length prefix
            r := mload(add(sig, 32))
            // second 32 bytes
            s := mload(add(sig, 64))
            // final byte (first byte of the next 32 bytes)
            v := byte(0, mload(add(sig, 96)))
        }

        // implicitly return (r, s, v)
    }
}
