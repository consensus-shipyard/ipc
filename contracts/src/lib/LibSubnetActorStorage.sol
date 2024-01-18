// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity 0.8.19;

import {ConsensusType} from "../enums/ConsensusType.sol";
import {NotGateway, SubnetAlreadyKilled} from "../errors/IPCErrors.sol";
import {RelayerRewardsInfo, BottomUpCheckpoint, BottomUpMsgBatchInfo} from "../structs/CrossNet.sol";
import {SubnetID, ValidatorSet, StakingChangeLog, StakingReleaseQueue, SupplySource, Validator, PermissionMode} from "../structs/Subnet.sol";
import {EnumerableSet} from "openzeppelin-contracts/utils/structs/EnumerableSet.sol";

struct SubnetActorStorage {
    /// @notice contains all committed bottom-up checkpoint at specific epoch
    mapping(uint256 => BottomUpCheckpoint) committedCheckpoints;
    /// @notice initial set of validators joining in genesis
    Validator[] genesisValidators;
    /// @notice initial circulating supply provided by genesis validators to use when bootstrapping
    /// the network.
    uint256 genesisCircSupply;
    /// @notice genesis balance to be allocated to the subnet in genesis.
    mapping(address => uint256) genesisBalance;
    /// @notice genesis balance addresses
    address[] genesisBalanceKeys;
    /// @notice The height of the last committed bottom-up checkpoint.
    uint256 lastBottomUpCheckpointHeight;
    /// @notice Maximum number of messages per batch
    uint64 maxMsgsPerBottomUpBatch;
    /// @notice Minimal activation collateral
    uint256 minActivationCollateral;
    /// @notice number of blocks in a bottom-up epoch
    uint256 bottomUpCheckPeriod;
    /// @notice Minimal number of validators required for the subnet to be able to validate new blocks.
    uint64 minValidators;
    // @notice Hash of the current subnet id
    bytes32 currentSubnetHash;
    /// @notice Address of the IPC gateway for the subnet
    address ipcGatewayAddr;
    /// @notice majority percentage value (must be greater than or equal to 51)
    uint8 majorityPercentage;
    /// @notice minimum fee amount charged per cross message by the subnet
    uint256 minCrossMsgFee;
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
    /// @notice Power scale determining the accuracy of the power scale (in number of decimals from whole FIL)
    /// (e.g. Fil = 0, miliFil = 3; microFIL = 6, attoFil = 18, etc.)
    /// We allow negative values to also allow 10 FIL = 1 unit of power for power_scale = -1.
    int8 powerScale;
    /// @notice relayers rewards
    RelayerRewardsInfo relayerRewards;
    /// =============
    /// mapping of bootstrap owner to its bootstrap node address
    mapping(address => string) bootstrapNodes;
    /// @notice the list ov validators that announces bootstrap nodes
    EnumerableSet.AddressSet bootstrapOwners;
    /// @notice subnet supply strategy.
    SupplySource supplySource;
}

library LibSubnetActorStorage {
    function appStorage() internal pure returns (SubnetActorStorage storage ds) {
        assembly {
            ds.slot := 0
        }
        return ds;
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
