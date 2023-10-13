// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity 0.8.19;

import "forge-std/Test.sol";
import "forge-std/StdInvariant.sol";
import "../src/errors/IPCErrors.sol";
import {TestUtils} from "./TestUtils.sol";
import {EMPTY_BYTES, METHOD_SEND, EMPTY_HASH} from "../src/constants/Constants.sol";
import {ConsensusType} from "../src/enums/ConsensusType.sol";
import {Status} from "../src/enums/Status.sol";
import {IDiamond} from "../src/interfaces/IDiamond.sol";
import {IDiamondLoupe} from "../src/interfaces/IDiamondLoupe.sol";
import {IDiamondCut} from "../src/interfaces/IDiamondCut.sol";
import {IPCMsgType} from "../src/enums/IPCMsgType.sol";
import {ISubnetActor} from "../src/interfaces/ISubnetActor.sol";
import {CheckpointInfo} from "../src/structs/Checkpoint.sol";
import {CrossMsg, BottomUpCheckpoint, StorableMsg, ParentFinality} from "../src/structs/Checkpoint.sol";
import {FvmAddress} from "../src/structs/FvmAddress.sol";
import {SubnetID, Subnet, IPCAddress, Membership, Validator, StakingChange, StakingChangeRequest, StakingOperation} from "../src/structs/Subnet.sol";
import {SubnetIDHelper} from "../src/lib/SubnetIDHelper.sol";
import {FvmAddressHelper} from "../src/lib/FvmAddressHelper.sol";
import {CheckpointHelper} from "../src/lib/CheckpointHelper.sol";
import {CrossMsgHelper} from "../src/lib/CrossMsgHelper.sol";
import {StorableMsgHelper} from "../src/lib/StorableMsgHelper.sol";
import {FilAddress} from "fevmate/utils/FilAddress.sol";
import {GatewayDiamond} from "../src/GatewayDiamond.sol";
import {SubnetActorDiamond} from "../src/SubnetActorDiamond.sol";
import {GatewayGetterFacet} from "../src/gateway/GatewayGetterFacet.sol";
import {GatewayMessengerFacet} from "../src/gateway/GatewayMessengerFacet.sol";
import {GatewayManagerFacet} from "../src/gateway/GatewayManagerFacet.sol";
import {GatewayRouterFacet} from "../src/gateway/GatewayRouterFacet.sol";
import {SubnetActorManagerFacet} from "../src/subnet/SubnetActorManagerFacet.sol";
import {SubnetActorGetterFacet} from "../src/subnet/SubnetActorGetterFacet.sol";
import {DiamondLoupeFacet} from "../src/diamond/DiamondLoupeFacet.sol";
import {MerkleTreeHelper} from "./MerkleTreeHelper.sol";

import {SubnetManagerTestUtil} from "./subnetActorMock/SubnetManagerTestUtil.sol";

