// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity 0.8.19;

import "forge-std/Test.sol";
import "forge-std/console.sol";
import "../src/errors/IPCErrors.sol";
import {TestUtils} from "./TestUtils.sol";
import {EMPTY_BYTES, METHOD_SEND, EMPTY_HASH} from "../src/constants/Constants.sol";
import {ConsensusType} from "../src/enums/ConsensusType.sol";
import {Status} from "../src/enums/Status.sol";
import {CrossMsg, BottomUpCheckpoint, TopDownCheckpoint, StorableMsg, ChildCheck} from "../src/structs/Checkpoint.sol";
import {FvmAddress} from "../src/structs/FvmAddress.sol";
import {SubnetID, IPCAddress, Subnet} from "../src/structs/Subnet.sol";
import {StorableMsg} from "../src/structs/Checkpoint.sol";
import {IGateway} from "../src/interfaces/IGateway.sol";
import {IDiamond} from "../src/interfaces/IDiamond.sol";
import {IDiamondCut} from "../src/interfaces/IDiamondCut.sol";
import {FvmAddressHelper} from "../src/lib/FvmAddressHelper.sol";
import {CheckpointHelper} from "../src/lib/CheckpointHelper.sol";
import {StorableMsgHelper} from "../src/lib/StorableMsgHelper.sol";
import {SubnetIDHelper} from "../src/lib/SubnetIDHelper.sol";
import {GatewayDiamond} from "../src/GatewayDiamond.sol";
import {SubnetActorDiamond} from "../src/SubnetActorDiamond.sol";
import {GatewayGetterFacet} from "../src/gateway/GatewayGetterFacet.sol";
import {GatewayMessengerFacet} from "../src/gateway/GatewayMessengerFacet.sol";
import {GatewayManagerFacet} from "../src/gateway/GatewayManagerFacet.sol";
import {GatewayRouterFacet} from "../src/gateway/GatewayRouterFacet.sol";
import {SubnetActorManagerFacet} from "../src/subnet/SubnetActorManagerFacet.sol";
import {SubnetActorGetterFacet} from "../src/subnet/SubnetActorGetterFacet.sol";
import {FilAddress} from "fevmate/utils/FilAddress.sol";

