// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity ^0.8.23;

import {ParentFinality} from "../structs/CrossNet.sol";
import {LibUtil} from "../lib/LibUtil.sol";

import {ParentFinalityAlreadyCommitted} from "../errors/IPCErrors.sol";
import {ProofOfPower, Validator, LibPowerQuery, LibPowerTracking, PowerChangeRequest} from "../lib/power/LibPower.sol";

contract GatewayTopDownFacet {
    /// @notice commit the ipc parent finality into storage and returns the previous committed finality
    /// This is useful to understand if the finalities are consistent or if there have been reorgs.
    /// If there are no previous committed fainality, it will be default to zero values, i.e. zero height and block hash.
    /// @param finality - the parent finality
    /// @return hasCommittedBefore A flag that indicates if a finality record has been committed before.
    /// @return previousFinality The previous finality information.
    function commitParentFinality(
        ParentFinality calldata finality
    ) external returns (bool hasCommittedBefore, ParentFinality memory previousFinality) {
        LibUtil.enforceSystemActorOnly();

        previousFinality = LibTopDown.commitParentFinality(finality);
        hasCommittedBefore = previousFinality.height != 0;
    }

    /// @notice Store the validator change requests from parent.
    /// @param changeRequests - the validator changes
    function storeValidatorChanges(PowerChangeRequest[] calldata changeRequests) external {
        LibUtil.enforceSystemActorOnly();
        LibTopDown.storeValidatorChanges(changeRequests);
    }

    /// @notice Apply all changes committed through the commitment of parent finality.
    /// @return configurationNumber The configuration number of the changes set that has been confirmed.
    function applyFinalityChanges() external returns (uint64) {
        LibUtil.enforceSystemActorOnly();
        return LibTopDown.applyFinalityChanges();
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