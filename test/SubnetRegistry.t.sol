// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity 0.8.19;

import {EMPTY_BYTES} from "../src/constants/Constants.sol";
import {ConsensusType} from "../src/enums/ConsensusType.sol";

import "forge-std/Test.sol";
import "forge-std/console.sol";

import {SubnetActorGetterFacet} from "../src/subnet/SubnetActorGetterFacet.sol";
import {SubnetActorManagerFacet} from "../src/subnet/SubnetActorManagerFacet.sol";
import {SubnetActorDiamond} from "../src/SubnetActorDiamond.sol";
import {SubnetID} from "../src/structs/Subnet.sol";
import {SubnetRegistry} from "../src/SubnetRegistry.sol";
import {SubnetIDHelper} from "../src/lib/SubnetIDHelper.sol";

contract SubnetRegistryTest is Test {
    using SubnetIDHelper for SubnetID;

    address private constant DEFAULT_IPC_GATEWAY_ADDR = address(1024);
    uint64 constant DEFAULT_CHECKPOINT_PERIOD = 10;
    bytes32 private constant DEFAULT_NETWORK_NAME = bytes32("test");
    uint256 private constant DEFAULT_MIN_VALIDATOR_STAKE = 1 ether;
    uint64 private constant DEFAULT_MIN_VALIDATORS = 1;
    bytes private constant GENESIS = EMPTY_BYTES;
    uint8 private constant DEFAULT_MAJORITY_PERCENTAGE = 70;
    uint64 private constant ROOTNET_CHAINID = 123;

    SubnetRegistry registry;
    bytes4[] empty;

    error FacetCannotBeZero();
    error WrongGateway();
    error CannotFindSubnet();
    error UnknownSubnet();
    error GatewayCannotBeZero();

    function setUp() public {
        bytes4[] memory mockedSelectors = new bytes4[](1);
        mockedSelectors[0] = 0x6cb2ecee;

        bytes4[] memory mockedSelectors2 = new bytes4[](1);
        mockedSelectors2[0] = 0x133f74ea;

        address getter = address(new SubnetActorGetterFacet());
        address manager = address(new SubnetActorManagerFacet());

        registry = new SubnetRegistry(DEFAULT_IPC_GATEWAY_ADDR, getter, manager, mockedSelectors, mockedSelectors2);
    }

    function test_Registry_Deployment_ZeroGetterFacet() public {
        vm.expectRevert(FacetCannotBeZero.selector);
        registry = new SubnetRegistry(DEFAULT_IPC_GATEWAY_ADDR, address(0), address(1), empty, empty);
    }

    function test_Registry_Deployment_ZeroManagerFacet() public {
        vm.expectRevert(FacetCannotBeZero.selector);
        registry = new SubnetRegistry(DEFAULT_IPC_GATEWAY_ADDR, address(1), address(0), empty, empty);
    }

    function test_Registry_Deployment_ZeroGateway() public {
        vm.expectRevert(GatewayCannotBeZero.selector);
        registry = new SubnetRegistry(address(0), address(1), address(1), empty, empty);
    }

    function test_Registry_Deployment_DifferentGateway() public {
        SubnetActorDiamond.ConstructorParams memory params = SubnetActorDiamond.ConstructorParams({
            parentId: SubnetID({root: ROOTNET_CHAINID, route: new address[](0)}),
            name: DEFAULT_NETWORK_NAME,
            ipcGatewayAddr: address(1),
            consensus: ConsensusType.Mir,
            minActivationCollateral: DEFAULT_MIN_VALIDATOR_STAKE,
            minValidators: DEFAULT_MIN_VALIDATORS,
            bottomUpCheckPeriod: DEFAULT_CHECKPOINT_PERIOD,
            topDownCheckPeriod: DEFAULT_CHECKPOINT_PERIOD,
            majorityPercentage: DEFAULT_MAJORITY_PERCENTAGE,
            genesis: GENESIS
        });
        vm.expectRevert(WrongGateway.selector);
        registry.newSubnetActor(params);
    }

    function test_Registry_LatestSubnetDeploy_Revert() public {
        vm.startPrank(DEFAULT_SENDER);
        SubnetActorDiamond.ConstructorParams memory params = SubnetActorDiamond.ConstructorParams({
            parentId: SubnetID({root: ROOTNET_CHAINID, route: new address[](0)}),
            name: DEFAULT_NETWORK_NAME,
            ipcGatewayAddr: DEFAULT_IPC_GATEWAY_ADDR,
            consensus: ConsensusType.Mir,
            minActivationCollateral: DEFAULT_MIN_VALIDATOR_STAKE,
            minValidators: DEFAULT_MIN_VALIDATORS,
            bottomUpCheckPeriod: DEFAULT_CHECKPOINT_PERIOD,
            topDownCheckPeriod: DEFAULT_CHECKPOINT_PERIOD,
            majorityPercentage: DEFAULT_MAJORITY_PERCENTAGE,
            genesis: GENESIS
        });
        registry.newSubnetActor(params);
        vm.expectRevert(CannotFindSubnet.selector);
        registry.latestSubnetDeployed(address(0));
    }

    function test_Registry_GetSubnetDeployedByNonce_Revert() public {
        vm.startPrank(DEFAULT_SENDER);
        SubnetActorDiamond.ConstructorParams memory params = SubnetActorDiamond.ConstructorParams({
            parentId: SubnetID({root: ROOTNET_CHAINID, route: new address[](0)}),
            name: DEFAULT_NETWORK_NAME,
            ipcGatewayAddr: DEFAULT_IPC_GATEWAY_ADDR,
            consensus: ConsensusType.Mir,
            minActivationCollateral: DEFAULT_MIN_VALIDATOR_STAKE,
            minValidators: DEFAULT_MIN_VALIDATORS,
            bottomUpCheckPeriod: DEFAULT_CHECKPOINT_PERIOD,
            topDownCheckPeriod: DEFAULT_CHECKPOINT_PERIOD,
            majorityPercentage: DEFAULT_MAJORITY_PERCENTAGE,
            genesis: GENESIS
        });
        registry.newSubnetActor(params);
        vm.expectRevert(CannotFindSubnet.selector);
        registry.getSubnetDeployedByNonce(address(0), 1);
    }

    function test_Registry_Deployment_Works() public {
        _assertDeploySubnetActor(
            DEFAULT_NETWORK_NAME,
            DEFAULT_IPC_GATEWAY_ADDR,
            ConsensusType.Mir,
            DEFAULT_MIN_VALIDATOR_STAKE,
            DEFAULT_MIN_VALIDATORS,
            DEFAULT_CHECKPOINT_PERIOD,
            GENESIS,
            DEFAULT_MAJORITY_PERCENTAGE
        );
    }

    function _assertDeploySubnetActor(
        bytes32 _name,
        address _ipcGatewayAddr,
        ConsensusType _consensus,
        uint256 _minActivationCollateral,
        uint64 _minValidators,
        uint64 _checkPeriod,
        bytes memory _genesis,
        uint8 _majorityPercentage
    ) public {
        vm.startPrank(DEFAULT_SENDER);
        SubnetActorDiamond.ConstructorParams memory params = SubnetActorDiamond.ConstructorParams({
            parentId: SubnetID({root: ROOTNET_CHAINID, route: new address[](0)}),
            name: _name,
            ipcGatewayAddr: _ipcGatewayAddr,
            consensus: _consensus,
            minActivationCollateral: _minActivationCollateral,
            minValidators: _minValidators,
            bottomUpCheckPeriod: _checkPeriod,
            topDownCheckPeriod: _checkPeriod,
            majorityPercentage: _majorityPercentage,
            genesis: _genesis
        });
        registry.newSubnetActor(params);
        require(registry.latestSubnetDeployed(DEFAULT_SENDER) != address(0));
        require(registry.subnets(DEFAULT_SENDER, 0) != address(0), "fails");
        require(registry.getSubnetDeployedByNonce(DEFAULT_SENDER, 0) == registry.latestSubnetDeployed(DEFAULT_SENDER));
    }
}
