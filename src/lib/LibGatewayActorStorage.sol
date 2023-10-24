// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity 0.8.19;

import {NotEnoughFee, NotSystemActor, NotEnoughFunds} from "../errors/IPCErrors.sol";
import {BottomUpCheckpoint, CrossMsg, ParentFinality, CheckpointInfo} from "../structs/Checkpoint.sol";
import {SubnetID, Subnet, ParentValidatorsTracker} from "../structs/Subnet.sol";
import {Membership} from "../structs/Subnet.sol";
import {AccountHelper} from "../lib/AccountHelper.sol";
import {FilAddress} from "fevmate/utils/FilAddress.sol";
import {EnumerableSet} from "openzeppelin-contracts/utils/structs/EnumerableSet.sol";

struct GatewayActorStorage {
    /// @notice List of subnets
    /// SubnetID => Subnet
    mapping(bytes32 => Subnet) subnets;
    /// @notice a mapping of block number to top-down cross-messages
    /// SubnetID => blockNumber => messages
    mapping(bytes32 => mapping(uint256 => CrossMsg[])) topDownMsgs;
    /// @notice The parent finalities. Key is the block number, value is the finality struct.
    mapping(uint256 => ParentFinality) finalitiesMap;
    /// @notice The latest parent height committed.
    uint256 latestParentHeight;
    /// @notice Postbox keeps track of all the cross-net messages triggered by
    /// an actor that need to be propagated further through the hierarchy.
    /// cross-net message id => CrossMsg
    mapping(bytes32 => CrossMsg) postbox;
    /// @notice List of validators and how many votes of the total each validator has for top-down messages
    // configurationNumber => validator fvm address => weight
    mapping(uint64 => mapping(bytes32 => uint256)) validatorSetWeights;
    /// @notice The current membership of the child subnet
    Membership currentMembership;
    /// @notice The last membership received from the parent and adopted
    Membership lastMembership;
    /// @notice A mapping of block numbers to bottom-up checkpoints
    // slither-disable-next-line uninitialized-state
    mapping(uint64 => BottomUpCheckpoint) bottomUpCheckpoints;
    /// @notice A mapping of block numbers to checkpoint data
    // slither-disable-next-line uninitialized-state
    mapping(uint64 => CheckpointInfo) bottomUpCheckpointInfo;
    /// @notice A mapping of block numbers to bottom-up cross-messages
    // slither-disable-next-line uninitialized-state
    mapping(uint64 => CrossMsg[]) bottomUpMessages;
    /// @notice The height of the first bottom-up checkpoint that must be retained since they have not been processed in the parent.
    /// All checkpoint with the height less than this number may be garbage collected in the child subnet.
    /// @dev Initial retention index is 1.
    uint64 bottomUpCheckpointRetentionHeight;
    /// @notice A list of incomplete checkpoints.
    // slither-disable-next-line uninitialized-state
    EnumerableSet.UintSet incompleteCheckpoints;
    /// @notice The addresses of the validators that have already sent signatures at height `h`
    mapping(uint64 => EnumerableSet.AddressSet) bottomUpSignatureSenders;
    /// @notice The list of the collected signatures at height `h`
    mapping(uint64 => mapping(address => bytes)) bottomUpSignatures;
    /// @notice Keys of the registered subnets. Useful to iterate through them
    bytes32[] subnetKeys;
    /// @notice path to the current network
    SubnetID networkName;
    /// @notice Minimum stake required to create a new subnet
    uint256 minStake;
    /// @notice minimum fee amount charged per cross message
    uint256 minCrossMsgFee;
    /// @notice majority percentage value (must be greater than or equal to 51)
    uint8 majorityPercentage;
    /// @notice nonce for bottom-up messages
    uint64 bottomUpNonce;
    /// @notice AppliedNonces keep track of the next nonce of the message to be applied.
    /// This prevents potential replay attacks.
    uint64 appliedTopDownNonce;
    /// @notice Number of active subnets spawned from this one
    uint64 totalSubnets;
    // @notice bottom-up period in number of epochs for the subnet
    uint64 bottomUpCheckPeriod;
    /// Tracking validator changes from parent in child subnet
    ParentValidatorsTracker validatorsTracker;
}

library LibGatewayActorStorage {
    function appStorage() internal pure returns (GatewayActorStorage storage ds) {
        assembly {
            ds.slot := 0
        }
        return ds;
    }
}

contract GatewayActorModifiers {
    GatewayActorStorage internal s;

    using FilAddress for address;
    using FilAddress for address payable;
    using AccountHelper for address;

    function validateFee(uint256 fee) internal view {
        if (fee < s.minCrossMsgFee) {
            revert NotEnoughFee();
        }
        if (msg.value < fee) {
            revert NotEnoughFunds();
        }
    }

    function _systemActorOnly() private view {
        if (!msg.sender.isSystemActor()) {
            revert NotSystemActor();
        }
    }

    modifier systemActorOnly() {
        _systemActorOnly();
        _;
    }

    modifier validFee(uint256 fee) {
        validateFee(fee);
        _;
    }
}
