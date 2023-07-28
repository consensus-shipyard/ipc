// SPDX-License-Identifier: UNLICENSED
pragma solidity 0.8.19;

interface IDiamond {
    enum FacetCutAction {
        Add,
        Replace,
        Remove
    }
    // Add=0, Replace=1, Remove=2

    struct FacetCut {
        address facetAddress;
        FacetCutAction action;
        bytes4[] functionSelectors;
    }
    // The DiamondCut event records all function changes to a diamond.
    event DiamondCut(FacetCut[] _diamondCut, address _init, bytes _calldata);
}
