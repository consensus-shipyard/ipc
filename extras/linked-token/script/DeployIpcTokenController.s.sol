// SPDX-License-Identifier: MIT
pragma solidity ^0.8.23;

import "./ConfigManager.sol";
import "@ipc/src/structs/Subnet.sol";

import "../src/LinkedTokenDiamond.sol";

import "../src/LinkedTokenControllerFacet.sol";
import "@ipc/src/diamond/DiamondCutFacet.sol";
import "@ipc/src/diamond/DiamondLoupeFacet.sol";
import "@ipc/src/OwnershipFacet.sol";

import {IDiamond} from "@ipc/src/interfaces/IDiamond.sol";

import "./SelectorLibrary.sol";

contract DeployIpcTokenController is ConfigManager {
    function deployFacets() external {
        vm.startBroadcast();

        // Deploy facets
        DiamondCutFacet cutFacet = new DiamondCutFacet();
        DiamondLoupeFacet loupeFacet = new DiamondLoupeFacet();
        OwnershipFacet ownershipFacet = new OwnershipFacet();
        LinkedTokenControllerFacet linkedTokenControllerFacet = new LinkedTokenControllerFacet();

        writeConfig("LinkedTokenController.DiamondCutFacet", vm.toString(address(cutFacet)));
        writeConfig("LinkedTokenController.DiamondLoupeFacet", vm.toString(address(loupeFacet)));
        writeConfig("LinkedTokenController.OwnershipFacet", vm.toString(address(ownershipFacet)));
        writeConfig("LinkedTokenController.LinkedTokenControllerFacet", vm.toString(address(linkedTokenControllerFacet)));

    }
    function deployDiamond(address gateway, address tokenContractAddress, uint64 _rootNetChainId, address[] memory _route, address cutFacet, address loupeFacet, address ownershipFacet, address linkedTokenControllerFacet ) external {

        vm.startBroadcast();

        SubnetID memory destinationSubnet = SubnetID({root: _rootNetChainId, route: _route});
        LinkedTokenDiamond.ConstructorParams memory params;
        params.gateway=gateway;
        params.underlyingToken=tokenContractAddress;
        params.linkedSubnet = destinationSubnet;

        // Prepare diamond cut with all facets
        IDiamond.FacetCut[] memory cuts = new IDiamond.FacetCut[](4);

        cuts[0] = createCut(cutFacet, SelectorLibrary.resolveSelectors("DiamondCutFacet"));
        cuts[1] = createCut(loupeFacet, SelectorLibrary.resolveSelectors("DiamondLoupeFacet"));
        cuts[2] = createCut(ownershipFacet, SelectorLibrary.resolveSelectors("OwnershipFacet"));
        cuts[3] = createCut(linkedTokenControllerFacet, SelectorLibrary.resolveSelectors("LinkedTokenControllerFacet"));
        //
        // Deploy the diamond with all facet cuts

        LinkedTokenDiamond diamond = new LinkedTokenDiamond(cuts, params);

        writeConfig("LinkedTokenController.LinkedTokenController", vm.toString(address(diamond)));

        vm.stopBroadcast();
    }

    function createCut(address _facet, bytes4[] memory _selectors) internal pure returns (IDiamond.FacetCut memory cut) {
        return IDiamond.FacetCut({
            facetAddress: _facet,
            action: IDiamond.FacetCutAction.Add,
            functionSelectors: _selectors
        });
    }
}

