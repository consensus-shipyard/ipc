// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity ^0.8.23;

import {InvalidXnetMessage, InvalidXnetMessageReason, DuplicatedGenesisValidator, WrongSubnet, InvalidFederationPayload, NotEnoughGenesisValidators} from "../errors/IPCErrors.sol";
import {LibPowerChange, LibPowerChangeStorage, ProofOfPower, LibPowerQuery} from "../lib/power/LibPower.sol";
import {ReentrancyGuard} from "../lib/LibReentrancyGuard.sol";
import {Pausable} from "../lib/LibPausable.sol";
import {LibDiamond} from "../lib/LibDiamond.sol";
import {ISubnet} from "../interfaces/ISubnet.sol";
import {LibSubnetActor} from "./SubnetActorFacet.sol";
import {IGenesisComponent} from "../interfaces/IGenesis.sol";

library LibFederatedPower {
    // The federated power storage
    struct FederatedPowerStorage {
        uint64 minValidators;
        /// @notice If the federated power mode is bootstrapped
        bool bootstrapped;
    }

    function diamondStorage() internal pure returns (FederatedPowerStorage storage ds) {
        bytes32 position = keccak256("ipc.subnet.federated.storage");
        assembly {
            ds.slot := position
        }
    }
}

contract FederatedSubnetFacet is IGenesisComponent, ReentrancyGuard, Pausable {
    using LibPowerQuery for ProofOfPower;
    using LibPowerChange for ProofOfPower;

    event FederatedPowerBootstrapped();

    // ============== Genesis related =============
    /// @notice Returns the id of the component
    function id() external view returns(bytes4) {
        return bytes4(keccak256("federated-power"));
    }

    /// @notice Returns the actual bytes of the genesis
    function genesis() external view returns(bytes memory) {
        require(false, "todo");
    }

    /// @notice Checks if the component is bootstrapped
    function bootstrapped() external view returns(bool) {
        return LibFederatedPower.diamondStorage().bootstrapped;
    }

    // ============== Federated power related ===========
    function setPower(
        address[] calldata validators,
        bytes[] calldata publicKeys,
        uint256[] calldata powers
    ) external {
        if (validators.length != powers.length) {
            revert InvalidFederationPayload();
        }

        if (validators.length != publicKeys.length) {
            revert InvalidFederationPayload();
        }

        LibPowerChange.validatePublicKeys(validators, publicKeys);

        // only subnet owner is allowed to set powers
        LibDiamond.enforceIsContractOwner();

        LibFederatedPower.FederatedPowerStorage storage fps = LibFederatedPower.diamondStorage();
        if (!fps.bootstrapped) {
            preBootstrap(validators, publicKeys, powers);
        } else {
            postBootstrap(validators, publicKeys, powers);
        }
    }

    // ===== Getters =====
    function confimedPower(address addr) external view returns(uint256) {
        return LibPowerChangeStorage.diamondStorage().getConfirmedPower(addr);
    }

    function unconfirmedPower(address addr) external view returns(uint256) {
        return LibPowerChangeStorage.diamondStorage().getUnconfirmedPower(addr);
    }

    // ======= Internal functions ======
    function preBootstrap(
        address[] calldata validators,
        bytes[] calldata publicKeys,
        uint256[] calldata powers
    ) internal {
        uint256 length = validators.length;

        LibFederatedPower.FederatedPowerStorage storage fps = LibFederatedPower.diamondStorage();
        ProofOfPower storage proofS = LibPowerChangeStorage.diamondStorage();

        if (length <= fps.minValidators) {
            revert NotEnoughGenesisValidators();
        }

        for (uint256 i; i < length; ) {
            // performing deduplication
            // validator should have no power when first added
            if (proofS.getConfirmedPower(validators[i]) > 0) {
                revert DuplicatedGenesisValidator();
            }

            proofS.confirmMetadata(validators[i], publicKeys[i]);
            proofS.confirmNewPower(validators[i], powers[i]);

            // s.genesisValidators.push(Validator({addr: validators[i], weight: powers[i], metadata: publicKeys[i]}));

            unchecked {
                ++i;
            }
        }

        fps.bootstrapped = true;
        // emit FederatedPowerBootstrapped(s.genesisValidators);

        // TODO: register with the gateway
    }

    function postBootstrap(
        address[] calldata validators,
        bytes[] calldata publicKeys,
        uint256[] calldata powers
    ) internal {
        uint256 length = validators.length;
        ProofOfPower storage proofS = LibPowerChangeStorage.diamondStorage();

        for (uint256 i; i < length; ) {
            proofS.setValidatorMetadata(validators[i], publicKeys[i]);
            proofS.setNewPower(validators[i], powers[i]);

            unchecked {
                ++i;
            }
        }
    }
}
