// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity 0.8.19;

import {SubnetActorStorage} from "./lib/LibSubnetActorStorage.sol";
import {ConsensusType} from "./enums/ConsensusType.sol";
import {IDiamond} from "./interfaces/IDiamond.sol";
import {GatewayCannotBeZero, NotGateway, InvalidSubmissionPeriod, InvalidCollateral} from "./errors/IPCErrors.sol";
import {LibDiamond} from "./lib/LibDiamond.sol";
import {LibVoting} from "./lib/LibVoting.sol";
import {SubnetID} from "./structs/Subnet.sol";
import {SubnetIDHelper} from "./lib/SubnetIDHelper.sol";
import {Status} from "./enums/Status.sol";

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
        uint64 topDownCheckPeriod;
        uint8 majorityPercentage;
        bytes genesis;
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

        LibDiamond.setContractOwner(msg.sender);
        LibDiamond.diamondCut({_diamondCut: _diamondCut, _init: address(0), _calldata: new bytes(0)});

        s.parentId = params.parentId;
        s.name = params.name;
        s.ipcGatewayAddr = params.ipcGatewayAddr;
        s.consensus = params.consensus;
        s.minActivationCollateral = params.minActivationCollateral;
        s.minValidators = params.minValidators;
        s.topDownCheckPeriod = params.topDownCheckPeriod;
        s.bottomUpCheckPeriod = params.bottomUpCheckPeriod;
        s.status = Status.Instantiated;
        s.genesis = params.genesis;
        s.currentSubnetHash = s.parentId.createSubnetId(address(this)).toHash();
        // NOTE: we currently use 0 as the genesisEpoch for subnets so checkpoints
        // are submitted directly from epoch 0.
        // In the future we can use the current epoch. This will be really
        // useful once we support the docking of subnets to new parents, etc.
        LibVoting.initGenesisEpoch(0);

        // init Voting params.
        LibVoting.initVoting(params.majorityPercentage, params.topDownCheckPeriod);
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
