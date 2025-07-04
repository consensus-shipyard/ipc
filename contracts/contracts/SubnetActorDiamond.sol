// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity ^0.8.23;

import {SubnetActorStorage} from "./lib/LibSubnetActorStorage.sol";
import {ConsensusType} from "./enums/ConsensusType.sol";
import {IDiamond} from "./interfaces/IDiamond.sol";
import {IDiamondCut} from "./interfaces/IDiamondCut.sol";
import {IDiamondLoupe} from "./interfaces/IDiamondLoupe.sol";
import {IERC165} from "./interfaces/IERC165.sol";
import {GatewayCannotBeZero, NotGateway, InvalidSubmissionPeriod, InvalidCollateral, InvalidMajorityPercentage, InvalidPowerScale, MissingGenesisSubnetIpcContractsOwner} from "./errors/IPCErrors.sol";
import {LibDiamond} from "./lib/LibDiamond.sol";
import {PermissionMode, SubnetID, AssetKind, Asset} from "./structs/Subnet.sol";
import {SubnetIDHelper} from "./lib/SubnetIDHelper.sol";
import {LibPower} from "./lib/LibPower.sol";
import {IERC20} from "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import {AssetHelper} from "./lib/AssetHelper.sol";
import {LibActivity} from "./lib/LibActivity.sol";

error FunctionNotFound(bytes4 _functionSelector);

contract SubnetActorDiamond {
    SubnetActorStorage internal s;

    using SubnetIDHelper for SubnetID;
    using AssetHelper for Asset;

    struct ConstructorParams {
        uint256 minActivationCollateral;
        uint64 minValidators;
        uint64 bottomUpCheckPeriod;
        address ipcGatewayAddr;
        uint16 activeValidatorsLimit;
        uint8 majorityPercentage;
        ConsensusType consensus;
        int8 powerScale;
        PermissionMode permissionMode;
        Asset supplySource;
        Asset collateralSource;
        SubnetID parentId;
        address validatorGater;
        address validatorRewarder;
        /// @notice Genesis address assigned as owner of all IPC diamond contracts deployed on this subnet (child) chain.
        /// @dev    This is only the initial (genesis) owner; ownership can be transferred or updated later via on-chain transaction.
        ///         The address lives on the subnet network and controls contract‐level administrative functions
        ///         (e.g. pausing, upgrading, facet management) for every IPC diamond contract within the subnet.
        address genesisSubnetIpcContractsOwner;
    }

    constructor(IDiamond.FacetCut[] memory _diamondCut, ConstructorParams memory params, address owner) {
        if (params.ipcGatewayAddr == address(0)) {
            revert GatewayCannotBeZero();
        }
        // The bottomUpCheckPeriod should be non-zero for now.
        if (params.bottomUpCheckPeriod == 0) {
            revert InvalidSubmissionPeriod();
        }
        if (params.permissionMode != PermissionMode.Federated && params.minActivationCollateral == 0) {
            revert InvalidCollateral();
        }
        if (params.majorityPercentage < 51 || params.majorityPercentage > 100) {
            revert InvalidMajorityPercentage();
        }
        if (params.powerScale > 18) {
            revert InvalidPowerScale();
        }
        if (params.genesisSubnetIpcContractsOwner == address(0)) {
            revert MissingGenesisSubnetIpcContractsOwner();
        }

        params.supplySource.validate();

        LibDiamond.setContractOwner(owner);
        LibDiamond.diamondCut({_diamondCut: _diamondCut, _init: address(0), _calldata: new bytes(0)});

        LibDiamond.DiamondStorage storage ds = LibDiamond.diamondStorage();
        // adding ERC165 data
        ds.supportedInterfaces[type(IERC165).interfaceId] = true;
        ds.supportedInterfaces[type(IDiamondCut).interfaceId] = true;
        ds.supportedInterfaces[type(IDiamondLoupe).interfaceId] = true;

        if (params.permissionMode == PermissionMode.Federated) {
            // ignore min activation collateral for now
            params.minActivationCollateral = 0;
        }

        s.parentId = params.parentId;
        s.ipcGatewayAddr = params.ipcGatewayAddr;
        s.consensus = params.consensus;
        s.minActivationCollateral = params.minActivationCollateral;
        s.minValidators = params.minValidators;
        s.bottomUpCheckPeriod = params.bottomUpCheckPeriod;
        s.majorityPercentage = params.majorityPercentage;
        s.powerScale = params.powerScale;
        s.currentSubnetHash = s.parentId.createSubnetId(address(this)).toHash();
        s.validatorSet.permissionMode = params.permissionMode;
        s.genesisSubnetIpcContractsOwner = params.genesisSubnetIpcContractsOwner;

        s.validatorSet.activeLimit = params.activeValidatorsLimit;
        // Start the next configuration number from 1, 0 is reserved for no change and the genesis membership
        s.changeSet.nextConfigurationNumber = LibPower.INITIAL_CONFIGURATION_NUMBER;
        // The startConfiguration number is also 1 to match with nextConfigurationNumber, indicating we have
        // empty validator change logs
        s.changeSet.startConfigurationNumber = LibPower.INITIAL_CONFIGURATION_NUMBER;
        // Set the supply strategy.
        s.supplySource = params.supplySource;
        s.collateralSource = params.collateralSource;

        if (params.validatorGater != address(0)) {
            s.validatorGater = params.validatorGater;
        }

        if (params.validatorRewarder != address(0)) {
            LibActivity.setRewarder(params.validatorRewarder);
        }
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
    receive() external payable onlyGateway {
        // The function body is empty since here we are implementing Diamond mechanism.
    }

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
