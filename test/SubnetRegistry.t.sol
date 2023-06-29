// SPDX-License-Identifier: UNLICENSED
pragma solidity 0.8.19;

import {EMPTY_BYTES} from "../src/constants/Constants.sol";
import {ConsensusType} from "../src/enums/ConsensusType.sol";

import "forge-std/Test.sol";
import "forge-std/console.sol";

import "../src/SubnetRegistry.sol";

contract SubnetRegistryTest is Test {
    using SubnetIDHelper for SubnetID;

    address private constant DEFAULT_IPC_GATEWAY_ADDR = address(1024);
    uint64 constant DEFAULT_CHECKPOINT_PERIOD = 10;
    bytes32 private constant DEFAULT_NETWORK_NAME = bytes32("test");
    uint256 private constant DEFAULT_MIN_VALIDATOR_STAKE = 1 ether;
    uint64 private constant DEFAULT_MIN_VALIDATORS = 1;
    string private constant DEFAULT_NET_ADDR = "netAddr";
    bytes private constant GENESIS = EMPTY_BYTES;
    uint256 constant CROSS_MSG_FEE = 10 gwei;
    uint8 private constant DEFAULT_MAJORITY_PERCENTAGE = 70;
    uint64 private constant ROOTNET_CHAINID = 123;

    SubnetRegistry sr;

    function setUp() public {
        sr = new SubnetRegistry(DEFAULT_IPC_GATEWAY_ADDR);
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
        SubnetActor.ConstructParams memory params = SubnetActor.ConstructParams({
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
        sr.newSubnetActor(params);
        require(sr.latestSubnetDeployed(DEFAULT_SENDER) != address(0));
        require(sr.subnets(DEFAULT_SENDER, 0) != address(0), "fails");
        require(sr.getSubnetDeployedByNonce(DEFAULT_SENDER, 0) == sr.latestSubnetDeployed(DEFAULT_SENDER));
    }
}
