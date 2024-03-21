import {SubnetID} from "@ipc/src/structs/Subnet.sol";

import "../src/LinkedTokenDiamond.sol";
import {USDCTest} from "../src/USDCTest.sol";
import "../src/LinkedTokenFacet.sol";
import "../src/LinkedTokenControllerFacet.sol";
import "../src/LinkedTokenReplicaFacet.sol";
import "@ipc/src/diamond/DiamondCutFacet.sol";
import "@ipc/src/diamond/DiamondLoupeFacet.sol";
import "@ipc/src/OwnershipFacet.sol";

import "./../script/SelectorLibrary.sol";

contract LinkedTokenTestBase {

    LinkedTokenDiamond controller;
    LinkedTokenDiamond replica;

    function setUpLinkedTokenContracts (address controllerGateway,address replicaGateway, address controllerSubnetUSDC, SubnetID memory replicaSubnetName,  SubnetID memory controllerSubnet) internal  {

        //Controller 

        // Deploy controller facets
        DiamondCutFacet cutFacetC = new DiamondCutFacet();
        DiamondLoupeFacet loupeFacetC = new DiamondLoupeFacet();
        OwnershipFacet ownershipFacetC = new OwnershipFacet();
        LinkedTokenControllerFacet linkedTokenControllerFacetC = new LinkedTokenControllerFacet();

        // controller diamond constructor params
        LinkedTokenDiamond.ConstructorParams memory paramsController;
        paramsController.gateway=controllerGateway;
        paramsController.underlyingToken=controllerSubnetUSDC;
        paramsController.linkedSubnet = replicaSubnetName;

        // Prepare diamond cut with all facets
        IDiamond.FacetCut[] memory cuts = new IDiamond.FacetCut[](4);

        cuts[0] = createCut(address(cutFacetC), SelectorLibrary.resolveSelectors("DiamondCutFacet"));
        cuts[1] = createCut(address(loupeFacetC), SelectorLibrary.resolveSelectors("DiamondLoupeFacet"));
        cuts[2] = createCut(address(ownershipFacetC), SelectorLibrary.resolveSelectors("OwnershipFacet"));
        cuts[3] = createCut(address(linkedTokenControllerFacetC), SelectorLibrary.resolveSelectors("LinkedTokenControllerFacet"));
        //
        // Deploy the diamond with all facet cuts

        controller = new LinkedTokenDiamond(cuts, paramsController);

      

        //Replica
        
        //Deploy replica facets
        DiamondCutFacet cutFacetR = new DiamondCutFacet();
        DiamondLoupeFacet loupeFacetR = new DiamondLoupeFacet();
        OwnershipFacet ownershipFacetR = new OwnershipFacet();
        LinkedTokenReplicaFacet linkedTokenReplicaFacetR = new LinkedTokenReplicaFacet();

        // replica diamond constructor params
        LinkedTokenDiamond.ConstructorParams memory paramsReplica;
        paramsReplica.gateway=replicaGateway;
        paramsReplica.underlyingToken=controllerSubnetUSDC;
        paramsReplica.linkedSubnet = controllerSubnet;


        // Prepare diamond cut with all facets
        IDiamond.FacetCut[] memory cutsR = new IDiamond.FacetCut[](4);

        cutsR[0] = createCut(address(cutFacetR), SelectorLibrary.resolveSelectors("DiamondCutFacet"));
        cutsR[1] = createCut(address(loupeFacetR), SelectorLibrary.resolveSelectors("DiamondLoupeFacet"));
        cutsR[2] = createCut(address(ownershipFacetR), SelectorLibrary.resolveSelectors("OwnershipFacet"));
        cutsR[3] = createCut(address(linkedTokenReplicaFacetR), SelectorLibrary.resolveSelectors("LinkedTokenReplicaFacet"));
        //
        // Deploy the diamond with all facet cuts


        replica = new LinkedTokenDiamond(cutsR, paramsReplica);

        LinkedTokenReplicaFacet(address(replica)).initialize(address(controller));
        LinkedTokenControllerFacet(address(controller)).initialize(address(replica));
    }

    function createCut(address _facet, bytes4[] memory _selectors) internal pure returns (IDiamond.FacetCut memory cut) {
        return IDiamond.FacetCut({
            facetAddress: _facet,
            action: IDiamond.FacetCutAction.Add,
            functionSelectors: _selectors
        });
    }
}
