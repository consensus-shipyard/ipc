// SPDX-License-Identifier: MIT
pragma solidity 0.8.23;

import {StrategyManager} from "./StrategyManager.sol";
import {DelegationManager} from "./DelegationManager.sol";

contract AVSDirectory {
    enum OperatorAVSRegistrationStatus {
        UNREGISTERED,
        REGISTERED
    }
    mapping(address => string) metadataURIs;
    DelegationManager public delegation;
    mapping(address => mapping(address => OperatorAVSRegistrationStatus))
        public avsOperatorStatus;

    constructor(DelegationManager _delegation) {
        delegation = _delegation;
    }

    function registerOperatorToAVS(address operator) external {
        require(
            avsOperatorStatus[msg.sender][operator] !=
                OperatorAVSRegistrationStatus.REGISTERED,
            "AVSDirectory: operator already registered"
        );
        require(
            delegation.isOperator(operator),
            "AVSDirectory.registerOperatorToAVS: operator not registered to EigenLayer yet"
        );
        avsOperatorStatus[msg.sender][operator] = OperatorAVSRegistrationStatus
            .REGISTERED;
    }

    function deregisterOperatorFromAVS(address operator) external {
        require(
            avsOperatorStatus[msg.sender][operator] ==
                OperatorAVSRegistrationStatus.REGISTERED,
            "AVSDirectory.deregisterOperatorFromAVS: operator not registered"
        );
        avsOperatorStatus[msg.sender][operator] = OperatorAVSRegistrationStatus
            .UNREGISTERED;
    }

    function updateAVSMetadataURI(string memory metadataURI) external {
        metadataURIs[msg.sender] = metadataURI;
    }
}
