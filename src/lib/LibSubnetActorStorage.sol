// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity 0.8.19;

import {ConsensusType} from "../enums/ConsensusType.sol";
import {NotGateway, SubnetAlreadyKilled} from "../errors/IPCErrors.sol";
import {FvmAddress} from "../structs/FvmAddress.sol";
import {BottomUpCheckpoint} from "../structs/Checkpoint.sol";
import {SubnetID, ValidatorSet, StakingChangeLog, StakingReleaseQueue} from "../structs/Subnet.sol";
import {Address} from "openzeppelin-contracts/utils/Address.sol";
import {EnumerableSet} from "openzeppelin-contracts/utils/structs/EnumerableSet.sol";

struct SubnetActorStorage {
    /// @notice contains all committed bottom-up checkpoint at specific epoch
    mapping(uint64 => BottomUpCheckpoint) committedCheckpoints;
    /// @notice The height of the last committed bottom-up checkpoint.
    uint64 lastBottomUpCheckpointHeight;
    /// @notice Minimal activation collateral
    uint256 minActivationCollateral;
    /// @notice number of blocks in a bottom-up epoch
    uint64 bottomUpCheckPeriod;
    /// @notice Minimal number of validators required for the subnet to be able to validate new blocks.
    uint64 minValidators;
    /// @notice Human-readable name of the subnet.
    bytes32 name;
    // @notice Hash of the current subnet id
    bytes32 currentSubnetHash;
    /// @notice Address of the IPC gateway for the subnet
    address ipcGatewayAddr;
    /// @notice majority percentage value (must be greater than or equal to 51)
    uint8 majorityPercentage;
    /// @notice ID of the parent subnet
    SubnetID parentId;
    /// immutable params
    ConsensusType consensus;
    /// @notice Determines if the subnet has been bootstrapped (i.e. it has been activated)
    bool bootstrapped;
    /// @notice Determines if the subnet has been successfully killed
    bool killed;
    // =========== Staking ===========
    /// @notice the list of validators staking
    ValidatorSet validatorSet;
    /// @notice Contains the list of changes to validator set. Configuration number is associated at each change.
    StakingChangeLog changeSet;
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
        if (s.killed) {
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
