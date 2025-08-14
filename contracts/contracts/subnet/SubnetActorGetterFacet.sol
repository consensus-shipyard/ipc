// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity ^0.8.23;

import {ConsensusType} from "../enums/ConsensusType.sol";
import {BottomUpCheckpoint, IpcEnvelope} from "../structs/CrossNet.sol";
import {SubnetID, Asset} from "../structs/Subnet.sol";
import {SubnetID, ValidatorInfo, Validator, PermissionMode} from "../structs/Subnet.sol";
import {SubnetActorStorage} from "../lib/LibSubnetActorStorage.sol";
import {SubnetIDHelper} from "../lib/SubnetIDHelper.sol";
import {Address} from "@openzeppelin/contracts/utils/Address.sol";
import {EnumerableSet} from "@openzeppelin/contracts/utils/structs/EnumerableSet.sol";
import {LibPower} from "../lib/LibPower.sol";
import {LibBottomUpBatch} from "../lib/LibBottomUpBatch.sol";

contract SubnetActorGetterFacet {
    using EnumerableSet for EnumerableSet.AddressSet;
    using SubnetIDHelper for SubnetID;
    using Address for address payable;

    // slither-disable-next-line uninitialized-state
    SubnetActorStorage internal s;

    /// @notice Returns the parent subnet id.
    function getParent() external view returns (SubnetID memory) {
        return s.parentId;
    }

    /// @notice Returns the permission mode.
    function permissionMode() external view returns (PermissionMode) {
        return s.validatorSet.permissionMode;
    }

    /// @notice Returns the gateway address.
    function ipcGatewayAddr() external view returns (address) {
        return s.ipcGatewayAddr;
    }

    /// @notice Returns the minimum validators number needed to activate the subnet.
    function minValidators() external view returns (uint64) {
        return s.minValidators;
    }

    /// @notice Returns the majority percentage required for consensus.
    function majorityPercentage() external view returns (uint8) {
        return s.majorityPercentage;
    }

    /// @notice Fetches the limit on the number of active validators.
    function activeValidatorsLimit() external view returns (uint16) {
        return s.validatorSet.activeLimit;
    }

    /// @notice Returns the next and start configuration numbers related to the changes.
    function getConfigurationNumbers() external view returns (uint64, uint64) {
        return (s.changeSet.nextConfigurationNumber, s.changeSet.startConfigurationNumber);
    }

    /// @notice Returns the initial set of validators of the genesis block.
    function genesisValidators() external view returns (Validator[] memory) {
        return s.genesisValidators;
    }

    // @notice Provides the circulating supply of the genesis block.
    function genesisCircSupply() external view returns (uint256) {
        return s.genesisCircSupply;
    }

    /// @notice Retrieves initial balances and corresponding addresses of the genesis block.
    function genesisBalances() external view returns (address[] memory, uint256[] memory) {
        uint256 numAddresses = s.genesisBalanceKeys.length;
        address[] memory addresses = new address[](numAddresses);
        uint256[] memory balances = new uint256[](numAddresses);

        for (uint256 i; i < numAddresses; ) {
            address addr = s.genesisBalanceKeys[i];
            addresses[i] = addr;
            balances[i] = s.genesisBalance[addr];

            unchecked {
                ++i;
            }
        }
        return (addresses, balances);
    }

    function chainID() external view returns (string memory) {
        return s.chainID;
    }

    /// @notice Returns the period for bottom-up checkpointing operations.
    function bottomUpCheckPeriod() external view returns (uint256) {
        return s.bottomUpCheckPeriod;
    }

    /// @notice Returns the block height of the last bottom-up checkpoint.
    function lastBottomUpCheckpointHeight() external view returns (uint256) {
        return s.lastBottomUpCheckpointHeight;
    }

    /// @notice Returns the consensus protocol type used in the subnet.
    function consensus() external view returns (ConsensusType) {
        return s.consensus;
    }

    /// @notice Checks if the subnet has been bootstrapped.
    function bootstrapped() external view returns (bool) {
        return s.bootstrapped;
    }

    /// @notice Checks if the subnet has been terminated or "killed".
    function killed() external view returns (bool) {
        return s.killed;
    }

    /// @notice Returns the minimum collateral required for subnet activation.
    function minActivationCollateral() external view returns (uint256) {
        return s.minActivationCollateral;
    }

    /// @notice Obtain the active validator address by its position index in the validator list array.
    function getActiveValidatorAddressByIndex(uint16 index) external view returns (address) {
        return LibPower.getActiveValidatorAddressByIndex(index);
    }

    /// @notice Returns detailed information about a specific validator.
    /// @param validatorAddress The address of the validator to query information for.
    function getValidator(address validatorAddress) external view returns (ValidatorInfo memory validator) {
        validator = s.validatorSet.validators[validatorAddress];
    }

    /// @notice Returns detailed information about all active validators.
    function getActiveValidators() external view returns (address[] memory) {
        return LibPower.listActiveValidators();
    }

    /// @notice Returns detailed information about all waiting validators.
    function getWaitingValidators() external view returns (address[] memory) {
        return LibPower.listWaitingValidators();
    }

    /// @notice Returns the total number of validators (active and waiting).
    function getTotalValidatorsNumber() external view returns (uint16) {
        return LibPower.totalValidators();
    }

    /// @notice Returns the number of active validators.
    function getActiveValidatorsNumber() external view returns (uint16) {
        return LibPower.totalActiveValidators();
    }

    /// @notice Returns the total amount of confirmed collateral across all validators.
    function getTotalCurrentPower() external view returns (uint256) {
        return LibPower.getTotalCurrentPower();
    }

    /// @notice Returns the total collateral amount for a specific validator.
    /// @param validator The address of the validator for which collateral is queried.
    function getTotalValidatorCollateral(address validator) external view returns (uint256) {
        return LibPower.totalValidatorCollateral(validator);
    }

    /// @notice Checks if the validator address is in an active state.
    /// @param validator The address of the checked validator
    function getCurrentPower(address validator) external view returns (uint256) {
        return LibPower.getCurrentPower(validator);
    }

    /// @notice Checks if the validator address is an active validator
    function isActiveValidator(address validator) external view returns (bool) {
        return LibPower.isActiveValidator(validator);
    }

    /// @notice Checks if the validator is in a waiting state.
    /// @param validator The address of the checked validator.
    function isWaitingValidator(address validator) external view returns (bool) {
        return LibPower.isWaitingValidator(validator);
    }

    /// @notice returns the committed bottom-up checkpoint at specific epoch.
    /// @param epoch - the epoch to check.
    /// @return exists - whether the checkpoint exists.
    /// @return checkpoint - the checkpoint struct.
    function bottomUpCheckpointAtEpoch(
        uint256 epoch
    ) public view returns (bool exists, BottomUpCheckpoint memory checkpoint) {
        checkpoint = s.committedCheckpoints[epoch];
        exists = !checkpoint.subnetID.isEmpty();
        return (exists, checkpoint);
    }

    /// @notice returns the historical committed bottom-up checkpoint hash.
    /// @param epoch - the epoch to check
    /// @return exists - whether the checkpoint exists
    /// @return hash - the hash of the checkpoint
    function bottomUpCheckpointHashAtEpoch(uint256 epoch) external view returns (bool, bytes32) {
        (bool exists, BottomUpCheckpoint memory checkpoint) = bottomUpCheckpointAtEpoch(epoch);
        return (exists, keccak256(abi.encode(checkpoint)));
    }

    /// @notice Returns the power scale in number of decimals from whole FIL.
    function powerScale() external view returns (int8) {
        return s.powerScale;
    }

    /// @notice Returns the bootstrap nodes addresses.
    function getBootstrapNodes() external view returns (string[] memory) {
        uint256 n = s.bootstrapOwners.length();
        string[] memory nodes = new string[](n);
        if (n == 0) {
            return nodes;
        }
        address[] memory owners = s.bootstrapOwners.values();
        for (uint256 i; i < n; ) {
            nodes[i] = s.bootstrapNodes[owners[i]];
            unchecked {
                ++i;
            }
        }
        return nodes;
    }

    /// @notice Computes a hash of an array of IpcEnvelopes.
    /// @dev This exists for testing purposes.
    /// @param messages An array of cross-chain envelopes to be hashed.
    /// @return The keccak256 hash of the encoded cross-chain messages.
    function crossMsgsHash(IpcEnvelope[] calldata messages) external pure returns (bytes32) {
        return keccak256(abi.encode(messages));
    }

    /// @notice Returns the supply strategy for the subnet.
    function supplySource() external view returns (Asset memory supply) {
        return s.supplySource;
    }

    /// @notice Returns the collateral asset kind for the subnet.
    function collateralSource() external view returns (Asset memory supply) {
        return s.collateralSource;
    }

    /// @notice Returns the genesis owner of the subnet IPC contracts
    function genesisSubnetIpcContractsOwner() external view returns (address) {
        return s.genesisSubnetIpcContractsOwner;
    }

    function listPendingBottomUpBatchCommitments()
        external
        view
        returns (LibBottomUpBatch.ListPendingCommitmentsEntry[] memory)
    {
        return LibBottomUpBatch.listPendingCommitments();
    }
}
