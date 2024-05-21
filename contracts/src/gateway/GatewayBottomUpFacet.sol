// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity ^0.8.23;

import {ParentFinality} from "../structs/CrossNet.sol";
import {LibUtil} from "../lib/LibUtil.sol";
import {FvmAddress} from "../structs/FvmAddress.sol";

import {SubnetIDHelper} from "../lib/SubnetIDHelper.sol";
import {IpcEnvelope} from "../structs/CrossNet.sol";
import {Consensus} from "../enums/ConsensusType.sol";
import {CrossMsgHelper} from "../lib/CrossMsgHelper.sol";
import {FvmAddressHelper} from "../lib/FvmAddressHelper.sol";
import {LibSubnetActorQuery} from "../subnet/SubnetActorFacet.sol";

import {ParentFinalityAlreadyCommitted, InvalidXnetMessage, InvalidXnetMessageReason} from "../errors/IPCErrors.sol";
import {ProofOfPower, Validator, LibPowerQuery, LibPowerTracking, PowerChangeRequest} from "../lib/power/LibPower.sol";

/// @notice Handles requests coming 
contract GatewayBottomUpFacet {
    using FvmAddressHelper for FvmAddress;
    using LibSubnetGenesis for SubnetGenesis;
    using SubnetIDHelper for SubnetID;

    /// @notice release() burns the received value locally in subnet and commits a bottom-up message to release the assets in the parent.
    ///         The local supply of a subnet is always the native coin, so this method doesn't have to deal with tokens.
    function release(FvmAddress calldata to, uint256 amount) external payable {
        if (amount == 0) {
            // prevent spamming if there's no value to release.
            revert InvalidXnetMessage(InvalidXnetMessageReason.Value);
        }
        IpcEnvelope memory crossMsg = CrossMsgHelper.createReleaseMsg({
            subnet: s.networkName,
            signer: msg.sender,
            to: to,
            value: amount
        });

        LibGateway.commitBottomUpMsg(crossMsg);
        // burn funds that are being released
        payable(BURNT_FUNDS_ACTOR).sendValue(msg.value);
    }
}

// ============ Internal Usage Only ============

/// @notice Membership information stored in the gateway.
struct Membership {
    Validator[] validators;
    uint64 configurationNumber;
}

/// @notice Handles the request coming from the parent. This sits in the child network that handles topdown related
///         requests and updates.
library LibTopDown {
    using LibPowerTracking for ProofOfPower;
    using LibPowerQuery for ProofOfPower;

    /// @notice commit the ipc parent finality into storage
    /// @param finality - the finality to be committed
    function commitParentFinality(
        ParentFinality calldata finality
    ) internal returns (ParentFinality memory lastFinality) {
        TopdownStorage storage s = LibTopDownStorage.diamondStorage();

        uint256 lastHeight = s.latestParentHeight;
        if (lastHeight >= finality.height) {
            revert ParentFinalityAlreadyCommitted();
        }
        lastFinality = s.finalitiesMap[lastHeight];

        s.finalitiesMap[finality.height] = finality;
        s.latestParentHeight = finality.height;
    }

    /// @notice Store the validator change requests from parent.
    /// @param changeRequests - the validator changes
    function storeValidatorChanges(PowerChangeRequest[] calldata changeRequests) internal {
        TopdownStorage storage s = LibTopDownStorage.diamondStorage();
        s.validatorPowers.batchStoreChange(changeRequests);
    }

    /// @notice Apply all changes committed through the commitment of parent finality.
    /// @return configurationNumber The configuration number of the changes set that has been confirmed.
    function applyFinalityChanges() internal returns (uint64) {
        TopdownStorage storage s = LibTopDownStorage.diamondStorage();

        // get the latest configuration number for the change set
        uint64 configurationNumber = s.validatorPowers.changeSet.nextConfigurationNumber - 1;
        // return immediately if there are no changes to confirm by looking at next configNumber
        if (
            // nextConfiguration == startConfiguration (i.e. no changes)
            (configurationNumber + 1) == s.validatorPowers.changeSet.startConfigurationNumber
        ) {
            // 0 flags that there are no changes
            return 0;
        }

        // confirm the change
        s.validatorPowers.confirmChange(configurationNumber);

        return configurationNumber;
    }

    function currentMembership() internal returns(Membership memory membership) {
        TopdownStorage storage s = LibTopDownStorage.diamondStorage();

        // Get active validators and populate the new power table.
        address[] memory validators = s.validatorPowers.listActiveValidators();
        uint256 vLength = validators.length;
        Validator[] memory vs = new Validator[](vLength);
        for (uint256 i; i < vLength; ) {
            address addr = validators[i];
            vs[i] = s.validatorPowers.validators[addr];

            unchecked {
                ++i;
            }
        }

        uint64 configurationNumber = s.validatorPowers.changeSet.nextConfigurationNumber - 1;
        return Membership({configurationNumber: configurationNumber, validators: vs});
    }

}

// ============ Private Usage Only ============

struct TopdownStorage {
    /// @notice The latest parent height committed.
    uint256 latestParentHeight;
    /// @notice The parent finalities. Key is the block number, value is the finality struct.
    mapping(uint256 => ParentFinality) finalitiesMap;
    /// @notice Tracking the validator powers from the parent
    ProofOfPower validatorPowers;
}

library LibTopDownStorage {
    function diamondStorage() internal pure returns (TopdownStorage storage ds) {
        bytes32 position = keccak256("ipc.gateway.topdown.storage");
        assembly {
            ds.slot := position
        }
    }
}