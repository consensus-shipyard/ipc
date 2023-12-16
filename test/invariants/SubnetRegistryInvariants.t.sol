// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity 0.8.19;

import {StdInvariant, Test} from "forge-std/Test.sol";
import {TestUtils} from "../helpers/TestUtils.sol";
import {IDiamond} from "../../src/interfaces/IDiamond.sol";
import {SubnetRegistryHandler} from "./handlers/SubnetRegistryHandler.sol";
import {SubnetRegistryDiamond} from "../../src/SubnetRegistryDiamond.sol";
import {SubnetIDHelper} from "../../src/lib/SubnetIDHelper.sol";
import {SubnetActorGetterFacet} from "../../src/subnet/SubnetActorGetterFacet.sol";
import {SubnetActorManagerFacet} from "../../src/subnet/SubnetActorManagerFacet.sol";
import {SubnetID} from "../../src/structs/Subnet.sol";
import {RegisterSubnetFacet} from "../../src/subnetregistry/RegisterSubnetFacet.sol";
import {SubnetGetterFacet} from "../../src/subnetregistry/SubnetGetterFacet.sol";
import {DiamondLoupeFacet} from "../../src/diamond/DiamondLoupeFacet.sol";
import {DiamondCutFacet} from "../../src/diamond/DiamondCutFacet.sol";

contract SubnetRegistryInvariants is StdInvariant, Test {
    address private constant DEFAULT_IPC_GATEWAY_ADDR = address(1024);

    SubnetRegistryHandler private registryHandler;

    SubnetRegistryDiamond private registry;
    bytes4[] private empty;

    address private louperFacetAddr;
    address private cutFacetAddr;
    address private registerSubnetFacetAddr;
    address private subnetGetterFacetAddr;

    DiamondLoupeFacet private louperFacet;
    DiamondCutFacet private cutFacet;
    RegisterSubnetFacet private registerSubnetFacet;
    SubnetGetterFacet private subnetGetterFacet;

    bytes4[] private cutFacetSelectors;
    bytes4[] private louperSelectors;

    bytes4[] private registerSubnetFacetSelectors;
    bytes4[] private subnetGetterFacetSelectors;

    error FacetCannotBeZero();
    error WrongGateway();
    error CannotFindSubnet();
    error UnknownSubnet();
    error GatewayCannotBeZero();

    constructor() {
        louperSelectors = TestUtils.generateSelectors(vm, "DiamondLoupeFacet");
        cutFacetSelectors = TestUtils.generateSelectors(vm, "DiamondCutFacet");
        registerSubnetFacetSelectors = TestUtils.generateSelectors(vm, "RegisterSubnetFacet");
        subnetGetterFacetSelectors = TestUtils.generateSelectors(vm, "SubnetGetterFacet");
    }

    // Event emitted when a new SubnetRegistry is created
    event SubnetRegistryCreated(address indexed subnetRegistryAddress);

    function setUp() public {
        bytes4[] memory mockedSelectors = new bytes4[](1);
        mockedSelectors[0] = 0x6cb2ecee;

        bytes4[] memory mockedSelectors2 = new bytes4[](1);
        mockedSelectors2[0] = 0x133f74ea;

        SubnetRegistryDiamond.ConstructorParams memory params;
        params.gateway = DEFAULT_IPC_GATEWAY_ADDR;
        params.getterFacet = address(new SubnetActorGetterFacet());
        params.managerFacet = address(new SubnetActorManagerFacet());
        params.subnetGetterSelectors = mockedSelectors;
        params.subnetManagerSelectors = mockedSelectors2;

        louperFacet = new DiamondLoupeFacet();
        louperFacetAddr = address(louperFacet);

        cutFacet = new DiamondCutFacet();
        cutFacetAddr = address(cutFacet);

        registerSubnetFacet = new RegisterSubnetFacet();
        registerSubnetFacetAddr = address(registerSubnetFacet);

        subnetGetterFacet = new SubnetGetterFacet();
        subnetGetterFacetAddr = address(subnetGetterFacet);

        IDiamond.FacetCut[] memory gwDiamondCut = new IDiamond.FacetCut[](4);

        gwDiamondCut[0] = (
            IDiamond.FacetCut({
                facetAddress: louperFacetAddr,
                action: IDiamond.FacetCutAction.Add,
                functionSelectors: louperSelectors
            })
        );
        gwDiamondCut[1] = (
            IDiamond.FacetCut({
                facetAddress: cutFacetAddr,
                action: IDiamond.FacetCutAction.Add,
                functionSelectors: cutFacetSelectors
            })
        );
        gwDiamondCut[2] = (
            IDiamond.FacetCut({
                facetAddress: registerSubnetFacetAddr,
                action: IDiamond.FacetCutAction.Add,
                functionSelectors: registerSubnetFacetSelectors
            })
        );
        gwDiamondCut[3] = (
            IDiamond.FacetCut({
                facetAddress: subnetGetterFacetAddr,
                action: IDiamond.FacetCutAction.Add,
                functionSelectors: subnetGetterFacetSelectors
            })
        );

        registry = new SubnetRegistryDiamond(gwDiamondCut, params);
        registryHandler = new SubnetRegistryHandler(registry);

        bytes4[] memory fuzzSelectors = new bytes4[](1);
        fuzzSelectors[0] = SubnetRegistryHandler.deploySubnetActorFromRegistry.selector;

        targetSelector(FuzzSelector({addr: address(registryHandler), selectors: fuzzSelectors}));
        targetContract(address(registryHandler));
    }

    /// @notice The Gateway address is not changed.
    /// forge-config: default.invariant.runs = 5
    /// forge-config: default.invariant.depth = 10
    /// forge-config: default.invariant.fail-on-revert = false
    function invariant_SR_01_gateway_address_is_persistent() public {
        assertEq(registryHandler.getGateway(), DEFAULT_IPC_GATEWAY_ADDR);
    }

    /// @notice If a subnet was created then it's address can be retrieved.
    /// TODO: this test has the same issue as https://github.com/foundry-rs/foundry/issues/6074
    /// We may need to update the test setup when the issue is fixed.
    ///
    /// forge-config: default.invariant.runs = 50
    /// forge-config: default.invariant.depth = 10
    /// forge-config: default.invariant.fail-on-revert = false
    function invariant_SR_02_subnet_address_can_be_retrieved() public {
        address[] memory owners = registryHandler.getOwners();
        uint256 length = owners.length;
        if (length == 0) {
            return;
        }
        for (uint256 i; i < length; ++i) {
            address owner = owners[i];
            uint64 nonce = registryHandler.getUserLastNonce(owner);

            assertNotEq(nonce, 0);
            assertEq(
                registryHandler.getSubnetDeployedBy(owner),
                registryHandler.getSubnetDeployedWithNonce(owner, nonce - 1)
            );
        }
    }
}
