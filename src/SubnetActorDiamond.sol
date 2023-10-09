// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity 0.8.19;

import {SubnetActorStorage} from "./lib/LibSubnetActorStorage.sol";
import {ConsensusType} from "./enums/ConsensusType.sol";
import {IDiamond} from "./interfaces/IDiamond.sol";
import {GatewayCannotBeZero, NotGateway, InvalidSubmissionPeriod, InvalidCollateral, InvalidMajorityPercentage} from "./errors/IPCErrors.sol";
import {LibDiamond} from "./lib/LibDiamond.sol";
import {SubnetID} from "./structs/Subnet.sol";
import {SubnetIDHelper} from "./lib/SubnetIDHelper.sol";
import {LibStaking} from "./lib/LibStaking.sol";

error FunctionNotFound(bytes4 _functionSelector);

contract SubnetActorDiamond {
    SubnetActorStorage internal s;

    using SubnetIDHelper for SubnetID;

    struct ConstructorParams {
        SubnetID parentId;
        bytes32 name;
        address ipcGatewayAddr;
        ConsensusType consensus;
        uint256 minActivationCollateral;
        uint64 minValidators;
        uint64 bottomUpCheckPeriod;
        uint8 majorityPercentage;
        uint16 activeValidatorsLimit;
    }

    constructor(IDiamond.FacetCut[] memory _diamondCut, ConstructorParams memory params) {
        if (params.ipcGatewayAddr == address(0)) {
            revert GatewayCannotBeZero();
        }
        // topDownCheckPeriod can be equal 0, since validators can propose anything they want.
        // The bottomUpCheckPeriod should be non-zero for now.
        if (params.bottomUpCheckPeriod == 0) {
            revert InvalidSubmissionPeriod();
        }
        if (params.minActivationCollateral == 0) {
            revert InvalidCollateral();
        }
        if (params.majorityPercentage < 51 || params.majorityPercentage > 100) {
            revert InvalidMajorityPercentage();
        }

        LibDiamond.setContractOwner(msg.sender);
        LibDiamond.diamondCut({_diamondCut: _diamondCut, _init: address(0), _calldata: new bytes(0)});

        s.parentId = params.parentId;
        s.name = params.name;
        s.ipcGatewayAddr = params.ipcGatewayAddr;
        s.consensus = params.consensus;
        s.minActivationCollateral = params.minActivationCollateral;
        s.minValidators = params.minValidators;
        s.bottomUpCheckPeriod = params.bottomUpCheckPeriod;
        s.majorityPercentage = params.majorityPercentage;
        s.currentSubnetHash = s.parentId.createSubnetId(address(this)).toHash();

        s.validatorSet.activeLimit = params.activeValidatorsLimit;
        // Start the next configuration number from 1, 0 is reserved for no change and the genesis membership
        s.changeSet.nextConfigurationNumber = LibStaking.INITIAL_CONFIGURATION_NUMBER;
        // The startConfiguration number is also 1 to match with nextConfigurationNumber, indicating we have
        // empty validator change logs
        s.changeSet.startConfigurationNumber = LibStaking.INITIAL_CONFIGURATION_NUMBER;
    }

    function _fallback() internal {
        LibDiamond.DiamondStorage storage ds;
        bytes32 position = LibDiamond.DIAMOND_STORAGE_POSITION;
        // get diamond storage
        // slither-disable-next-line assembly
        assembly {
            ds.slot := position
        }
        // get facet from function selector
        address facet = ds.facetAddressAndSelectorPosition[msg.sig].facetAddress;
        if (facet == address(0)) {
            revert FunctionNotFound(msg.sig);
        }
        // Execute external function from facet using delegatecall and return any value.
        // slither-disable-next-line assembly
        assembly {
            // copy function selector and any arguments
            calldatacopy(0, 0, calldatasize())
            // execute function call using the facet
            let result := delegatecall(gas(), facet, 0, calldatasize(), 0, 0)
            // get any return value
            returndatacopy(0, 0, returndatasize())
            // return any return value or error back to the caller
            switch result
            case 0 {
                revert(0, returndatasize())
            }
            default {
                return(0, returndatasize())
            }
        }
    }

    /// @notice Will run when no functions matches call data
    fallback() external payable {
        _fallback();
    }

    /// @notice Same as fallback but called when calldata is empty
    /* solhint-disable no-empty-blocks */
    receive() external payable onlyGateway {}

    /* solhint-enable no-empty-blocks */

    function _onlyGateway() private view {
        if (msg.sender != s.ipcGatewayAddr) {
            revert NotGateway();
        }
    }

    modifier onlyGateway() {
        _onlyGateway();
        _;
    }
}