contract GatewayDiamondDeploymentTest is StdInvariant, Test {
    using SubnetIDHelper for SubnetID;
    using CheckpointHelper for BottomUpCheckpoint;
    using CrossMsgHelper for CrossMsg;
    using StorableMsgHelper for StorableMsg;
    using FvmAddressHelper for FvmAddress;

    uint64 constant MAX_NONCE = type(uint64).max;
    address constant BLS_ACCOUNT_ADDREESS = address(0xfF000000000000000000000000000000bEefbEEf);
    bytes32 private constant DEFAULT_NETWORK_NAME = bytes32("test");
    uint64 private constant DEFAULT_MIN_VALIDATORS = 1;
    uint8 private constant DEFAULT_MAJORITY_PERCENTAGE = 70;
    uint64 constant DEFAULT_COLLATERAL_AMOUNT = 1 ether;
    uint64 constant DEFAULT_CHECKPOINT_PERIOD = 10;
    string private constant DEFAULT_NET_ADDR = "netAddr";
    bytes private constant GENESIS = EMPTY_BYTES;
    uint256 constant CROSS_MSG_FEE = 10 gwei;
    uint256 constant DEFAULT_RELAYER_REWARD = 10 gwei;
    address constant CHILD_NETWORK_ADDRESS = address(10);
    address constant CHILD_NETWORK_ADDRESS_2 = address(11);
    uint64 constant EPOCH_ONE = 1 * DEFAULT_CHECKPOINT_PERIOD;
    uint256 constant INITIAL_VALIDATOR_FUNDS = 1 ether;

    bytes4[] gwRouterSelectors;
    bytes4[] gwManagerSelectors;
    bytes4[] gwGetterSelectors;
    bytes4[] gwMessengerSelectors;

    GatewayDiamond gatewayDiamond;
    GatewayManagerFacet gwManager;
    GatewayGetterFacet gwGetter;
    GatewayRouterFacet gwRouter;
    GatewayMessengerFacet gwMessenger;

    GatewayDiamond gatewayDiamond2;
    GatewayManagerFacet gwManager2;
    GatewayGetterFacet gwGetter2;
    GatewayRouterFacet gwRouter2;
    GatewayMessengerFacet gwMessenger2;

    bytes4[] saGetterSelectors;
    bytes4[] saManagerSelectors;
    SubnetActorDiamond saDiamond;
    SubnetManagerTestUtil saManager;
    SubnetActorGetterFacet saGetter;

    bytes4[] louperSelectors;
    DiamondLoupeFacet louper;

    uint64 private constant ROOTNET_CHAINID = 123;
    address public constant ROOTNET_ADDRESS = address(1);

    address private constant TOPDOWN_VALIDATOR_1 = address(12);

    constructor() {
        saGetterSelectors = TestUtils.generateSelectors(vm, "SubnetActorGetterFacet");
        saManagerSelectors = TestUtils.generateSelectors(vm, "SubnetManagerTestUtil");

        gwRouterSelectors = TestUtils.generateSelectors(vm, "GatewayRouterFacet");
        gwGetterSelectors = TestUtils.generateSelectors(vm, "GatewayGetterFacet");
        gwManagerSelectors = TestUtils.generateSelectors(vm, "GatewayManagerFacet");
        gwMessengerSelectors = TestUtils.generateSelectors(vm, "GatewayMessengerFacet");

        louperSelectors = TestUtils.generateSelectors(vm, "DiamondLoupeFacet");
    }

    function createDiamond(GatewayDiamond.ConstructorParams memory params) public returns (GatewayDiamond) {
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

    function setUp() public {
        address[] memory path = new address[](1);
        path[0] = ROOTNET_ADDRESS;

        address[] memory path2 = new address[](2);
        path2[0] = CHILD_NETWORK_ADDRESS;
        path2[1] = CHILD_NETWORK_ADDRESS_2;

        // create a gateway actor.

        GatewayDiamond.ConstructorParams memory gwConstructorParams = GatewayDiamond.ConstructorParams({
            networkName: SubnetID({root: ROOTNET_CHAINID, route: new address[](0)}),
            bottomUpCheckPeriod: DEFAULT_CHECKPOINT_PERIOD,
            msgFee: CROSS_MSG_FEE,
            minCollateral: DEFAULT_COLLATERAL_AMOUNT,
            majorityPercentage: DEFAULT_MAJORITY_PERCENTAGE,
            genesisValidators: new Validator[](0),
            activeValidatorsLimit: 100
        });

        gwRouter = new GatewayRouterFacet();
        gwManager = new GatewayManagerFacet();
        gwGetter = new GatewayGetterFacet();
        gwMessenger = new GatewayMessengerFacet();
        louper = new DiamondLoupeFacet();

        IDiamond.FacetCut[] memory gwDiamondCut = new IDiamond.FacetCut[](5);

        gwDiamondCut[0] = (
            IDiamond.FacetCut({
                facetAddress: address(gwRouter),
                action: IDiamond.FacetCutAction.Add,
                functionSelectors: gwRouterSelectors
            })
        );

        gwDiamondCut[1] = (
            IDiamond.FacetCut({
                facetAddress: address(gwManager),
                action: IDiamond.FacetCutAction.Add,
                functionSelectors: gwManagerSelectors
            })
        );

        gwDiamondCut[2] = (
            IDiamond.FacetCut({
                facetAddress: address(gwGetter),
                action: IDiamond.FacetCutAction.Add,
                functionSelectors: gwGetterSelectors
            })
        );

        gwDiamondCut[3] = (
            IDiamond.FacetCut({
                facetAddress: address(gwMessenger),
                action: IDiamond.FacetCutAction.Add,
                functionSelectors: gwMessengerSelectors
            })
        );

        gwDiamondCut[4] = (
            IDiamond.FacetCut({
                facetAddress: address(louper),
                action: IDiamond.FacetCutAction.Add,
                functionSelectors: louperSelectors
            })
        );

        gatewayDiamond = new GatewayDiamond(gwDiamondCut, gwConstructorParams);
        gwGetter = GatewayGetterFacet(address(gatewayDiamond));
        gwManager = GatewayManagerFacet(address(gatewayDiamond));
        gwRouter = GatewayRouterFacet(address(gatewayDiamond));
        gwMessenger = GatewayMessengerFacet(address(gatewayDiamond));
        louper = DiamondLoupeFacet(address(gatewayDiamond));

        gwConstructorParams.networkName = SubnetID({root: ROOTNET_CHAINID, route: path2});

        gatewayDiamond2 = new GatewayDiamond(gwDiamondCut, gwConstructorParams);
        gwGetter2 = GatewayGetterFacet(address(gatewayDiamond2));
        gwManager2 = GatewayManagerFacet(address(gatewayDiamond2));
        gwRouter2 = GatewayRouterFacet(address(gatewayDiamond2));
        gwMessenger2 = GatewayMessengerFacet(address(gatewayDiamond2));

        // create a subnet actor.

        SubnetActorDiamond.ConstructorParams memory saConstructorParams = SubnetActorDiamond.ConstructorParams({
            parentId: SubnetID({root: ROOTNET_CHAINID, route: new address[](0)}),
            name: DEFAULT_NETWORK_NAME,
            ipcGatewayAddr: address(gatewayDiamond),
            consensus: ConsensusType.Fendermint,
            minActivationCollateral: DEFAULT_COLLATERAL_AMOUNT,
            minValidators: DEFAULT_MIN_VALIDATORS,
            bottomUpCheckPeriod: DEFAULT_CHECKPOINT_PERIOD,
            majorityPercentage: DEFAULT_MAJORITY_PERCENTAGE,
            activeValidatorsLimit: 100,
            powerScale: 12
        });

        saManager = new SubnetManagerTestUtil();
        saGetter = new SubnetActorGetterFacet();

        IDiamond.FacetCut[] memory saDiamondCut = new IDiamond.FacetCut[](2);

        saDiamondCut[0] = (
            IDiamond.FacetCut({
                facetAddress: address(saGetter),
                action: IDiamond.FacetCutAction.Add,
                functionSelectors: saGetterSelectors
            })
        );

        saDiamondCut[1] = (
            IDiamond.FacetCut({
                facetAddress: address(saManager),
                action: IDiamond.FacetCutAction.Add,
                functionSelectors: saManagerSelectors
            })
        );

        saDiamond = new SubnetActorDiamond(saDiamondCut, saConstructorParams);
        saManager = SubnetManagerTestUtil(address(saDiamond));
        saGetter = SubnetActorGetterFacet(address(saDiamond));

        addValidator(TOPDOWN_VALIDATOR_1, 100);
    }

    function invariant_CrossMsgFee() public view {
        require(gwGetter.crossMsgFee() == CROSS_MSG_FEE, "gw.crossMsgFee == CROSS_MSG_FEE");
    }

    function testGatewayDiamond_Constructor() public view {
        require(gwGetter.totalSubnets() == 0, "totalSubnets");
        require(gwGetter.bottomUpNonce() == 0, "bottomUpNonce");
        require(gwGetter.minStake() == DEFAULT_COLLATERAL_AMOUNT, "minStake");

        require(gwGetter.crossMsgFee() == CROSS_MSG_FEE, "crossMsgFee");
        require(gwGetter.bottomUpCheckPeriod() == DEFAULT_CHECKPOINT_PERIOD, "bottomUpCheckPeriod");
        require(
            gwGetter.getNetworkName().equals(SubnetID({root: ROOTNET_CHAINID, route: new address[](0)})),
            "getNetworkName"
        );
        require(gwGetter.majorityPercentage() == DEFAULT_MAJORITY_PERCENTAGE, "majorityPercentage");

        (StorableMsg memory storableMsg, bool wrapped) = gwGetter.postbox(0);
        StorableMsg memory msg1;
        require(msg1.toHash() == storableMsg.toHash());
        require(!wrapped);
    }

    function testGatewayDiamond_LoupeFunction() public view {
        require(louper.facets().length == 5);
    }

    function testGatewayDiamond_Deployment_Works_Root(uint64 checkpointPeriod) public {
        vm.assume(checkpointPeriod >= DEFAULT_CHECKPOINT_PERIOD);

        GatewayDiamond dep;
        GatewayManagerFacet depManager = new GatewayManagerFacet();
        GatewayGetterFacet depGetter = new GatewayGetterFacet();
        GatewayRouterFacet depRouter = new GatewayRouterFacet();

        GatewayDiamond.ConstructorParams memory constructorParams = GatewayDiamond.ConstructorParams({
            networkName: SubnetID({root: ROOTNET_CHAINID, route: new address[](0)}),
            bottomUpCheckPeriod: checkpointPeriod,
            msgFee: CROSS_MSG_FEE,
            minCollateral: DEFAULT_COLLATERAL_AMOUNT,
            majorityPercentage: DEFAULT_MAJORITY_PERCENTAGE,
            genesisValidators: new Validator[](0),
            activeValidatorsLimit: 100
        });

        dep = createDiamond(constructorParams);
        depGetter = GatewayGetterFacet(address(dep));
        depManager = GatewayManagerFacet(address(dep));
        depRouter = GatewayRouterFacet(address(dep));

        SubnetID memory networkName = depGetter.getNetworkName();

        require(networkName.isRoot(), "networkName.isRoot()");
        require(depGetter.minStake() == DEFAULT_COLLATERAL_AMOUNT, "gw.minStake() == MIN_COLLATERAL_AMOUNT");
        require(depGetter.bottomUpCheckPeriod() == checkpointPeriod, "gw.bottomUpCheckPeriod() == checkpointPeriod");
        require(
            depGetter.majorityPercentage() == DEFAULT_MAJORITY_PERCENTAGE,
            "gw.majorityPercentage() == DEFAULT_MAJORITY_PERCENTAGE"
        );
    }

    function testGatewayDiamond_Deployment_Works_NotRoot(uint64 checkpointPeriod) public {
        vm.assume(checkpointPeriod >= DEFAULT_CHECKPOINT_PERIOD);

        address[] memory path = new address[](2);
        path[0] = address(0);
        path[1] = address(1);

        GatewayDiamond dep;
        GatewayManagerFacet depManager = new GatewayManagerFacet();
        GatewayGetterFacet depGetter = new GatewayGetterFacet();
        GatewayRouterFacet depRouter = new GatewayRouterFacet();

        GatewayDiamond.ConstructorParams memory constructorParams = GatewayDiamond.ConstructorParams({
            networkName: SubnetID({root: ROOTNET_CHAINID, route: path}),
            bottomUpCheckPeriod: checkpointPeriod,
            msgFee: CROSS_MSG_FEE,
            minCollateral: DEFAULT_COLLATERAL_AMOUNT,
            majorityPercentage: 100,
            genesisValidators: new Validator[](0),
            activeValidatorsLimit: 100
        });

        IDiamond.FacetCut[] memory diamondCut = new IDiamond.FacetCut[](3);

        diamondCut[0] = (
            IDiamond.FacetCut({
                facetAddress: address(depRouter),
                action: IDiamond.FacetCutAction.Add,
                functionSelectors: gwRouterSelectors
            })
        );

        diamondCut[1] = (
            IDiamond.FacetCut({
                facetAddress: address(depManager),
                action: IDiamond.FacetCutAction.Add,
                functionSelectors: gwManagerSelectors
            })
        );

        diamondCut[2] = (
            IDiamond.FacetCut({
                facetAddress: address(depGetter),
                action: IDiamond.FacetCutAction.Add,
                functionSelectors: gwGetterSelectors
            })
        );

        dep = new GatewayDiamond(diamondCut, constructorParams);

        depGetter = GatewayGetterFacet(address(dep));
        depManager = GatewayManagerFacet(address(dep));
        depRouter = GatewayRouterFacet(address(dep));

        SubnetID memory networkName = depGetter.getNetworkName();

        require(networkName.isRoot() == false, "networkName.isRoot()");
        require(depGetter.minStake() == DEFAULT_COLLATERAL_AMOUNT, "gw.minStake() == MIN_COLLATERAL_AMOUNT");
        require(depGetter.bottomUpCheckPeriod() == checkpointPeriod, "gw.bottomUpCheckPeriod() == checkpointPeriod");
        require(depGetter.majorityPercentage() == 100, "gw.majorityPercentage() == 100");
    }

    function testGatewayDiamond_Register_Works_SingleSubnet(uint256 subnetCollateral) public {
        vm.assume(subnetCollateral >= DEFAULT_COLLATERAL_AMOUNT && subnetCollateral < type(uint64).max);
        address subnetAddress = vm.addr(100);
        vm.prank(subnetAddress);
        vm.deal(subnetAddress, subnetCollateral);

        registerSubnet(subnetCollateral, subnetAddress);
        require(gwGetter.totalSubnets() == 1);
        Subnet[] memory subnets = gwGetter.listSubnets();
        require(subnets.length == 1, "subnets.length == 1");

        SubnetID memory subnetId = gwGetter.getNetworkName().createSubnetId(subnetAddress);

        (bool ok, Subnet memory targetSubnet) = gwGetter.getSubnet(subnetId);

        require(ok, "subnet found");

        (SubnetID memory id, uint256 stake, , , , Status status) = getSubnet(subnetAddress);

        require(targetSubnet.status == Status.Active);
        require(targetSubnet.status == status);
        require(targetSubnet.stake == stake);
        require(targetSubnet.stake == subnetCollateral);
        require(id.equals(subnetId));
    }

    function testGatewayDiamond_Register_Works_MultipleSubnets(uint8 numberOfSubnets) public {
        vm.assume(numberOfSubnets > 0);

        for (uint256 i = 1; i <= numberOfSubnets; i++) {
            address subnetAddress = vm.addr(i);
            vm.prank(subnetAddress);
            vm.deal(subnetAddress, DEFAULT_COLLATERAL_AMOUNT);

            registerSubnet(DEFAULT_COLLATERAL_AMOUNT, subnetAddress);
        }

        require(gwGetter.totalSubnets() == numberOfSubnets);
        Subnet[] memory subnets = gwGetter.listSubnets();
        require(subnets.length == numberOfSubnets, "subnets.length == numberOfSubnets");
    }

    function testGatewayDiamond_Register_Fail_InsufficientCollateral(uint256 collateral) public {
        vm.assume(collateral < DEFAULT_COLLATERAL_AMOUNT);
        vm.expectRevert(NotEnoughFunds.selector);

        gwManager.register{value: collateral}();
    }

    function testGatewayDiamond_Register_Fail_SubnetAlreadyExists() public {
        registerSubnet(DEFAULT_COLLATERAL_AMOUNT, address(this));

        vm.expectRevert(AlreadyRegisteredSubnet.selector);

        gwManager.register{value: DEFAULT_COLLATERAL_AMOUNT}();
    }

    function testGatewayDiamond_AddStake_Works_SingleStaking(uint256 stakeAmount, uint256 registerAmount) public {
        address subnetAddress = vm.addr(100);
        vm.assume(registerAmount >= DEFAULT_COLLATERAL_AMOUNT && registerAmount < type(uint64).max);
        vm.assume(stakeAmount > 0 && stakeAmount < type(uint256).max - registerAmount);

        uint256 totalAmount = stakeAmount + registerAmount;

        vm.startPrank(subnetAddress);
        vm.deal(subnetAddress, totalAmount);

        registerSubnet(registerAmount, subnetAddress);
        addStake(stakeAmount, subnetAddress);

        (, uint256 totalStaked, , , , ) = getSubnet(subnetAddress);

        require(totalStaked == totalAmount);
    }

    function testGatewayDiamond_AddStake_Works_Reactivate() public {
        address subnetAddress = vm.addr(100);
        uint256 registerAmount = DEFAULT_COLLATERAL_AMOUNT;
        uint256 stakeAmount = DEFAULT_COLLATERAL_AMOUNT;

        vm.startPrank(subnetAddress);
        vm.deal(subnetAddress, registerAmount);

        registerSubnet(registerAmount, subnetAddress);
        gwManager.releaseStake(registerAmount);

        (, , , , , Status statusInactive) = getSubnet(subnetAddress);
        require(statusInactive == Status.Inactive);

        vm.deal(subnetAddress, stakeAmount);
        addStake(stakeAmount, subnetAddress);

        (, uint256 staked, , , , Status statusActive) = getSubnet(subnetAddress);

        require(staked == stakeAmount);
        require(statusActive == Status.Active);
    }

    function testGatewayDiamond_AddStake_Works_NotEnoughFundsToReactivate() public {
        address subnetAddress = vm.addr(100);
        uint256 registerAmount = DEFAULT_COLLATERAL_AMOUNT;
        uint256 stakeAmount = DEFAULT_COLLATERAL_AMOUNT - 1;

        vm.startPrank(subnetAddress);
        vm.deal(subnetAddress, registerAmount);

        registerSubnet(registerAmount, subnetAddress);
        gwManager.releaseStake(registerAmount);

        vm.deal(subnetAddress, stakeAmount);
        addStake(stakeAmount, subnetAddress);

        (, uint256 staked, , , , Status status) = getSubnet(subnetAddress);

        require(staked == stakeAmount);
        require(status == Status.Inactive);
    }

    function testGatewayDiamond_AddStake_Works_MultipleStakings(uint8 numberOfStakes) public {
        vm.assume(numberOfStakes > 0);

        address subnetAddress = address(1);
        uint256 singleStakeAmount = 1 ether;
        uint256 registerAmount = DEFAULT_COLLATERAL_AMOUNT;
        uint256 expectedStakedAmount = registerAmount;

        vm.startPrank(subnetAddress);
        vm.deal(subnetAddress, registerAmount + singleStakeAmount * numberOfStakes);

        registerSubnet(registerAmount, subnetAddress);

        for (uint256 i = 0; i < numberOfStakes; i++) {
            addStake(singleStakeAmount, subnetAddress);

            expectedStakedAmount += singleStakeAmount;
        }

        (, uint256 totalStake, , , , ) = getSubnet(subnetAddress);

        require(totalStake == expectedStakedAmount);
    }

    function testGatewayDiamond_AddStake_Fail_ZeroAmount() public {
        registerSubnet(DEFAULT_COLLATERAL_AMOUNT, address(this));

        vm.expectRevert(NotEnoughFunds.selector);

        gwManager.addStake{value: 0}();
    }

    function testGatewayDiamond_AddStake_Fail_SubnetNotExists() public {
        vm.expectRevert(NotRegisteredSubnet.selector);

        gwManager.addStake{value: 1}();
    }

    function testGatewayDiamond_ReleaseStake_Works_FullAmount(uint256 stakeAmount) public {
        address subnetAddress = CHILD_NETWORK_ADDRESS;
        uint256 registerAmount = DEFAULT_COLLATERAL_AMOUNT;

        vm.assume(stakeAmount > 0 && stakeAmount < type(uint256).max - registerAmount);

        uint256 fullAmount = stakeAmount + registerAmount;

        vm.startPrank(subnetAddress);
        vm.deal(subnetAddress, fullAmount);

        registerSubnet(registerAmount, subnetAddress);
        addStake(stakeAmount, subnetAddress);

        gwManager.releaseStake(fullAmount);

        (, uint256 stake, , , , Status status) = getSubnet(subnetAddress);

        require(stake == 0);
        require(status == Status.Inactive);
        require(subnetAddress.balance == fullAmount);
    }

    function testGatewayDiamond_ReleaseStake_Works_SubnetInactive() public {
        address subnetAddress = vm.addr(100);
        vm.startPrank(subnetAddress);
        vm.deal(subnetAddress, DEFAULT_COLLATERAL_AMOUNT);
        registerSubnet(DEFAULT_COLLATERAL_AMOUNT, subnetAddress);

        gwManager.releaseStake(DEFAULT_COLLATERAL_AMOUNT / 2);

        (, uint256 stake, , , , Status status) = getSubnet(subnetAddress);
        require(stake == DEFAULT_COLLATERAL_AMOUNT / 2, "stake == MIN_COLLATERAL_AMOUNT / 2");
        require(status == Status.Inactive, "status == Status.Inactive");
    }

    function testGatewayDiamond_ReleaseStake_Works_PartialAmount(uint256 partialAmount) public {
        address subnetAddress = CHILD_NETWORK_ADDRESS;
        uint256 registerAmount = DEFAULT_COLLATERAL_AMOUNT;

        vm.assume(partialAmount > registerAmount && partialAmount < type(uint256).max - registerAmount);

        uint256 totalAmount = partialAmount + registerAmount;

        vm.startPrank(subnetAddress);
        vm.deal(subnetAddress, totalAmount);

        registerSubnet(registerAmount, subnetAddress);
        addStake(partialAmount, subnetAddress);

        gwManager.releaseStake(partialAmount);

        (, uint256 stake, , , , Status status) = getSubnet(subnetAddress);

        require(stake == registerAmount);
        require(status == Status.Active);
        require(subnetAddress.balance == partialAmount);
    }

    function testGatewayDiamond_ReleaseStake_Fail_ZeroAmount() public {
        registerSubnet(DEFAULT_COLLATERAL_AMOUNT, address(this));

        vm.expectRevert(CannotReleaseZero.selector);

        gwManager.releaseStake(0);
    }

    function testGatewayDiamond_ReleaseStake_Fail_InsufficientSubnetBalance(
        uint256 releaseAmount,
        uint256 subnetBalance
    ) public {
        vm.assume(subnetBalance > DEFAULT_COLLATERAL_AMOUNT);
        vm.assume(releaseAmount > subnetBalance && releaseAmount < type(uint256).max - subnetBalance);

        address subnetAddress = vm.addr(100);
        vm.startPrank(subnetAddress);
        vm.deal(subnetAddress, releaseAmount);

        registerSubnet(subnetBalance, subnetAddress);

        vm.expectRevert(NotEnoughFundsToRelease.selector);

        gwManager.releaseStake(releaseAmount);
    }

    function testGatewayDiamond_ReleaseStake_Fail_NotRegisteredSubnet() public {
        vm.expectRevert(NotRegisteredSubnet.selector);

        gwManager.releaseStake(1);
    }

    function testGatewayDiamond_ReleaseStake_Works_TransitionToInactive() public {
        address subnetAddress = vm.addr(100);

        vm.startPrank(subnetAddress);
        vm.deal(subnetAddress, DEFAULT_COLLATERAL_AMOUNT);

        registerSubnet(DEFAULT_COLLATERAL_AMOUNT, subnetAddress);

        gwManager.releaseStake(10);

        (, uint256 stake, , , , Status status) = getSubnet(subnetAddress);

        require(stake == DEFAULT_COLLATERAL_AMOUNT - 10, "stake should be MIN_COLLATERAL_AMOUNT - 10");
        require(status == Status.Inactive, "status should be Inactive");
    }

    function testGatewayDiamond_Kill_Works() public {
        address subnetAddress = CHILD_NETWORK_ADDRESS;

        vm.startPrank(subnetAddress);
        vm.deal(subnetAddress, DEFAULT_COLLATERAL_AMOUNT);

        registerSubnet(DEFAULT_COLLATERAL_AMOUNT, subnetAddress);

        require(subnetAddress.balance == 0);

        gwManager.kill();

        (SubnetID memory id, uint256 stake, uint256 nonce, , uint256 circSupply, Status status) = getSubnet(
            subnetAddress
        );

        require(id.toHash() == SubnetID(0, new address[](0)).toHash());
        require(stake == 0);
        require(nonce == 0);
        require(circSupply == 0);
        require(status == Status.Unset);
        require(gwGetter.totalSubnets() == 0);
        require(subnetAddress.balance == DEFAULT_COLLATERAL_AMOUNT);
    }

    function testGatewayDiamond_Kill_Fail_SubnetNotExists() public {
        vm.expectRevert(NotRegisteredSubnet.selector);

        gwManager.kill();
    }

    // function testGatewayDiamond_Kill_Fail_CircSupplyMoreThanZero() public {
    //     address validatorAddress = address(100);
    //     _join(validatorAddress);

    //     address funderAddress = address(101);
    //     uint256 fundAmount = 1 ether;

    //     vm.startPrank(funderAddress);
    //     vm.deal(funderAddress, fundAmount + 1);

    //     fund(funderAddress, fundAmount);

    //     vm.stopPrank();
    //     vm.startPrank(address(saManager));
    //     vm.expectRevert(NotEmptySubnetCircSupply.selector);

    //     gwManager.kill();
    // }

    // function testGatewayDiamond_Fund_Works_ReactivatedSubnet() public {
    //     address validatorAddress = address(100);

    //     _join(validatorAddress);

    //     vm.prank(validatorAddress);
    //     saManager.leave();

    //     _join(validatorAddress);

    //     require(saGetter.status() == Status.Active);

    //     address funderAddress = address(101);
    //     uint256 fundAmount = 1 ether;

    //     vm.startPrank(funderAddress);
    //     vm.deal(funderAddress, fundAmount + 1);

    //     fund(funderAddress, fundAmount);
    // }

    // function testGatewayDiamond_Fund_Works_EthAccountSingleFunding() public {
    //     address validatorAddress = address(100);

    //     _join(validatorAddress);

    //     address funderAddress = address(101);
    //     uint256 fundAmount = 1 ether;

    //     vm.startPrank(funderAddress);
    //     vm.deal(funderAddress, fundAmount + 1);

    //     fund(funderAddress, fundAmount);
    // }

    // function testGatewayDiamond_Fund_Works_BLSAccountSingleFunding() public {
    //     address validatorAddress = address(100);
    //     _join(validatorAddress);

    //     uint256 fundAmount = 1 ether;
    //     vm.startPrank(BLS_ACCOUNT_ADDREESS);
    //     vm.deal(BLS_ACCOUNT_ADDREESS, fundAmount + 1);

    //     fund(BLS_ACCOUNT_ADDREESS, fundAmount);
    // }

    // function testGatewayDiamond_Fund_Works_MultipleFundings() public {
    //     uint8 numberOfFunds = 5;
    //     uint256 fundAmount = 1 ether;

    //     address validatorAddress = address(100);
    //     address funderAddress = address(101);

    //     _join(validatorAddress);

    //     vm.startPrank(funderAddress);

    //     for (uint256 i = 0; i < numberOfFunds; i++) {
    //         vm.deal(funderAddress, fundAmount + 1);

    //         fund(funderAddress, fundAmount);
    //     }
    // }

    // function testGatewayDiamond_Fund_Fails_WrongSubnet() public {
    //     address validatorAddress = address(100);
    //     address funderAddress = address(101);
    //     uint256 fundAmount = 1 ether;

    //     _join(validatorAddress);

    //     vm.startPrank(funderAddress);
    //     vm.deal(funderAddress, fundAmount + 1);

    //     address[] memory wrongPath = new address[](3);
    //     wrongPath[0] = address(1);
    //     wrongPath[1] = address(2);

    //     vm.expectRevert(NotRegisteredSubnet.selector);

    //     gwManager.fund{value: fundAmount}(SubnetID(ROOTNET_CHAINID, wrongPath), FvmAddressHelper.from(msg.sender));
    // }

    // function testGatewayDiamond_Fund_Fails_NotRegistered() public {
    //     address validatorAddress = address(100);
    //     address funderAddress = address(101);
    //     uint256 fundAmount = 1 ether;

    //     _join(validatorAddress);

    //     vm.startPrank(funderAddress);
    //     vm.deal(funderAddress, fundAmount + 1);

    //     address[] memory wrongSubnetPath = new address[](2);
    //     wrongSubnetPath[0] = vm.addr(102);
    //     wrongSubnetPath[0] = vm.addr(103);
    //     SubnetID memory wrongSubnetId = SubnetID({root: ROOTNET_CHAINID, route: wrongSubnetPath});

    //     vm.expectRevert(NotRegisteredSubnet.selector);
    //     gwManager.fund{value: fundAmount}(wrongSubnetId, FvmAddressHelper.from(msg.sender));
    // }

    // function testGatewayDiamond_Fund_Fails_InsufficientAmount() public {
    //     address validatorAddress = address(100);
    //     address funderAddress = address(101);

    //     _join(validatorAddress);

    //     vm.startPrank(funderAddress);
    //     vm.deal(funderAddress, 1 ether);

    //     (SubnetID memory subnetId, , , , , ) = getSubnet(address(saManager));

    //     vm.expectRevert(NotEnoughFee.selector);

    //     gwManager.fund{value: 0}(subnetId, FvmAddressHelper.from(msg.sender));
    // }

    function testGatewayDiamond_Release_Fails_InsufficientAmount() public {
        address[] memory path = new address[](2);
        path[0] = address(1);
        path[1] = address(2);

        GatewayDiamond.ConstructorParams memory constructorParams = GatewayDiamond.ConstructorParams({
            networkName: SubnetID({root: ROOTNET_CHAINID, route: path}),
            bottomUpCheckPeriod: DEFAULT_CHECKPOINT_PERIOD,
            msgFee: CROSS_MSG_FEE,
            minCollateral: DEFAULT_COLLATERAL_AMOUNT,
            majorityPercentage: DEFAULT_MAJORITY_PERCENTAGE,
            genesisValidators: new Validator[](0),
            activeValidatorsLimit: 100
        });
        gatewayDiamond = createDiamond(constructorParams);
        gwGetter = GatewayGetterFacet(address(gatewayDiamond));
        gwManager = GatewayManagerFacet(address(gatewayDiamond));
        gwRouter = GatewayRouterFacet(address(gatewayDiamond));

        address callerAddress = address(100);

        vm.startPrank(callerAddress);
        vm.deal(callerAddress, 1 ether);
        vm.expectRevert(NotEnoughFee.selector);

        gwManager.release{value: 0 ether}(FvmAddressHelper.from(msg.sender), 0);
    }

    function testGatewayDiamond_Release_Works_BLSAccount(uint256 releaseAmount, uint256 crossMsgFee) public {
        vm.assume(crossMsgFee >= CROSS_MSG_FEE);
        vm.assume(releaseAmount < type(uint256).max);
        vm.assume(crossMsgFee > 0 && crossMsgFee < releaseAmount);

        address[] memory path = new address[](2);
        path[0] = makeAddr("root");
        path[1] = makeAddr("subnet_one");

        GatewayDiamond.ConstructorParams memory constructorParams = GatewayDiamond.ConstructorParams({
            networkName: SubnetID({root: ROOTNET_CHAINID, route: path}),
            bottomUpCheckPeriod: DEFAULT_CHECKPOINT_PERIOD,
            msgFee: crossMsgFee,
            minCollateral: DEFAULT_COLLATERAL_AMOUNT,
            majorityPercentage: DEFAULT_MAJORITY_PERCENTAGE,
            genesisValidators: new Validator[](0),
            activeValidatorsLimit: 100
        });
        gatewayDiamond = createDiamond(constructorParams);
        gwGetter = GatewayGetterFacet(address(gatewayDiamond));
        gwManager = GatewayManagerFacet(address(gatewayDiamond));
        gwRouter = GatewayRouterFacet(address(gatewayDiamond));

        vm.roll(0);
        vm.warp(0);
        vm.startPrank(BLS_ACCOUNT_ADDREESS);
        vm.deal(BLS_ACCOUNT_ADDREESS, releaseAmount + 1);
        release(releaseAmount, crossMsgFee);
    }

    function testGatewayDiamond_Release_Works_EmptyCrossMsgMeta(uint256 releaseAmount, uint256 crossMsgFee) public {
        vm.assume(crossMsgFee >= CROSS_MSG_FEE);
        vm.assume(releaseAmount < type(uint256).max);
        vm.assume(crossMsgFee > 0 && crossMsgFee < releaseAmount);

        address[] memory path = new address[](2);
        path[0] = makeAddr("root");
        path[1] = makeAddr("subnet_one");

        GatewayDiamond.ConstructorParams memory constructorParams = GatewayDiamond.ConstructorParams({
            networkName: SubnetID({root: ROOTNET_CHAINID, route: path}),
            bottomUpCheckPeriod: DEFAULT_CHECKPOINT_PERIOD,
            msgFee: crossMsgFee,
            minCollateral: DEFAULT_COLLATERAL_AMOUNT,
            majorityPercentage: DEFAULT_MAJORITY_PERCENTAGE,
            genesisValidators: new Validator[](0),
            activeValidatorsLimit: 100
        });
        gatewayDiamond = createDiamond(constructorParams);
        gwGetter = GatewayGetterFacet(address(gatewayDiamond));
        gwManager = GatewayManagerFacet(address(gatewayDiamond));
        gwRouter = GatewayRouterFacet(address(gatewayDiamond));

        address callerAddress = address(100);

        vm.roll(0);
        vm.warp(0);
        vm.startPrank(callerAddress);
        vm.deal(callerAddress, releaseAmount + 1);
        release(releaseAmount, crossMsgFee);
    }

    function testGatewayDiamond_Release_Works_NonEmptyCrossMsgMeta(uint256 releaseAmount, uint256 crossMsgFee) public {
        vm.assume(crossMsgFee >= CROSS_MSG_FEE);
        vm.assume(releaseAmount < type(uint256).max / 2);
        vm.assume(crossMsgFee > 0 && crossMsgFee < releaseAmount);

        address[] memory path = new address[](2);
        path[0] = makeAddr("root");
        path[1] = makeAddr("subnet_one");

        GatewayDiamond.ConstructorParams memory constructorParams = GatewayDiamond.ConstructorParams({
            networkName: SubnetID({root: ROOTNET_CHAINID, route: path}),
            bottomUpCheckPeriod: DEFAULT_CHECKPOINT_PERIOD,
            msgFee: crossMsgFee,
            minCollateral: DEFAULT_COLLATERAL_AMOUNT,
            majorityPercentage: DEFAULT_MAJORITY_PERCENTAGE,
            genesisValidators: new Validator[](0),
            activeValidatorsLimit: 100
        });
        gatewayDiamond = createDiamond(constructorParams);
        gwGetter = GatewayGetterFacet(address(gatewayDiamond));
        gwManager = GatewayManagerFacet(address(gatewayDiamond));
        gwRouter = GatewayRouterFacet(address(gatewayDiamond));

        address callerAddress = address(100);

        vm.roll(0);
        vm.warp(0);
        vm.startPrank(callerAddress);
        vm.deal(callerAddress, 2 * releaseAmount + 1);

        release(releaseAmount, crossMsgFee);

        release(releaseAmount, crossMsgFee);
    }

    function testGatewayDiamond_SendCrossMessage_Fails_NoDestination() public {
        address caller = vm.addr(100);
        vm.startPrank(caller);
        vm.deal(caller, DEFAULT_COLLATERAL_AMOUNT + CROSS_MSG_FEE + 2);
        registerSubnet(DEFAULT_COLLATERAL_AMOUNT, caller);

        vm.expectRevert(InvalidCrossMsgDstSubnet.selector);
        gwMessenger.sendCrossMessage{value: CROSS_MSG_FEE + 1}(
            CrossMsg({
                message: StorableMsg({
                    from: IPCAddress({
                        subnetId: SubnetID({root: ROOTNET_CHAINID, route: new address[](0)}),
                        rawAddress: FvmAddressHelper.from(caller)
                    }),
                    to: IPCAddress({
                        subnetId: SubnetID({root: 0, route: new address[](0)}),
                        rawAddress: FvmAddressHelper.from(caller)
                    }),
                    value: 1,
                    nonce: 0,
                    method: METHOD_SEND,
                    params: new bytes(0),
                    fee: CROSS_MSG_FEE
                }),
                wrapped: false
            })
        );
    }

    function testGatewayDiamond_SendCrossMessage_Fails_NoCurrentNetwork() public {
        address caller = vm.addr(100);
        vm.startPrank(caller);
        vm.deal(caller, DEFAULT_COLLATERAL_AMOUNT + CROSS_MSG_FEE + 2);
        registerSubnet(DEFAULT_COLLATERAL_AMOUNT, caller);
        SubnetID memory destinationSubnet = gwGetter.getNetworkName();
        vm.expectRevert(CannotSendCrossMsgToItself.selector);
        gwMessenger.sendCrossMessage{value: CROSS_MSG_FEE + 1}(
            CrossMsg({
                message: StorableMsg({
                    from: IPCAddress({
                        subnetId: SubnetID({root: ROOTNET_CHAINID, route: new address[](0)}),
                        rawAddress: FvmAddressHelper.from(caller)
                    }),
                    to: IPCAddress({subnetId: destinationSubnet, rawAddress: FvmAddressHelper.from(caller)}),
                    value: 1,
                    nonce: 0,
                    method: METHOD_SEND,
                    params: new bytes(0),
                    fee: CROSS_MSG_FEE
                }),
                wrapped: true
            })
        );
    }

    function testGatewayDiamond_SendCrossMessage_Fails_Failes_InvalidCrossMsgValue() public {
        address caller = vm.addr(100);
        vm.startPrank(caller);
        vm.deal(caller, DEFAULT_COLLATERAL_AMOUNT + CROSS_MSG_FEE + 2);
        registerSubnet(DEFAULT_COLLATERAL_AMOUNT, caller);
        SubnetID memory destinationSubnet = gwGetter.getNetworkName().createSubnetId(caller);
        vm.expectRevert(InvalidCrossMsgValue.selector);
        gwMessenger.sendCrossMessage{value: CROSS_MSG_FEE + 1}(
            CrossMsg({
                message: StorableMsg({
                    from: IPCAddress({
                        subnetId: SubnetID({root: ROOTNET_CHAINID, route: new address[](0)}),
                        rawAddress: FvmAddressHelper.from(caller)
                    }),
                    to: IPCAddress({subnetId: destinationSubnet, rawAddress: FvmAddressHelper.from(caller)}),
                    value: 5,
                    nonce: 0,
                    method: METHOD_SEND,
                    params: new bytes(0),
                    fee: CROSS_MSG_FEE
                }),
                wrapped: true
            })
        );
    }

    function testGatewayDiamond_SendCrossMessage_Fails_EmptyNetwork() public {
        address caller = vm.addr(100);
        vm.startPrank(caller);
        vm.deal(caller, DEFAULT_COLLATERAL_AMOUNT + CROSS_MSG_FEE + 2);

        registerSubnet(DEFAULT_COLLATERAL_AMOUNT, caller);

        SubnetID memory destinationSubnet = SubnetID(0, new address[](0));
        vm.expectRevert(InvalidCrossMsgDstSubnet.selector);

        gwMessenger.sendCrossMessage{value: CROSS_MSG_FEE + 1}(
            CrossMsg({
                message: StorableMsg({
                    from: IPCAddress({
                        subnetId: SubnetID({root: ROOTNET_CHAINID, route: new address[](0)}),
                        rawAddress: FvmAddressHelper.from(caller)
                    }),
                    to: IPCAddress({subnetId: destinationSubnet, rawAddress: FvmAddressHelper.from(caller)}),
                    value: 1,
                    nonce: 0,
                    method: METHOD_SEND,
                    params: new bytes(0),
                    fee: CROSS_MSG_FEE
                }),
                wrapped: true
            })
        );
    }

    function testGatewayDiamond_SendCrossMessage_Fails_InvalidCrossMsgFromSubnet() public {
        address caller = vm.addr(100);
        vm.startPrank(caller);
        vm.deal(caller, DEFAULT_COLLATERAL_AMOUNT + CROSS_MSG_FEE + 2);
        registerSubnet(DEFAULT_COLLATERAL_AMOUNT, caller);
        SubnetID memory destinationSubnet = gwGetter.getNetworkName().createSubnetId(caller);
        vm.expectRevert(InvalidCrossMsgFromSubnet.selector);
        gwMessenger.sendCrossMessage{value: CROSS_MSG_FEE + 1}(
            CrossMsg({
                message: StorableMsg({
                    from: IPCAddress({
                        subnetId: SubnetID({root: 0, route: new address[](0)}),
                        rawAddress: FvmAddressHelper.from(caller)
                    }),
                    to: IPCAddress({subnetId: destinationSubnet, rawAddress: FvmAddressHelper.from(caller)}),
                    value: 1,
                    nonce: 0,
                    method: METHOD_SEND,
                    params: new bytes(0),
                    fee: CROSS_MSG_FEE
                }),
                wrapped: true
            })
        );
    }

    function testGatewayDiamond_SendCrossMessage_Fails_NotEnoughFee() public {
        address caller = vm.addr(100);
        vm.startPrank(caller);
        vm.deal(caller, DEFAULT_COLLATERAL_AMOUNT + CROSS_MSG_FEE);
        registerSubnet(DEFAULT_COLLATERAL_AMOUNT, caller);
        SubnetID memory destinationSubnet = gwGetter.getNetworkName().createSubnetId(caller);

        vm.expectRevert(NotEnoughFee.selector);
        gwMessenger.sendCrossMessage{value: CROSS_MSG_FEE}(
            CrossMsg({
                message: StorableMsg({
                    from: IPCAddress({
                        subnetId: SubnetID({root: ROOTNET_CHAINID, route: new address[](0)}),
                        rawAddress: FvmAddressHelper.from(caller)
                    }),
                    to: IPCAddress({subnetId: destinationSubnet, rawAddress: FvmAddressHelper.from(address(0))}),
                    value: 0,
                    nonce: 0,
                    method: METHOD_SEND,
                    params: new bytes(0),
                    fee: 0
                }),
                wrapped: true
            })
        );
    }

    function testGatewayDiamond_SendCrossMessage_Fails_NotEnoughFunds() public {
        address caller = vm.addr(100);
        vm.startPrank(caller);
        vm.deal(caller, DEFAULT_COLLATERAL_AMOUNT + CROSS_MSG_FEE);
        registerSubnet(DEFAULT_COLLATERAL_AMOUNT, caller);
        SubnetID memory destinationSubnet = gwGetter.getNetworkName().createSubnetId(caller);

        vm.expectRevert(NotEnoughFunds.selector);
        gwMessenger.sendCrossMessage{value: 0}(
            CrossMsg({
                message: StorableMsg({
                    from: IPCAddress({
                        subnetId: SubnetID({root: ROOTNET_CHAINID, route: new address[](0)}),
                        rawAddress: FvmAddressHelper.from(caller)
                    }),
                    to: IPCAddress({subnetId: destinationSubnet, rawAddress: FvmAddressHelper.from(address(0))}),
                    value: 0,
                    nonce: 0,
                    method: METHOD_SEND,
                    params: new bytes(0),
                    fee: CROSS_MSG_FEE
                }),
                wrapped: true
            })
        );
    }

    function reward(uint256 amount) external view {
        console.log("reward method called with %d", amount);
    }

    function testGatewayDiamond_Propagate_Works_WithFeeRemainder() external {
        (, address[] memory validators) = setupValidators();
        address caller = validators[0];

        bytes32 postboxId = setupWhiteListMethod(caller);

        vm.deal(caller, 1 ether);
        vm.expectCall(caller, 1 ether - gwGetter.crossMsgFee(), new bytes(0), 1);

        vm.prank(caller);
        gwMessenger.propagate{value: 1 ether}(postboxId);

        require(caller.balance == 1 ether - gwGetter.crossMsgFee());
    }

    function testGatewayDiamond_Propagate_Works_NoFeeReminder() external {
        (, address[] memory validators) = setupValidators();
        address caller = validators[0];

        uint256 fee = gwGetter.crossMsgFee();

        bytes32 postboxId = setupWhiteListMethod(caller);

        vm.deal(caller, fee);
        vm.prank(caller);

        vm.expectCall(caller, 0, EMPTY_BYTES, 0);

        gwMessenger.propagate{value: fee}(postboxId);
        require(caller.balance == 0, "caller.balance == 0");
    }

    function testGatewayDiamond_Propagate_Fails_NotEnoughFee() public {
        address caller = vm.addr(100);
        vm.deal(caller, 1 ether);

        vm.expectRevert(NotEnoughFee.selector);
        gwMessenger.propagate(bytes32(""));
    }

    function setupWhiteListMethod(address caller) internal returns (bytes32) {
        registerSubnet(DEFAULT_COLLATERAL_AMOUNT, address(this));

        CrossMsg memory crossMsg = CrossMsg({
            message: StorableMsg({
                from: IPCAddress({
                    subnetId: gwGetter.getNetworkName().createSubnetId(caller),
                    rawAddress: FvmAddressHelper.from(caller)
                }),
                to: IPCAddress({
                    subnetId: gwGetter.getNetworkName().createSubnetId(address(this)),
                    rawAddress: FvmAddressHelper.from(address(this))
                }),
                value: CROSS_MSG_FEE + 1,
                nonce: 0,
                method: METHOD_SEND,
                params: new bytes(0),
                fee: CROSS_MSG_FEE
            }),
            wrapped: false
        });
        CrossMsg[] memory msgs = new CrossMsg[](1);
        msgs[0] = crossMsg;

        // we add a validator with 10 times as much weight as the default validator.
        // This way we have 10/11 votes and we reach majority, setting the message in postbox
        // addValidator(caller, 1000);

        vm.prank(FilAddress.SYSTEM_ACTOR);
        gwRouter.applyCrossMessages(msgs);

        return crossMsg.toHash();
    }

    function testGatewayDiamond_CommitParentFinality_Fails_NotSystemActor() public {
        address caller = vm.addr(100);

        FvmAddress[] memory validators = new FvmAddress[](1);
        validators[0] = FvmAddressHelper.from(caller);
        uint256[] memory weights = new uint256[](1);
        weights[0] = 100;

        vm.prank(caller);
        vm.expectRevert(NotSystemActor.selector);

        ParentFinality memory finality = ParentFinality({height: block.number, blockHash: bytes32(0)});

        gwRouter.commitParentFinality(finality);
    }

    function testGatewayDiamond_applyFinality_works() public {
        // changes included for two validators joining
        address val1 = vm.addr(100);
        address val2 = vm.addr(101);
        uint256 amount = 10000;
        StakingChangeRequest[] memory changes = new StakingChangeRequest[](2);

        changes[0] = StakingChangeRequest({
            configurationNumber: 0,
            change: StakingChange({validator: val1, op: StakingOperation.Deposit, payload: abi.encode(amount)})
        });
        changes[1] = StakingChangeRequest({
            configurationNumber: 1,
            change: StakingChange({validator: val2, op: StakingOperation.Deposit, payload: abi.encode(amount)})
        });

        vm.startPrank(FilAddress.SYSTEM_ACTOR);

        gwRouter.storeValidatorChanges(changes);
        uint64 configNumber = gwRouter.applyFinalityChanges();
        require(configNumber == 1, "wrong config number after applying finality");
        require(gwGetter.getCurrentMembership().validators.length == 2, "current membership should be 2");

        vm.stopPrank();

        // new change with a validator leaving
        changes = new StakingChangeRequest[](1);

        changes[0] = StakingChangeRequest({
            configurationNumber: 2,
            change: StakingChange({validator: val1, op: StakingOperation.Withdraw, payload: abi.encode(amount)})
        });
        vm.startPrank(FilAddress.SYSTEM_ACTOR);

        gwRouter.storeValidatorChanges(changes);
        configNumber = gwRouter.applyFinalityChanges();
        require(configNumber == 2, "wrong config number after applying finality");
        require(gwGetter.getCurrentMembership().validators.length == 1, "current membership should be 1");
        require(gwGetter.getLastMembership().validators.length == 2, "last membership should be 2");

        // no changes
        configNumber = gwRouter.applyFinalityChanges();
        require(configNumber == 0, "wrong config number after applying finality");
        require(gwGetter.getCurrentMembership().validators.length == 1, "current membership should be 1");
        require(gwGetter.getLastMembership().validators.length == 2, "last membership should be 2");

        vm.stopPrank();
    }

    function testGatewayDiamond_CommitParentFinality_Works_WithQuery() public {
        FvmAddress[] memory validators = new FvmAddress[](2);
        validators[0] = FvmAddressHelper.from(vm.addr(100));
        validators[1] = FvmAddressHelper.from(vm.addr(101));
        uint256[] memory weights = new uint256[](2);
        weights[0] = 100;
        weights[1] = 150;

        vm.startPrank(FilAddress.SYSTEM_ACTOR);

        ParentFinality memory finality = ParentFinality({height: block.number, blockHash: bytes32(0)});

        gwRouter.commitParentFinality(finality);
        ParentFinality memory committedFinality = gwGetter.getParentFinality(block.number);

        require(committedFinality.height == finality.height, "height not equal");
        require(committedFinality.blockHash == finality.blockHash, "blockHash not equal");

        vm.stopPrank();
    }

    function testGatewayDiamond_CommitParentFinality_BigNumberOfMessages() public {
        uint256 n = 2000;
        FvmAddress[] memory validators = new FvmAddress[](1);
        validators[0] = FvmAddressHelper.from(vm.addr(100));
        vm.deal(vm.addr(100), 1);

        uint256[] memory weights = new uint[](1);
        weights[0] = 100;

        CrossMsg[] memory topDownMsgs = new CrossMsg[](n);
        for (uint64 i = 0; i < n; i++) {
            topDownMsgs[i] = CrossMsg({
                message: StorableMsg({
                    from: IPCAddress({
                        subnetId: gwGetter.getNetworkName(),
                        rawAddress: FvmAddressHelper.from(address(this))
                    }),
                    to: IPCAddress({
                        subnetId: gwGetter.getNetworkName(),
                        rawAddress: FvmAddressHelper.from(address(this))
                    }),
                    value: 0,
                    nonce: i,
                    method: this.callback.selector,
                    params: EMPTY_BYTES,
                    fee: CROSS_MSG_FEE
                }),
                wrapped: false
            });
        }

        vm.startPrank(FilAddress.SYSTEM_ACTOR);

        gwRouter.applyCrossMessages(topDownMsgs);

        vm.stopPrank();
    }

    function testGatewayDiamond_createBottomUpCheckpoint() public {
        (, address[] memory addrs, uint256[] memory weights) = setupThreeValidatorsForCheckpoint();

        (bytes32 membershipRoot, ) = MerkleTreeHelper.createMerkleProofsForValidators(addrs, weights);

        BottomUpCheckpoint memory checkpoint = BottomUpCheckpoint({
            subnetID: gwGetter.getNetworkName(),
            blockHeight: gwGetter.bottomUpCheckPeriod(),
            blockHash: keccak256("block"),
            nextConfigurationNumber: 1,
            crossMessagesHash: keccak256("messages")
        });

        // failed to create a checkpoint with zero membership weight
        vm.startPrank(FilAddress.SYSTEM_ACTOR);
        vm.expectRevert(ZeroMembershipWeight.selector);
        gwRouter.createBottomUpCheckpoint(checkpoint, membershipRoot, 0);
        vm.stopPrank();

        // create a checkpoint
        vm.startPrank(FilAddress.SYSTEM_ACTOR);
        gwRouter.createBottomUpCheckpoint(checkpoint, membershipRoot, weights[0] + weights[1] + weights[2]);
        vm.stopPrank();

        // failed to create a checkpoint with the same height
        checkpoint = BottomUpCheckpoint({
            subnetID: gwGetter.getNetworkName(),
            blockHeight: gwGetter.bottomUpCheckPeriod(),
            blockHash: keccak256("block"),
            nextConfigurationNumber: 2,
            crossMessagesHash: keccak256("newmessages")
        });

        vm.startPrank(FilAddress.SYSTEM_ACTOR);
        vm.expectRevert(CheckpointAlreadyExists.selector);
        gwRouter.createBottomUpCheckpoint(checkpoint, membershipRoot, weights[0] + weights[1] + weights[2]);
        vm.stopPrank();

        // failed to create a checkpoint with the height not multiple to checkpoint period
        checkpoint = BottomUpCheckpoint({
            subnetID: gwGetter.getNetworkName(),
            blockHeight: gwGetter.bottomUpCheckPeriod() + gwGetter.bottomUpCheckPeriod() / 2,
            blockHash: keccak256("block"),
            nextConfigurationNumber: 2,
            crossMessagesHash: keccak256("newmessages")
        });

        vm.startPrank(FilAddress.SYSTEM_ACTOR);
        vm.expectRevert(InvalidCheckpointEpoch.selector);
        gwRouter.createBottomUpCheckpoint(checkpoint, membershipRoot, weights[0] + weights[1] + weights[2]);
        vm.stopPrank();
    }

    function testGatewayDiamond_listIncompleteCheckpoints() public {
        (, address[] memory addrs, uint256[] memory weights) = setupThreeValidatorsForCheckpoint();

        (bytes32 membershipRoot, ) = MerkleTreeHelper.createMerkleProofsForValidators(addrs, weights);

        BottomUpCheckpoint memory checkpoint1 = BottomUpCheckpoint({
            subnetID: gwGetter.getNetworkName(),
            blockHeight: gwGetter.bottomUpCheckPeriod(),
            blockHash: keccak256("block1"),
            nextConfigurationNumber: 1,
            crossMessagesHash: keccak256("messages1")
        });

        BottomUpCheckpoint memory checkpoint2 = BottomUpCheckpoint({
            subnetID: gwGetter.getNetworkName(),
            blockHeight: 2 * gwGetter.bottomUpCheckPeriod(),
            blockHash: keccak256("block2"),
            nextConfigurationNumber: 1,
            crossMessagesHash: keccak256("messages2")
        });

        // create a checkpoint
        vm.startPrank(FilAddress.SYSTEM_ACTOR);
        gwRouter.createBottomUpCheckpoint(checkpoint1, membershipRoot, weights[0] + weights[1] + weights[2]);
        gwRouter.createBottomUpCheckpoint(checkpoint2, membershipRoot, weights[0] + weights[1] + weights[2]);
        vm.stopPrank();

        uint256[] memory heights = gwGetter.getIncompleteCheckpointHeights();

        require(heights.length == 2, "heights.length == 2");
        require(heights[0] == gwGetter.bottomUpCheckPeriod(), "heights[0] == period");
        require(heights[1] == 2 * gwGetter.bottomUpCheckPeriod(), "heights[1] == 2*period");

        CheckpointInfo memory info = gwGetter.getCheckpointInfo(gwGetter.bottomUpCheckPeriod());
        require(info.rootHash == membershipRoot, "info.rootHash == membershipRoot");
        require(
            info.threshold == gwGetter.getQuorumThreshold(weights[0] + weights[1] + weights[2]),
            "checkpoint 1 correct threshold"
        );

        info = gwGetter.getCheckpointInfo(2 * gwGetter.bottomUpCheckPeriod());
        require(info.rootHash == membershipRoot, "info.rootHash == membershipRoot");
        require(
            info.threshold == gwGetter.getQuorumThreshold(weights[0] + weights[1] + weights[2]),
            "checkpoint 2 correct threshold"
        );

        BottomUpCheckpoint[] memory incomplete = gwGetter.getIncompleteCheckpoints();
        require(incomplete.length == 2, "incomplete.length == 2");
        require(incomplete[0].blockHeight == gwGetter.bottomUpCheckPeriod(), "incomplete[0].blockHeight");
        require(incomplete[0].blockHash == keccak256("block1"), "incomplete[0].blockHash");
        require(incomplete[1].blockHeight == 2 * gwGetter.bottomUpCheckPeriod(), "incomplete[1].blockHeight");
        require(incomplete[1].blockHash == keccak256("block2"), "incomplete[1].blockHash");
    }

    function testGatewayDiamond_addCheckpointSignature_newCheckpoint() public {
        (
            uint256[] memory privKeys,
            address[] memory addrs,
            uint256[] memory weights
        ) = setupThreeValidatorsForCheckpoint();

        (bytes32 membershipRoot, bytes32[][] memory membershipProofs) = MerkleTreeHelper
            .createMerkleProofsForValidators(addrs, weights);

        BottomUpCheckpoint memory checkpoint = BottomUpCheckpoint({
            subnetID: gwGetter.getNetworkName(),
            blockHeight: gwGetter.bottomUpCheckPeriod(),
            blockHash: keccak256("block"),
            nextConfigurationNumber: 1,
            crossMessagesHash: keccak256("messages")
        });

        // create a checkpoint
        vm.startPrank(FilAddress.SYSTEM_ACTOR);
        gwRouter.createBottomUpCheckpoint(checkpoint, membershipRoot, weights[0] + weights[1] + weights[2]);
        vm.stopPrank();

        // adds signatures

        uint8 v;
        bytes32 r;
        bytes32 s;
        bytes memory signature;

        for (uint64 i = 0; i < 3; i++) {
            (v, r, s) = vm.sign(privKeys[i], keccak256(abi.encode(checkpoint)));
            signature = abi.encodePacked(r, s, v);

            vm.startPrank(vm.addr(privKeys[i]));
            gwRouter.addCheckpointSignature(checkpoint.blockHeight, membershipProofs[i], weights[i], signature);
            vm.stopPrank();
        }

        require(
            gwGetter.getCheckpointCurrentWeight(checkpoint.blockHeight) == totalWeight(weights),
            "checkpoint weight was updated"
        );
    }

    function testGatewayDiamond_addCheckpointSignature_quorum() public {
        (
            uint256[] memory privKeys,
            address[] memory addrs,
            uint256[] memory weights
        ) = setupThreeValidatorsForCheckpoint();

        (bytes32 membershipRoot, bytes32[][] memory membershipProofs) = MerkleTreeHelper
            .createMerkleProofsForValidators(addrs, weights);

        BottomUpCheckpoint memory checkpoint = BottomUpCheckpoint({
            subnetID: gwGetter.getNetworkName(),
            blockHeight: gwGetter.bottomUpCheckPeriod(),
            blockHash: keccak256("block"),
            nextConfigurationNumber: 1,
            crossMessagesHash: keccak256("messages")
        });

        // create a checkpoint
        vm.startPrank(FilAddress.SYSTEM_ACTOR);
        gwRouter.createBottomUpCheckpoint(checkpoint, membershipRoot, weights[0] + weights[1] + weights[2]);
        vm.stopPrank();

        // adds signatures

        uint8 v;
        bytes32 r;
        bytes32 s;
        bytes memory signature;

        for (uint64 i = 0; i < 2; i++) {
            (v, r, s) = vm.sign(privKeys[i], keccak256(abi.encode(checkpoint)));
            signature = abi.encodePacked(r, s, v);

            vm.startPrank(vm.addr(privKeys[i]));
            gwRouter.addCheckpointSignature(checkpoint.blockHeight, membershipProofs[i], weights[i], signature);
            vm.stopPrank();
        }

        CheckpointInfo memory info = gwGetter.getCheckpointInfo(1);
        require(!info.reached, "not reached");
        require(gwGetter.getIncompleteCheckpointHeights().length == 1, "size is 1");

        info = gwGetter.getCheckpointInfo(1);

        (v, r, s) = vm.sign(privKeys[2], keccak256(abi.encode(checkpoint)));
        signature = abi.encodePacked(r, s, v);

        vm.startPrank(vm.addr(privKeys[2]));
        gwRouter.addCheckpointSignature(checkpoint.blockHeight, membershipProofs[2], weights[2], signature);
        vm.stopPrank();

        info = gwGetter.getCheckpointInfo(checkpoint.blockHeight);
        require(info.reached, "reached");
        require(gwGetter.getIncompleteCheckpointHeights().length == 0, "size is 0");

        require(
            gwGetter.getCheckpointCurrentWeight(checkpoint.blockHeight) == totalWeight(weights),
            "checkpoint weight was updated"
        );
    }

    function testGatewayDiamond_addCheckpointSignature_notAuthorized() public {
        (
            uint256[] memory privKeys,
            address[] memory addrs,
            uint256[] memory weights
        ) = setupThreeValidatorsForCheckpoint();

        (bytes32 membershipRoot, bytes32[][] memory membershipProofs) = MerkleTreeHelper
            .createMerkleProofsForValidators(addrs, weights);

        BottomUpCheckpoint memory checkpoint = BottomUpCheckpoint({
            subnetID: gwGetter.getNetworkName(),
            blockHeight: gwGetter.bottomUpCheckPeriod(),
            blockHash: keccak256("block"),
            nextConfigurationNumber: 1,
            crossMessagesHash: keccak256("messages")
        });

        // create a checkpoint
        vm.startPrank(FilAddress.SYSTEM_ACTOR);
        gwRouter.createBottomUpCheckpoint(checkpoint, membershipRoot, 10);
        vm.stopPrank();

        uint8 v;
        bytes32 r;
        bytes32 s;
        bytes memory signature;

        (v, r, s) = vm.sign(privKeys[0], keccak256(abi.encode(checkpoint)));
        signature = abi.encodePacked(r, s, v);

        vm.startPrank(vm.addr(privKeys[0]));
        vm.expectRevert();
        gwRouter.addCheckpointSignature(1, membershipProofs[0], weights[1], signature);
        vm.stopPrank();
    }

    function testGatewayDiamond_garbage_collect_bottomUpCheckpoints() public {
        (, address[] memory addrs, uint256[] memory weights) = setupThreeValidatorsForCheckpoint();

        (bytes32 membershipRoot, ) = MerkleTreeHelper.createMerkleProofsForValidators(addrs, weights);

        uint64 index = gwGetter.getBottomUpRetentionHeight();
        require(index == 1, "retention height is 1");

        BottomUpCheckpoint memory checkpoint;

        // create a checkpoint
        uint64 n = 10;
        vm.startPrank(FilAddress.SYSTEM_ACTOR);
        for (uint64 i = 1; i <= n; i++) {
            checkpoint = BottomUpCheckpoint({
                subnetID: gwGetter.getNetworkName(),
                blockHeight: i * gwGetter.bottomUpCheckPeriod(),
                blockHash: keccak256("block"),
                nextConfigurationNumber: 1,
                crossMessagesHash: keccak256("messages")
            });

            gwRouter.createBottomUpCheckpoint(checkpoint, membershipRoot, 10);
        }
        vm.stopPrank();

        index = gwGetter.getBottomUpRetentionHeight();
        require(index == 1, "retention height is 1");

        uint256[] memory heights = gwGetter.getIncompleteCheckpointHeights();
        require(heights.length == n, "heights.len == n");

        vm.startPrank(FilAddress.SYSTEM_ACTOR);
        gwRouter.pruneBottomUpCheckpoints(4);
        vm.stopPrank();

        index = gwGetter.getBottomUpRetentionHeight();
        require(index == 4, "height updated");
        heights = gwGetter.getIncompleteCheckpointHeights();
        require(heights.length == n, "index the same");
    }

    function totalWeight(uint256[] memory weights) internal pure returns (uint256 sum) {
        for (uint64 i = 0; i < 3; i++) {
            sum += weights[i];
        }
        return sum;
    }

    function setupValidators() internal returns (FvmAddress[] memory validators, address[] memory addresses) {
        validators = new FvmAddress[](3);
        validators[0] = FvmAddressHelper.from(vm.addr(100));
        validators[1] = FvmAddressHelper.from(vm.addr(200));
        validators[2] = FvmAddressHelper.from(vm.addr(300));

        addresses = new address[](3);
        addresses[0] = vm.addr(100);
        addresses[1] = vm.addr(200);
        addresses[2] = vm.addr(300);

        uint256[] memory weights = new uint256[](3);

        vm.deal(vm.addr(100), 1);
        vm.deal(vm.addr(200), 1);
        vm.deal(vm.addr(300), 1);

        weights[0] = 100;
        weights[1] = 100;
        weights[2] = 100;

        // uint64 n = gwGetter.getLastConfigurationNumber() + 1;
        ParentFinality memory finality = ParentFinality({height: block.number, blockHash: bytes32(0)});

        vm.prank(FilAddress.SYSTEM_ACTOR);
        gwRouter.commitParentFinality(finality);
    }

    function setupThreeValidatorsForCheckpoint()
        internal
        returns (uint256[] memory validatorKeys, address[] memory addresses, uint256[] memory weights)
    {
        validatorKeys = new uint256[](3);
        validatorKeys[0] = 100;
        validatorKeys[1] = 200;
        validatorKeys[2] = 300;

        addresses = new address[](3);
        addresses[0] = vm.addr(validatorKeys[0]);
        addresses[1] = vm.addr(validatorKeys[1]);
        addresses[2] = vm.addr(validatorKeys[2]);

        weights = new uint256[](3);
        vm.deal(vm.addr(validatorKeys[0]), 1);
        vm.deal(vm.addr(validatorKeys[1]), 1);
        vm.deal(vm.addr(validatorKeys[2]), 1);

        weights = new uint256[](3);
        weights[0] = 100;
        weights[1] = 101;
        weights[2] = 102;
    }

    function addValidator(address validator) internal {
        addValidator(validator, 100);
    }

    function addValidator(address validator, uint256 weight) internal {
        FvmAddress[] memory validators = new FvmAddress[](1);
        validators[0] = FvmAddressHelper.from(validator);
        uint256[] memory weights = new uint256[](1);
        weights[0] = weight;

        vm.deal(validator, 1);
        ParentFinality memory finality = ParentFinality({height: block.number, blockHash: bytes32(0)});
        // uint64 n = gwGetter.getLastConfigurationNumber() + 1;

        vm.startPrank(FilAddress.SYSTEM_ACTOR);
        gwRouter.commitParentFinality(finality);
        vm.stopPrank();
    }

    function callback() public view {
        // console.log("callback called");
    }

    function fund(address funderAddress, uint256 fundAmount) internal {
        uint256 fundAmountWithSubtractedFee = fundAmount - gwGetter.crossMsgFee();

        (SubnetID memory subnetId, , uint256 nonceBefore, , uint256 circSupplyBefore, ) = getSubnet(address(saManager));

        uint256 expectedTopDownMsgsLength = 0;
        {
            expectedTopDownMsgsLength = gwGetter.getSubnetTopDownMsgsLength(subnetId) + 1;
            uint256 expectedNonce = nonceBefore + 1;
            uint256 expectedCircSupply = circSupplyBefore + fundAmountWithSubtractedFee;

            require(gwGetter.crossMsgFee() > 0, "crossMsgFee is 0");

            gwManager.fund{value: fundAmount}(subnetId, FvmAddressHelper.from(funderAddress));

            (, , uint256 nonce, , uint256 circSupply, ) = getSubnet(address(saManager));

            require(gwGetter.getSubnetTopDownMsgsLength(subnetId) == expectedTopDownMsgsLength);

            require(nonce == expectedNonce);
            require(circSupply == expectedCircSupply);
        }

        (CrossMsg[] memory topDownMsgs, ) = gwGetter.getTopDownMsgs(subnetId, block.number);
        for (uint256 msgIndex = 0; msgIndex < expectedTopDownMsgsLength; msgIndex++) {
            CrossMsg memory topDownMsg = topDownMsgs[msgIndex];

            require(topDownMsg.message.nonce == msgIndex);
            require(topDownMsg.message.value == fundAmountWithSubtractedFee);
            require(
                keccak256(abi.encode(topDownMsg.message.to)) ==
                    keccak256(
                        abi.encode(IPCAddress({subnetId: subnetId, rawAddress: FvmAddressHelper.from(funderAddress)}))
                    )
            );
            require(
                keccak256(abi.encode(topDownMsg.message.from)) ==
                    keccak256(
                        abi.encode(
                            IPCAddress({
                                subnetId: subnetId.getParentSubnet(),
                                rawAddress: FvmAddressHelper.from(funderAddress)
                            })
                        )
                    )
            );
        }
    }

    // function _join(address validatorAddress) internal {
    //     vm.prank(validatorAddress);
    //     vm.deal(validatorAddress, DEFAULT_COLLATERAL_AMOUNT + 1);

    //     saManager.join{value: DEFAULT_COLLATERAL_AMOUNT}(new bytes(20));

    //     (uint64 nextConfigNum, ) = saGetter.getConfigurationNumbers();
    //     saManager.confirmChange(nextConfigNum - 1);
    // }

    function release(uint256 releaseAmount, uint256 fee) internal {
        uint256 expectedNonce = gwGetter.bottomUpNonce() + 1;
        gwManager.release{value: releaseAmount}(FvmAddressHelper.from(msg.sender), fee);
        require(gwGetter.bottomUpNonce() == expectedNonce, "gwGetter.bottomUpNonce() == expectedNonce");
    }

    function addStake(uint256 stakeAmount, address subnetAddress) internal {
        uint256 balanceBefore = subnetAddress.balance;
        (, uint256 stakedBefore, , , , ) = getSubnet(subnetAddress);

        gwManager.addStake{value: stakeAmount}();

        uint256 balanceAfter = subnetAddress.balance;
        (, uint256 stakedAfter, , , , ) = getSubnet(subnetAddress);

        require(balanceAfter == balanceBefore - stakeAmount);
        require(stakedAfter == stakedBefore + stakeAmount);
    }

    function registerSubnetGW(uint256 collateral, address subnetAddress, GatewayDiamond gw) internal {
        gwRouter = GatewayRouterFacet(address(gw));
        gwManager = GatewayManagerFacet(address(gw));
        gwGetter = GatewayGetterFacet(address(gw));

        gwManager.register{value: collateral}();

        (SubnetID memory id, uint256 stake, uint256 topDownNonce, , uint256 circSupply, Status status) = getSubnetGW(
            subnetAddress,
            gw
        );

        SubnetID memory parentNetwork = gwGetter.getNetworkName();

        require(
            id.toHash() == parentNetwork.createSubnetId(subnetAddress).toHash(),
            "id.toHash() == parentNetwork.createSubnetId(subnetAddress).toHash()"
        );
        require(stake == collateral, "stake == collateral");
        require(topDownNonce == 0, "nonce == 0");
        require(circSupply == 0, "circSupply == 0");
        require(status == Status.Active, "status == Status.Active");
    }

    function registerSubnet(uint256 collateral, address subnetAddress) internal {
        registerSubnetGW(collateral, subnetAddress, gatewayDiamond);
    }

    function getSubnetGW(
        address subnetAddress,
        GatewayDiamond gw
    ) internal returns (SubnetID memory, uint256, uint256, uint256, uint256, Status) {
        gwRouter = GatewayRouterFacet(address(gw));
        gwManager = GatewayManagerFacet(address(gw));
        gwGetter = GatewayGetterFacet(address(gw));

        SubnetID memory subnetId = gwGetter.getNetworkName().createSubnetId(subnetAddress);

        Subnet memory subnet = gwGetter.subnets(subnetId.toHash());

        return (
            subnet.id,
            subnet.stake,
            subnet.topDownNonce,
            subnet.appliedBottomUpNonce,
            subnet.circSupply,
            subnet.status
        );
    }

    function getSubnet(
        address subnetAddress
    ) internal returns (SubnetID memory, uint256, uint256, uint256, uint256, Status) {
        return getSubnetGW(subnetAddress, gatewayDiamond);
    }
}
