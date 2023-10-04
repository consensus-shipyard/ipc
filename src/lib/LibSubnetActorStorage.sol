// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity 0.8.19;

import {ConsensusType} from "../enums/ConsensusType.sol";
import {Status} from "../enums/Status.sol";
import {NotGateway, SubnetAlreadyKilled} from "../errors/IPCErrors.sol";
import {FvmAddress} from "../structs/FvmAddress.sol";
import {BottomUpCheckpoint} from "../structs/Checkpoint.sol";
import {SubnetID, ValidatorSet, StakingChangeSet, StakingReleaseQueue} from "../structs/Subnet.sol";
import {Address} from "openzeppelin-contracts/utils/Address.sol";
import {EnumerableSet} from "openzeppelin-contracts/utils/structs/EnumerableSet.sol";

struct SubnetActorStorage {
    /// @notice validator address to stake amount
    mapping(address => uint256) stake;
    /// @notice validator address to accumulated rewards
    mapping(address => uint256) accumulatedRewards;
    /// @notice validator address to validator net address
    mapping(address => string) validatorNetAddresses;
    /// @notice validator address to validator worker address
    mapping(address => FvmAddress) validatorWorkerAddresses;
    /// @notice contains all committed bottom-up checkpoint at specific epoch
    mapping(uint64 => BottomUpCheckpoint) committedCheckpoints;
    /// @notice genesis block
    bytes genesis;
    /// @notice Total collateral currently deposited in the GW from the subnet
    uint256 totalStake;
    /// @notice Minimal activation collateral
    uint256 minActivationCollateral;
    /// @notice Sequence number that uniquely identifies a validator set.
    uint64 configurationNumber;
    /// @notice number of blocks in a top-down epoch
    uint64 topDownCheckPeriod;
    /// @notice number of blocks in a bottom-up epoch
    uint64 bottomUpCheckPeriod;
    /// @notice Minimal number of validators required for the subnet to be able to validate new blocks.
    uint64 minValidators;
    /// @notice Human-readable name of the subnet.
    bytes32 name;
    // @notice Hash of the current subnet id
    bytes32 currentSubnetHash;
    /// @notice contains the last executed checkpoint hash
    bytes32 prevExecutedCheckpointHash;
    /// @notice Address of the IPC gateway for the subnet
    address ipcGatewayAddr;
    /// @notice majority percentage value (must be greater than or equal to 51)
    uint8 majorityPercentage;
    /// @notice Type of consensus algorithm.
    /// @notice current status of the subnet
    Status status;
    /// @notice List of validators in the subnet
    EnumerableSet.AddressSet validators;
    /// @notice ID of the parent subnet
    SubnetID parentId;
    /// immutable params
    ConsensusType consensus;
    // =========== Staking ===========
    /// @notice the list of validators staking
    ValidatorSet validatorSet;
    /// @notice Contains the list of changes to validator set. Configuration number is associated at each change.
    StakingChangeSet changeSet;
    /// @notice The staking release queue that only allow transfer of collateral after certain locking period.
    StakingReleaseQueue releaseQueue;
}

library LibSubnetActorStorage {
    function appStorage() internal pure returns (SubnetActorStorage storage ds) {
        assembly {
            ds.slot := 0
        }
    }
}

contract SubnetActorModifiers {
    SubnetActorStorage internal s;

    function _onlyGateway() private view {
        if (msg.sender != s.ipcGatewayAddr) {
            revert NotGateway();
        }
    }

    function _notKilled() private view {
        if (s.status == Status.Killed) {
            revert SubnetAlreadyKilled();
        }
    }

    modifier onlyGateway() {
        _onlyGateway();
        _;
    }

    modifier notKilled() {
        _notKilled();
        _;
    }
}
