// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity 0.8.19;

import {EpochVoteTopDownSubmission} from "../structs/EpochVoteSubmission.sol";
import {NotEnoughFee, NotSystemActor} from "../errors/IPCErrors.sol";
import {BottomUpCheckpoint, CrossMsg} from "../structs/Checkpoint.sol";
import {SubnetID, Subnet} from "../structs/Subnet.sol";
import {AccountHelper} from "../lib/AccountHelper.sol";
import {FilAddress} from "fevmate/utils/FilAddress.sol";

struct GatewayActorStorage {
    /// @notice List of subnets
    /// SubnetID => Subnet
    mapping(bytes32 => Subnet) subnets;
    /// @notice Postbox keeps track of all the cross-net messages triggered by
    /// an actor that need to be propagated further through the hierarchy.
    /// cross-net message id => CrossMsg
    mapping(bytes32 => CrossMsg) postbox;
    /// @notice BottomUpCheckpoints in the GW per epoch
    // slither-disable-next-line uninitialized-state
    mapping(uint64 => BottomUpCheckpoint) bottomUpCheckpoints;
    /// @notice List of validators and how many votes of the total each validator has for top-down messages
    // validatorNonce => validator => weight
    mapping(uint256 => mapping(address => uint256)) validatorSet;
    /// @notice epoch => SubnetID => [childIndex, exists(0 - no, 1 - yes)]
    mapping(uint64 => mapping(bytes32 => uint256[2])) children;
    /// @notice epoch => SubnetID => check => exists
    mapping(uint64 => mapping(bytes32 => mapping(bytes32 => bool))) checks;
    /// @notice contains voted submissions for a given epoch
    // slither-disable-next-line uninitialized-state
    mapping(uint64 => EpochVoteTopDownSubmission) epochVoteSubmissions;
    /// @notice Keys of the registered subnets. Useful to iterate through them
    bytes32[] subnetKeys;
    /// @notice path to the current network
    SubnetID networkName;
    /// @notice Minimum stake required to create a new subnet
    uint256 minStake;
    /// @notice sequence number that uniquely identifies a validator set
    uint256 validatorNonce;
    /// @notice fee amount charged per cross message
    uint256 crossMsgFee;
    /// @notice total votes of all validators
    uint256 totalWeight;
    /// @notice nonce for bottom-up messages
    uint64 bottomUpNonce;
    /// @notice AppliedNonces keep track of the next nonce of the message to be applied.
    /// This prevents potential replay attacks.
    uint64 appliedTopDownNonce;
    /// @notice top-down period in number of epochs for the subnet
    uint64 topDownCheckPeriod;
    /// @notice Number of active subnets spawned from this one
    uint64 totalSubnets;
    // @notice bottom-up period in number of epochs for the subnet
    uint64 bottomUpCheckPeriod;
    /// @notice whether the contract is initialized
    bool initialized;
}

library LibGatewayActorStorage {
    function appStorage() internal pure returns (GatewayActorStorage storage ds) {
        assembly {
            ds.slot := 0
        }
    }
}

contract GatewayActorModifiers {
    GatewayActorStorage internal s;

    using FilAddress for address;
    using FilAddress for address payable;
    using AccountHelper for address;

    function _hasFee() private view {
        if (msg.value < s.crossMsgFee) {
            revert NotEnoughFee();
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

    modifier hasFee() {
        _hasFee();
        _;
    }
}
