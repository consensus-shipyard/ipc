// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity 0.8.19;

import {IDiamondCut} from "../interfaces/IDiamondCut.sol";
import {IDiamond} from "../interfaces/IDiamond.sol";

library LibDiamond {
    bytes32 public constant DIAMOND_STORAGE_POSITION = keccak256("libdiamond.lib.diamond.storage");

    error NotOwner();
    error NoBytecodeAtAddress(address _contractAddress, string _message);
    error IncorrectFacetCutAction(IDiamondCut.FacetCutAction _action);
    error NoSelectorsProvidedForFacetForCut(address _facetAddress);
    error CannotAddFunctionToDiamondThatAlreadyExists(bytes4 _selector);
    error CannotAddSelectorsToZeroAddress(bytes4[] _selectors);
    error InitializationFunctionReverted(address _initializationContractAddress, bytes _calldata);

    event DiamondCut(IDiamondCut.FacetCut[] _diamondCut, address _init, bytes _calldata);

    struct FacetAddressAndSelectorPosition {
        address facetAddress;
        uint16 selectorPosition;
    }

    struct DiamondStorage {
        // function selector => facet address and selector position in selectors array
        mapping(bytes4 => FacetAddressAndSelectorPosition) facetAddressAndSelectorPosition;
        bytes4[] selectors;
        mapping(bytes4 => bool) supportedInterfaces;
        // owner of the contract
        address contractOwner;
    }

    function diamondStorage() internal pure returns (DiamondStorage storage ds) {
        bytes32 position = DIAMOND_STORAGE_POSITION;
        assembly {
            ds.slot := position
        }
    }

    function setContractOwner(address _newOwner) internal {
        DiamondStorage storage ds = diamondStorage();
        ds.contractOwner = _newOwner;
    }

    function contractOwner() internal view returns (address contractOwner_) {
        contractOwner_ = diamondStorage().contractOwner;
    }

    modifier onlyOwner() {
        if (msg.sender != diamondStorage().contractOwner) {
            revert NotOwner();
        }
        _;
    }

    function diamondCut(IDiamond.FacetCut[] memory _diamondCut, address _init, bytes memory _calldata) internal {
        uint256 length = _diamondCut.length;
        for (uint256 facetIndex = 0; facetIndex < length; ) {
            bytes4[] memory functionSelectors = _diamondCut[facetIndex].functionSelectors;
            address facetAddress = _diamondCut[facetIndex].facetAddress;
            if (functionSelectors.length == 0) {
                revert NoSelectorsProvidedForFacetForCut(facetAddress);
            }
            IDiamondCut.FacetCutAction action = _diamondCut[facetIndex].action;
            if (action == IDiamond.FacetCutAction.Add) {
                addFunctions(facetAddress, functionSelectors);
            } else {
                revert IncorrectFacetCutAction(action);
            }
            unchecked {
                ++facetIndex;
            }
        }
        emit DiamondCut({_diamondCut: _diamondCut, _init: _init, _calldata: _calldata});
        initializeDiamondCut(_init, _calldata);
    }

    function addFunctions(address _facetAddress, bytes4[] memory _functionSelectors) internal {
        if (_facetAddress == address(0)) {
            revert CannotAddSelectorsToZeroAddress(_functionSelectors);
        }
        DiamondStorage storage ds = diamondStorage();
        uint16 selectorCount = uint16(ds.selectors.length);
        enforceHasContractCode(_facetAddress, "diamondCut: Add facet has no code");
        uint256 length = _functionSelectors.length;
        for (uint256 selectorIndex = 0; selectorIndex < length; ) {
            bytes4 selector = _functionSelectors[selectorIndex];
            address oldFacetAddress = ds.facetAddressAndSelectorPosition[selector].facetAddress;
            if (oldFacetAddress != address(0)) {
                revert CannotAddFunctionToDiamondThatAlreadyExists(selector);
            }
            ds.facetAddressAndSelectorPosition[selector] = FacetAddressAndSelectorPosition(
                _facetAddress,
                selectorCount
            );
            ds.selectors.push(selector);
            ++selectorCount;
            unchecked {
                ++selectorIndex;
            }
        }
    }

    function initializeDiamondCut(address _init, bytes memory _calldata) internal {
        if (_init == address(0)) {
            return;
        }
        enforceHasContractCode(_init, "diamondCut: _init address has no code");
        // slither-disable-next-line low-level-calls
        (bool success, bytes memory error) = _init.delegatecall(_calldata); // solhint-disable-line avoid-low-level-calls
        if (!success) {
            if (error.length > 0) {
                // bubble up error
                /// @solidity memory-safe-assembly
                assembly {
                    let returndata_size := mload(error)
                    revert(add(32, error), returndata_size)
                }
            } else {
                revert InitializationFunctionReverted(_init, _calldata);
            }
        }
    }

    function enforceHasContractCode(address _contract, string memory _errorMessage) internal view {
        uint256 contractSize;
        assembly {
            contractSize := extcodesize(_contract)
        }
        if (contractSize == 0) {
            revert NoBytecodeAtAddress(_contract, _errorMessage);
        }
    }
}