contract SubnetActorDiamondTest is Test {
    using SubnetIDHelper for SubnetID;
    using CheckpointHelper for BottomUpCheckpoint;
    using FilAddress for address;
    using FvmAddressHelper for FvmAddress;

    address private constant DEFAULT_IPC_GATEWAY_ADDR = address(1024);
    uint64 constant DEFAULT_CHECKPOINT_PERIOD = 10;
    bytes32 private constant DEFAULT_NETWORK_NAME = bytes32("test");
    uint256 private constant DEFAULT_MIN_VALIDATOR_STAKE = 1 ether;
    uint64 private constant DEFAULT_MIN_VALIDATORS = 1;
    string private constant DEFAULT_NET_ADDR = "netAddr";
    uint256 constant CROSS_MSG_FEE = 10 gwei;
    uint8 private constant DEFAULT_MAJORITY_PERCENTAGE = 70;
    uint64 private constant ROOTNET_CHAINID = 123;

    address GATEWAY_ADDRESS;

    bytes4[] saGetterSelectors;
    bytes4[] saManagerSelectors;

    bytes4[] gwRouterSelectors;
    bytes4[] gwManagerSelectors;
    bytes4[] gwGetterSelectors;
    bytes4[] gwMessengerSelectors;

    SubnetActorDiamond saDiamond;
    SubnetActorManagerFacet saManager;
    SubnetActorGetterFacet saGetter;

    GatewayDiamond gatewayDiamond;
    GatewayManagerFacet gwManager;
    GatewayGetterFacet gwGetter;
    GatewayRouterFacet gwRouter;
    GatewayMessengerFacet gwMessenger;

    constructor() {
        saGetterSelectors = TestUtils.generateSelectors(vm, "SubnetActorGetterFacet");
        saManagerSelectors = TestUtils.generateSelectors(vm, "SubnetActorManagerFacet");

        gwRouterSelectors = TestUtils.generateSelectors(vm, "GatewayRouterFacet");
        gwGetterSelectors = TestUtils.generateSelectors(vm, "GatewayGetterFacet");
        gwManagerSelectors = TestUtils.generateSelectors(vm, "GatewayManagerFacet");
        gwMessengerSelectors = TestUtils.generateSelectors(vm, "GatewayMessengerFacet");
    }

    function createGatewayDiamond(GatewayDiamond.ConstructorParams memory params) public returns (GatewayDiamond) {
        gwRouter = new GatewayRouterFacet();
        gwManager = new GatewayManagerFacet();
        gwGetter = new GatewayGetterFacet();
        gwMessenger = new GatewayMessengerFacet();

        IDiamond.FacetCut[] memory diamondCut = new IDiamond.FacetCut[](4);

        diamondCut[0] = (
            IDiamond.FacetCut({
                facetAddress: address(gwRouter),
                action: IDiamond.FacetCutAction.Add,
                functionSelectors: gwRouterSelectors
            })
        );

        diamondCut[1] = (
            IDiamond.FacetCut({
                facetAddress: address(gwManager),
                action: IDiamond.FacetCutAction.Add,
                functionSelectors: gwManagerSelectors
            })
        );

        diamondCut[2] = (
            IDiamond.FacetCut({
                facetAddress: address(gwGetter),
                action: IDiamond.FacetCutAction.Add,
                functionSelectors: gwGetterSelectors
            })
        );

        diamondCut[3] = (
            IDiamond.FacetCut({
                facetAddress: address(gwMessenger),
                action: IDiamond.FacetCutAction.Add,
                functionSelectors: gwMessengerSelectors
            })
        );

        gatewayDiamond = new GatewayDiamond(diamondCut, params);

        return gatewayDiamond;
    }

    function createSubnetActorDiamondWithFaucets(
        SubnetActorDiamond.ConstructorParams memory params,
        address getterFaucet,
        address managerFaucet
    ) public returns (SubnetActorDiamond) {
        IDiamond.FacetCut[] memory diamondCut = new IDiamond.FacetCut[](2);

        diamondCut[0] = (
            IDiamond.FacetCut({
                facetAddress: getterFaucet,
                action: IDiamond.FacetCutAction.Add,
                functionSelectors: saGetterSelectors
            })
        );

        diamondCut[1] = (
            IDiamond.FacetCut({
                facetAddress: managerFaucet,
                action: IDiamond.FacetCutAction.Add,
                functionSelectors: saManagerSelectors
            })
        );

        saDiamond = new SubnetActorDiamond(diamondCut, params);
        return saDiamond;
    }

    function setUp() public {
        GatewayDiamond.ConstructorParams memory gwConstructorParams = GatewayDiamond.ConstructorParams({
            networkName: SubnetID({root: ROOTNET_CHAINID, route: new address[](0)}),
            bottomUpCheckPeriod: DEFAULT_CHECKPOINT_PERIOD,
            topDownCheckPeriod: DEFAULT_CHECKPOINT_PERIOD,
            msgFee: CROSS_MSG_FEE,
            minCollateral: 1,
            majorityPercentage: DEFAULT_MAJORITY_PERCENTAGE
        });

        gatewayDiamond = createGatewayDiamond(gwConstructorParams);

        gwGetter = GatewayGetterFacet(address(gatewayDiamond));
        gwManager = GatewayManagerFacet(address(gatewayDiamond));
        gwRouter = GatewayRouterFacet(address(gatewayDiamond));
        gwMessenger = GatewayMessengerFacet(address(gatewayDiamond));

        GATEWAY_ADDRESS = address(gatewayDiamond);

        _assertDeploySubnetActor(
            DEFAULT_NETWORK_NAME,
            GATEWAY_ADDRESS,
            ConsensusType.Mir,
            DEFAULT_MIN_VALIDATOR_STAKE,
            DEFAULT_MIN_VALIDATORS,
            DEFAULT_CHECKPOINT_PERIOD,
            DEFAULT_MAJORITY_PERCENTAGE
        );
    }

    function testSubnetActorDiamond_Deployment_Works(
        bytes32 _networkName,
        address _ipcGatewayAddr,
        uint256 _minActivationCollateral,
        uint64 _minValidators,
        uint64 _checkPeriod,
        uint8 _majorityPercentage
    ) public {
        vm.assume(_minActivationCollateral > DEFAULT_MIN_VALIDATOR_STAKE);
        vm.assume(_checkPeriod > DEFAULT_CHECKPOINT_PERIOD);
        vm.assume(_majorityPercentage <= 100);
        vm.assume(_ipcGatewayAddr != address(0));

        _assertDeploySubnetActor(
            _networkName,
            _ipcGatewayAddr,
            ConsensusType.Mir,
            _minActivationCollateral,
            _minValidators,
            _checkPeriod,
            _majorityPercentage
        );

        SubnetID memory parent = saGetter.getParent();
        require(parent.isRoot(), "parent.isRoot()");

        require(saGetter.bottomUpCheckPeriod() == _checkPeriod, "bottomUpCheckPeriod");

        require(saGetter.getValidators().length == 0, "empty validators");

        require(saGetter.getValidatorSet().validators.length == 0, "empty validator set");
    }

    function testSubnetActorDiamond_Deployments_Fail_GatewayCannotBeZero() public {
        SubnetActorManagerFacet saDupMangerFaucet = new SubnetActorManagerFacet();
        SubnetActorGetterFacet saDupGetterFaucet = new SubnetActorGetterFacet();

        vm.expectRevert(GatewayCannotBeZero.selector);
        createSubnetActorDiamondWithFaucets(
            SubnetActorDiamond.ConstructorParams({
                parentId: SubnetID(ROOTNET_CHAINID, new address[](0)),
                name: DEFAULT_NETWORK_NAME,
                ipcGatewayAddr: address(0),
                consensus: ConsensusType.Mir,
                minActivationCollateral: DEFAULT_MIN_VALIDATOR_STAKE,
                minValidators: DEFAULT_MIN_VALIDATORS,
                bottomUpCheckPeriod: DEFAULT_CHECKPOINT_PERIOD,
                topDownCheckPeriod: DEFAULT_CHECKPOINT_PERIOD,
                majorityPercentage: DEFAULT_MAJORITY_PERCENTAGE
            }),
            address(saDupGetterFaucet),
            address(saDupMangerFaucet)
        );
    }

    function testSubnetActorDiamond_Receive_Fail_NotGateway() public {
        vm.expectRevert(NotGateway.selector);
        (bool success, ) = payable(address(saManager)).call{value: 1}("");
        require(success);
    }

    function testSubnetActorDiamond_Receive_Works() public {
        vm.prank(GATEWAY_ADDRESS);
        vm.deal(GATEWAY_ADDRESS, 1);
        (bool success, ) = payable(address(saManager)).call{value: 1}("");
        require(success);
    }

    function testSubnetActorDiamond_Join_Fail_NoMinColalteral() public {
        address validator = vm.addr(100);

        vm.deal(validator, 1 gwei);
        vm.prank(validator);
        vm.expectRevert(CollateralIsZero.selector);

        saManager.join(DEFAULT_NET_ADDR, FvmAddress({addrType: 1, payload: new bytes(20)}));
    }

    function testSubnetActorDiamond_Join_Fail_AlreadyKilled() public {
        address validator = vm.addr(1235);

        _assertJoin(validator, DEFAULT_MIN_VALIDATOR_STAKE);
        _assertLeave(validator, DEFAULT_MIN_VALIDATOR_STAKE);
        _assertKill(validator);

        vm.expectRevert(SubnetAlreadyKilled.selector);
        vm.prank(validator);
        vm.deal(validator, DEFAULT_MIN_VALIDATOR_STAKE + 1);

        saManager.join{value: DEFAULT_MIN_VALIDATOR_STAKE}(
            DEFAULT_NET_ADDR,
            FvmAddress({addrType: 1, payload: new bytes(20)})
        );
    }

    function testSubnetActorDiamond_Join_Works_CallAddStake() public {
        address validator = vm.addr(1235);

        _assertJoin(validator, DEFAULT_MIN_VALIDATOR_STAKE);

        vm.expectCall(
            GATEWAY_ADDRESS,
            DEFAULT_MIN_VALIDATOR_STAKE,
            abi.encodeWithSelector(gwManager.addStake.selector),
            1
        );

        _assertJoin(validator, DEFAULT_MIN_VALIDATOR_STAKE);

        require(saGetter.validatorCount() == 1);
        require(saGetter.getValidators().length == 1);
        require(saGetter.getValidatorSet().validators.length == 1);
        require(saGetter.getValidatorSet().configurationNumber == 0);
        require(saGetter.validatorAt(0) == validator);
    }

    function testSubnetActorDiamond_MultipleJoins_Works_GetValidators() public {
        address validator1 = vm.addr(1231);
        address validator2 = vm.addr(1232);
        address validator3 = vm.addr(1233);
        address validator4 = vm.addr(1234);
        address validator5 = vm.addr(1235);
        address validator6 = vm.addr(1236);
        address validator7 = vm.addr(1237);

        _assertJoin(validator1, DEFAULT_MIN_VALIDATOR_STAKE);
        _assertJoin(validator2, DEFAULT_MIN_VALIDATOR_STAKE);
        _assertJoin(validator3, DEFAULT_MIN_VALIDATOR_STAKE);
        _assertJoin(validator4, DEFAULT_MIN_VALIDATOR_STAKE);
        _assertJoin(validator5, DEFAULT_MIN_VALIDATOR_STAKE);
        _assertJoin(validator6, DEFAULT_MIN_VALIDATOR_STAKE);
        _assertJoin(validator7, DEFAULT_MIN_VALIDATOR_STAKE);

        require(saGetter.validatorCount() == 7);
        require(saGetter.getValidators().length == 7);
        require(saGetter.getValidatorSet().validators.length == 7);

        address[] memory result;
        uint256 offset;

        (result, offset) = saGetter.getRangeOfValidators(0, 2);
        require(result.length == 2);
        require(offset == 2);

        (result, offset) = saGetter.getRangeOfValidators(0, 0);
        require(result.length == 0);
        require(offset == 0);

        (result, offset) = saGetter.getRangeOfValidators(10, 0);
        require(result.length == 0);
        require(offset == 0);

        (result, offset) = saGetter.getRangeOfValidators(2, 4);
        require(result.length == 4);
        require(offset == 6);

        (result, offset) = saGetter.getRangeOfValidators(2, 0);
        require(result.length == 0);
        require(offset == 0);

        (result, offset) = saGetter.getRangeOfValidators(6, 10);
        require(result.length == 1);
        require(offset == 7);

        (result, offset) = saGetter.getRangeOfValidators(10, 10);
        require(result.length == 0);
        require(offset == 0);
    }

    function testSubnetActorDiamond_MultipleJoins_Fuzz_GetValidators(uint256 offset, uint256 limit, uint256 n) public {
        offset = bound(offset, 0, 10);
        limit = bound(limit, 0, 10);
        n = bound(n, 0, 10);

        console.log("fuzz data:");
        console.log(offset);
        console.log(limit);
        console.log(n);

        for (uint256 i = 0; i < n; i++) {
            address validator = vm.addr(i + 1000);
            _assertJoin(validator, DEFAULT_MIN_VALIDATOR_STAKE);
        }

        require(saGetter.validatorCount() == n);
        require(saGetter.getValidators().length == n);
        require(saGetter.getValidatorSet().validators.length == n);

        address[] memory result;
        uint256 newOffset;

        (result, newOffset) = saGetter.getRangeOfValidators(offset, limit);
        if (limit == 0 || n <= offset) {
            require(result.length == 0, "result.length == 0");
        } else {
            if (limit > n - offset) {
                limit = n - offset;
            }
            require(result.length == limit, "result.length == limit");
        }
    }

    function testSubnetActorDiamond_Join_Works_CallRegister() public {
        address validator = vm.addr(1235);

        vm.expectCall(
            GATEWAY_ADDRESS,
            DEFAULT_MIN_VALIDATOR_STAKE,
            abi.encodeWithSelector(gwManager.register.selector),
            1
        );

        _assertJoin(validator, DEFAULT_MIN_VALIDATOR_STAKE);
    }

    function testSubnetActorDiamond_Join_Works_LessThanMinStake() public {
        address validator = vm.addr(1235);
        uint256 amount = DEFAULT_MIN_VALIDATOR_STAKE / 2;
        vm.deal(validator, amount + 1);
        vm.prank(validator);
        vm.expectCall(GATEWAY_ADDRESS, amount, abi.encodeWithSelector(gwManager.register.selector), 0);
        vm.expectCall(GATEWAY_ADDRESS, amount, abi.encodeWithSelector(gwManager.addStake.selector), 0);
        saManager.join{value: amount}(DEFAULT_NET_ADDR, FvmAddress({addrType: 1, payload: new bytes(20)}));

        require(saGetter.validatorCount() == 0);
        require(gwGetter.listSubnets().length == 0);
    }

    function testSubnetActorDiamond_Join_Works_MultipleNewValidators() public {
        _assertJoin(vm.addr(1234), DEFAULT_MIN_VALIDATOR_STAKE);
        _assertJoin(vm.addr(1235), DEFAULT_MIN_VALIDATOR_STAKE);

        require(saGetter.validatorCount() == 2);
        require(gwGetter.listSubnets().length == 1);
    }

    function testSubnetActorDiamond_Join_Works_OneValidatorWithMinimumStake() public {
        require(gwGetter.listSubnets().length == 0, "listSubnets correct");
        require(saGetter.validatorCount() == 0, "validatorCount correct");

        address validator = vm.addr(1234);

        vm.startPrank(validator);
        vm.deal(validator, DEFAULT_MIN_VALIDATOR_STAKE);

        require(validator.balance == DEFAULT_MIN_VALIDATOR_STAKE, "balance() == DEFAULT_MIN_VALIDATOR_STAKE");
        require(saGetter.stake(validator) == 0, "stake(validator) == 0");
        require(saGetter.totalStake() == 0, "totalStake() == 0");

        saManager.join{value: DEFAULT_MIN_VALIDATOR_STAKE}(
            DEFAULT_NET_ADDR,
            FvmAddress({addrType: 1, payload: new bytes(20)})
        );

        require(saGetter.stake(validator) == DEFAULT_MIN_VALIDATOR_STAKE);
        require(saGetter.totalStake() == DEFAULT_MIN_VALIDATOR_STAKE);
        require(validator.balance == 0);

        vm.stopPrank();

        require(saGetter.validatorCount() == 1, "validatorCount() correct");
        require(gwGetter.listSubnets().length == 1, "listSubnets() correct");
    }

    function testSubnetActorDiamond_Join_Works_NoNewValidator_CollateralNotEnough() public {
        address validator = vm.addr(1235);

        _assertJoin(validator, DEFAULT_MIN_VALIDATOR_STAKE - 1);

        require(saGetter.validatorCount() == 0);
        require(saGetter.status() == Status.Instantiated);
    }

    function testSubnetActorDiamond_Join_Works_ReactivateSubnet() public {
        _assertJoin(vm.addr(1234), DEFAULT_MIN_VALIDATOR_STAKE);
        _assertLeave(vm.addr(1234), DEFAULT_MIN_VALIDATOR_STAKE);

        require(saGetter.totalStake() == 0);
        require(saGetter.validatorCount() == 0);
        require(saGetter.status() == Status.Inactive);

        _assertJoin(vm.addr(1235), DEFAULT_MIN_VALIDATOR_STAKE);

        require(saGetter.validatorCount() == 1);
        require(saGetter.status() == Status.Active);
    }

    function testSubnetActorDiamond_Leave_Works_NoValidatorsLeft() public payable {
        address validator = address(1235);
        uint256 amount = DEFAULT_MIN_VALIDATOR_STAKE;

        _assertJoin(validator, amount);

        _assertLeave(validator, amount);

        require(saGetter.totalStake() == 0);
        require(saGetter.validatorCount() == 0);
        require(saGetter.status() == Status.Inactive);
    }

    function testSubnetActorDiamond_Leave_Works_StillActive() public payable {
        address validator1 = address(1234);
        address validator2 = address(1235);
        uint256 amount = DEFAULT_MIN_VALIDATOR_STAKE;

        _assertJoin(validator1, amount);
        _assertJoin(validator2, amount);

        _assertLeave(validator1, amount);

        require(saGetter.totalStake() == amount);
        require(saGetter.validatorCount() == 1);
        require(saGetter.status() == Status.Active);
    }

    function testSubnetActorDiamond_Leave_Fail_AlreadyKilled() public payable {
        address validator = address(1235);
        uint256 amount = DEFAULT_MIN_VALIDATOR_STAKE;

        _assertJoin(validator, amount);

        _assertLeave(validator, amount);
        _assertKill(validator);

        vm.prank(validator);
        vm.deal(validator, amount);
        vm.expectRevert(SubnetAlreadyKilled.selector);

        saManager.leave();
    }

    function testSubnetActorDiamond_Leave_Fail_NoStake() public payable {
        address caller = address(1235);

        vm.prank(caller);
        vm.deal(caller, 1 ether);

        vm.expectRevert(NotValidator.selector);

        saManager.leave();
    }

    function testSubnetActorDiamond_Kill_Works() public payable {
        address validator = address(1235);

        _assertJoin(validator, DEFAULT_MIN_VALIDATOR_STAKE);
        _assertLeave(validator, DEFAULT_MIN_VALIDATOR_STAKE);

        _assertKill(validator);

        require(GATEWAY_ADDRESS.balance == 0);
        require(gwGetter.totalSubnets() == 0);
    }

    function testSubnetActorDiamond_Kill_Fails_NotAllValidatorsLeft() public payable {
        address validator1 = address(1235);
        address validator2 = address(1236);

        _assertJoin(validator1, DEFAULT_MIN_VALIDATOR_STAKE);
        _assertJoin(validator2, DEFAULT_MIN_VALIDATOR_STAKE);

        _assertLeave(validator1, DEFAULT_MIN_VALIDATOR_STAKE);

        vm.prank(validator1);
        vm.expectRevert(NotAllValidatorsHaveLeft.selector);
        saManager.kill();
    }

    function testSubnetActorDiamond_Kill_Fails_AlreadyTerminating() public {
        address validator = vm.addr(1235);

        _assertJoin(validator, DEFAULT_MIN_VALIDATOR_STAKE);
        _assertLeave(validator, DEFAULT_MIN_VALIDATOR_STAKE);

        _assertKill(validator);

        vm.prank(validator);
        vm.expectRevert(SubnetAlreadyKilled.selector);

        saManager.kill();
    }

    function testSubnetActorDiamond_SetValidatorNetAddr_Works() public payable {
        string memory newNetAddr = "1.2.3.4";

        address validator = address(1235);

        _assertJoin(validator, DEFAULT_MIN_VALIDATOR_STAKE);

        string memory result = saGetter.validatorNetAddr(validator);
        require(keccak256(bytes(result)) == keccak256(bytes(DEFAULT_NET_ADDR)), "result == DEFAULT_NET_ADDR");

        vm.prank(validator);
        saManager.setValidatorNetAddr(newNetAddr);

        result = saGetter.validatorNetAddr(validator);
        require(keccak256(bytes(result)) == keccak256(bytes(newNetAddr)), "netAddr == newNetAddr");
    }

    function testSubnetActorDiamond_SetValidatorNetAddr_Fails_NotValidator() public payable {
        address validator = address(1235);

        _assertJoin(validator, DEFAULT_MIN_VALIDATOR_STAKE);

        string memory result = saGetter.validatorNetAddr(validator);
        require(keccak256(bytes(result)) == keccak256(bytes(DEFAULT_NET_ADDR)), "netAddr == DEFAULT_NET_ADDR");

        vm.prank(address(1234));
        vm.expectRevert(NotValidator.selector);
        saManager.setValidatorNetAddr("newNetAddr");
    }

    function testSubnetActorDiamond_SetValidatorNetAddr_Fails_EmptyAddr() public payable {
        address validator = address(1235);

        _assertJoin(validator, DEFAULT_MIN_VALIDATOR_STAKE);

        vm.prank(validator);
        vm.expectRevert(EmptyAddress.selector);
        saManager.setValidatorNetAddr("");
    }

    function testSubnetActorDiamond_SetValidatorWorkerAddr_Works() public payable {
        address validator = address(1235);
        FvmAddress memory newWorkerAddr = FvmAddressHelper.from(validator);
        FvmAddress memory defaultWorkerAddr = FvmAddress({addrType: 1, payload: new bytes(20)});

        _assertJoin(validator, DEFAULT_MIN_VALIDATOR_STAKE);

        FvmAddress memory result = saGetter.validatorWorkerAddr(validator);
        require(result.addrType == defaultWorkerAddr.addrType);
        require(keccak256(result.payload) == keccak256(defaultWorkerAddr.payload));

        vm.prank(validator);
        saManager.setValidatorWorkerAddr(newWorkerAddr);

        result = saGetter.validatorWorkerAddr(validator);
        require(result.addrType == newWorkerAddr.addrType);
        require(keccak256(result.payload) == keccak256(newWorkerAddr.payload));
    }

    function testSubnetActorDiamond_SetValidatorWorkerAddr_Fails_NotValidator() public payable {
        address validator = address(1235);
        FvmAddress memory workerAddr = FvmAddressHelper.from(validator);
        _assertJoin(validator, DEFAULT_MIN_VALIDATOR_STAKE);

        vm.prank(address(1234));
        vm.expectRevert(NotValidator.selector);
        saManager.setValidatorWorkerAddr(workerAddr);
    }

    function callback() public view {
        // console.log("callback called");
    }

    function testSubnetActorDiamond_Reward_Works_SingleValidator() public {
        address validator = vm.addr(100);

        _assertJoin(validator, DEFAULT_MIN_VALIDATOR_STAKE);

        vm.startPrank(GATEWAY_ADDRESS);
        vm.deal(GATEWAY_ADDRESS, 1 ether);

        saManager.reward(1 ether);

        require(saGetter.accumulatedRewards(validator) == 1 ether);
        // sa.reward{value: 1}();

        // require(validator.balance == balanceBefore + 1);
    }

    function testSubnetActorDiamond_Reward_Works_MultipleValidators() public {
        address validator1 = vm.addr(100);
        address validator2 = vm.addr(101);

        _assertJoin(validator1, DEFAULT_MIN_VALIDATOR_STAKE);
        _assertJoin(validator2, DEFAULT_MIN_VALIDATOR_STAKE);

        uint256 validator1BalanceBefore = saGetter.accumulatedRewards(validator1);
        uint256 validator2BalanceBefore = saGetter.accumulatedRewards(validator2);

        vm.startPrank(GATEWAY_ADDRESS);
        vm.deal(GATEWAY_ADDRESS, 1 ether);

        saManager.reward(110);

        require(saGetter.accumulatedRewards(validator1) - validator1BalanceBefore == 55);
        require(saGetter.accumulatedRewards(validator2) - validator2BalanceBefore == 55);
    }

    function testSubnetActorDiamond_Reward_Fails_NoValidatorsInSubnet() public {
        vm.startPrank(GATEWAY_ADDRESS);
        vm.deal(GATEWAY_ADDRESS, 1 ether);
        vm.expectRevert(NoValidatorsInSubnet.selector);

        saManager.reward(1 ether);
    }

    function testSubnetActorDiamond_Reward_Fails_NotGateway() public {
        address notGatewayAddr = vm.addr(101);

        vm.startPrank(notGatewayAddr);
        vm.deal(notGatewayAddr, 1 ether);
        vm.expectRevert(NotGateway.selector);

        saManager.reward(1 ether);
    }

    function testSubnetActorDiamond_Reward_Fails_NotEnoughBalanceForRewards() public {
        _assertJoin(vm.addr(100), DEFAULT_MIN_VALIDATOR_STAKE);
        _assertJoin(vm.addr(101), DEFAULT_MIN_VALIDATOR_STAKE);

        vm.startPrank(GATEWAY_ADDRESS);
        vm.deal(GATEWAY_ADDRESS, 1 ether);
        vm.expectRevert(NotEnoughBalanceForRewards.selector);

        saManager.reward(1);
    }

    function testSubnetActorDiamond_Withdraw_Fails_NoRewardToWithdraw() public {
        address validator = vm.addr(100);
        _assertJoin(validator, DEFAULT_MIN_VALIDATOR_STAKE);

        vm.prank(validator);
        vm.expectRevert(NoRewardToWithdraw.selector);

        saManager.withdraw();
    }

    function testSubnetActorDiamond_Withdraw_Works() public {
        address validator = vm.addr(100);
        _assertJoin(validator, DEFAULT_MIN_VALIDATOR_STAKE);

        vm.deal(GATEWAY_ADDRESS, 1 ether + 1);
        vm.prank(GATEWAY_ADDRESS);
        saManager.reward(1 ether);

        uint256 balanceBefore = validator.balance;
        vm.prank(validator);
        saManager.withdraw();

        require(validator.balance == balanceBefore + 1 ether);
        require(saGetter.accumulatedRewards(validator) == 0);
    }

    function _assertJoin(address validator, uint256 amount) internal {
        vm.startPrank(validator);
        vm.deal(validator, amount + 1);

        uint256 balanceBefore = validator.balance;
        uint256 stakeBefore = saGetter.stake(validator);
        uint256 totalStakeBefore = saGetter.totalStake();

        saManager.join{value: amount}(DEFAULT_NET_ADDR, FvmAddress({addrType: 1, payload: new bytes(20)}));

        require(saGetter.stake(validator) == stakeBefore + amount);
        require(saGetter.totalStake() == totalStakeBefore + amount);
        require(validator.balance == balanceBefore - amount);

        vm.stopPrank();
    }

    function _assertLeave(address validator, uint256 amount) internal {
        uint256 validatorBalanceBefore = validator.balance;
        uint256 validatorsCountBefore = saGetter.validatorCount();
        uint256 totalStakeBefore = saGetter.totalStake();

        vm.prank(validator);
        vm.expectCall(GATEWAY_ADDRESS, abi.encodeWithSelector(gwManager.releaseStake.selector, amount));
        vm.expectCall(validator, amount, EMPTY_BYTES);

        saManager.leave();

        require(saGetter.stake(validator) == 0);
        require(saGetter.totalStake() == totalStakeBefore - amount);
        require(saGetter.validatorCount() == validatorsCountBefore - 1);
        require(validator.balance == validatorBalanceBefore + amount);
    }

    function _assertKill(address validator) internal {
        vm.startPrank(validator);
        vm.deal(validator, 1 ether);
        vm.expectCall(GATEWAY_ADDRESS, abi.encodeWithSelector(gwManager.kill.selector));

        saManager.kill();

        require(saGetter.totalStake() == 0);
        require(saGetter.validatorCount() == 0);
        require(saGetter.status() == Status.Killed);

        vm.stopPrank();
    }

    function _assertDeploySubnetActor(
        bytes32 _name,
        address _ipcGatewayAddr,
        ConsensusType _consensus,
        uint256 _minActivationCollateral,
        uint64 _minValidators,
        uint64 _checkPeriod,
        uint8 _majorityPercentage
    ) public {
        SubnetID memory _parentId = gwGetter.getNetworkName();

        saManager = new SubnetActorManagerFacet();
        saGetter = new SubnetActorGetterFacet();

        IDiamond.FacetCut[] memory diamondCut = new IDiamond.FacetCut[](2);

        diamondCut[0] = (
            IDiamond.FacetCut({
                facetAddress: address(saManager),
                action: IDiamond.FacetCutAction.Add,
                functionSelectors: saManagerSelectors
            })
        );

        diamondCut[1] = (
            IDiamond.FacetCut({
                facetAddress: address(saGetter),
                action: IDiamond.FacetCutAction.Add,
                functionSelectors: saGetterSelectors
            })
        );

        saDiamond = new SubnetActorDiamond(
            diamondCut,
            SubnetActorDiamond.ConstructorParams({
                parentId: _parentId,
                name: _name,
                ipcGatewayAddr: _ipcGatewayAddr,
                consensus: _consensus,
                minActivationCollateral: _minActivationCollateral,
                minValidators: _minValidators,
                bottomUpCheckPeriod: _checkPeriod,
                topDownCheckPeriod: _checkPeriod,
                majorityPercentage: _majorityPercentage
            })
        );

        saManager = SubnetActorManagerFacet(address(saDiamond));
        saGetter = SubnetActorGetterFacet(address(saDiamond));

        require(
            keccak256(abi.encodePacked(saGetter.name())) == keccak256(abi.encodePacked(_name)),
            "keccak256(abi.encodePacked(saGetter.name())) == keccak256(abi.encodePacked(_networkName))"
        );
        require(saGetter.ipcGatewayAddr() == _ipcGatewayAddr, "saGetter.ipcGatewayAddr() == _ipcGatewayAddr");
        require(
            saGetter.minActivationCollateral() == _minActivationCollateral,
            "saGetter.minActivationCollateral() == _minActivationCollateral"
        );
        require(saGetter.minValidators() == _minValidators, "saGetter.minValidators() == _minValidators");
        require(saGetter.topDownCheckPeriod() == _checkPeriod, "saGetter.topDownCheckPeriod() == _checkPeriod");
        require(saGetter.consensus() == _consensus);
        require(
            saGetter.getParent().toHash() == _parentId.toHash(),
            "parent.toHash() == SubnetID({root: ROOTNET_CHAINID, route: path}).toHash()"
        );
    }

    function invariant_BalanceEqualsTotalStake() public {
        assertEq(address(gatewayDiamond).balance, saGetter.totalStake());
        assertEq(address(saManager).balance, 0);
    }
}
