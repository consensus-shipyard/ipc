// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity 0.8.19;

import "forge-std/Test.sol";

import "../../src/errors/IPCErrors.sol";
import {NumberContractFacetSeven, NumberContractFacetEight} from "../helpers/NumberContract.sol";
import {EMPTY_BYTES, METHOD_SEND} from "../../src/constants/Constants.sol";
import {Status} from "../../src/enums/Status.sol";
import {IERC165} from "../../src/interfaces/IERC165.sol";
import {IDiamond} from "../../src/interfaces/IDiamond.sol";
import {IDiamondLoupe} from "../../src/interfaces/IDiamondLoupe.sol";
import {IDiamondCut} from "../../src/interfaces/IDiamondCut.sol";
import {QuorumInfo} from "../../src/structs/Quorum.sol";
import {CrossMsg, BottomUpMsgBatch, BottomUpCheckpoint, StorableMsg, ParentFinality} from "../../src/structs/CrossNet.sol";
import {FvmAddress} from "../../src/structs/FvmAddress.sol";
import {SubnetID, Subnet, IPCAddress, Validator, StakingChange, StakingChangeRequest, StakingOperation} from "../../src/structs/Subnet.sol";
import {SubnetIDHelper} from "../../src/lib/SubnetIDHelper.sol";
import {FvmAddressHelper} from "../../src/lib/FvmAddressHelper.sol";
import {CrossMsgHelper} from "../../src/lib/CrossMsgHelper.sol";
import {StorableMsgHelper} from "../../src/lib/StorableMsgHelper.sol";
import {FilAddress} from "fevmate/utils/FilAddress.sol";
import {GatewayDiamond, FunctionNotFound} from "../../src/GatewayDiamond.sol";
import {GatewayGetterFacet} from "../../src/gateway/GatewayGetterFacet.sol";
import {GatewayManagerFacet} from "../../src/gateway/GatewayManagerFacet.sol";
import {GatewayRouterFacet} from "../../src/gateway/GatewayRouterFacet.sol";
import {ERR_GENERAL_CROSS_MSG_DISABLED} from "../../src/gateway/GatewayMessengerFacet.sol";
import {DiamondCutFacet} from "../../src/diamond/DiamondCutFacet.sol";
import {LibDiamond} from "../../src/lib/LibDiamond.sol";
import {MerkleTreeHelper} from "../helpers/MerkleTreeHelper.sol";

import {TestUtils} from "../helpers/TestUtils.sol";
import {IntegrationTestBase} from "../IntegrationTestBase.sol";

contract GatewayActorDiamondTest is Test, IntegrationTestBase {
    using SubnetIDHelper for SubnetID;
    using CrossMsgHelper for CrossMsg;
    using StorableMsgHelper for StorableMsg;
    using FvmAddressHelper for FvmAddress;

    function setUp() public override {
        super.setUp();
    }

    function testGatewayDiamond_Constructor() public view {
        require(gwGetter.totalSubnets() == 0, "unexpected totalSubnets");
        require(gwGetter.bottomUpNonce() == 0, "unexpected bottomUpNonce");
        require(gwGetter.minStake() == DEFAULT_COLLATERAL_AMOUNT, "unexpected minStake");

        require(gwGetter.crossMsgFee() == DEFAULT_CROSS_MSG_FEE, "unexpected crossMsgFee");
        require(gwGetter.bottomUpCheckPeriod() == DEFAULT_CHECKPOINT_PERIOD, "unexpected bottomUpCheckPeriod");
        require(
            gwGetter.getNetworkName().equals(SubnetID({root: ROOTNET_CHAINID, route: new address[](0)})),
            "unexpected getNetworkName"
        );
        require(gwGetter.majorityPercentage() == DEFAULT_MAJORITY_PERCENTAGE, "unexpected majorityPercentage");

        (StorableMsg memory storableMsg, bool wrapped) = gwGetter.postbox(0);
        StorableMsg memory msg1;
        require(msg1.toHash() == storableMsg.toHash(), "unexpected hash");
        require(!wrapped, "unexpected wrapped message");
    }

    function testGatewayDiamond_NewGatewayWithDefaultParams() public view {
        GatewayDiamond.ConstructorParams memory params = defaultGatewayParams();

        require(gwGetter.crossMsgFee() == DEFAULT_CROSS_MSG_FEE, "unexpected cross-msg fee");
        require(gwGetter.bottomUpCheckPeriod() == params.bottomUpCheckPeriod, "unexpected bottom-up period");
        require(gwGetter.majorityPercentage() == params.majorityPercentage, "unexpected majority percentage");
    }

    function testGatewayDiamond_LoupeFunction() public view {
        require(gwLoupeFacet.facets().length == 6, "unexpected length");
        require(gwLoupeFacet.supportsInterface(type(IERC165).interfaceId) == true, "IERC165 not supported");
        require(gwLoupeFacet.supportsInterface(type(IDiamondCut).interfaceId) == true, "IDiamondCut not supported");
        require(gwLoupeFacet.supportsInterface(type(IDiamondLoupe).interfaceId) == true, "IDiamondLoupe not supported");
    }

    function testGatewayDiamond_DiamondCut() public {
        // add method getNum to gateway diamond and assert it can be correctly called
        // replace method getNum and assert it was correctly updated
        // delete method getNum and assert it no longer is callable
        // assert that diamondCut cannot be called by non-owner

        NumberContractFacetSeven ncFacetA = new NumberContractFacetSeven();
        NumberContractFacetEight ncFacetB = new NumberContractFacetEight();

        DiamondCutFacet gwDiamondCutter = DiamondCutFacet(address(gatewayDiamond));
        IDiamond.FacetCut[] memory gwDiamondCut = new IDiamond.FacetCut[](1);
        bytes4[] memory ncGetterSelectors = TestUtils.generateSelectors(vm, "NumberContractFacetSeven");

        gwDiamondCut[0] = (
            IDiamond.FacetCut({
                facetAddress: address(ncFacetA),
                action: IDiamond.FacetCutAction.Add,
                functionSelectors: ncGetterSelectors
            })
        );
        //test that other user cannot call diamondcut to add function
        vm.prank(0x1234567890123456789012345678901234567890);
        vm.expectRevert(LibDiamond.NotOwner.selector);
        gwDiamondCutter.diamondCut(gwDiamondCut, address(0), new bytes(0));

        gwDiamondCutter.diamondCut(gwDiamondCut, address(0), new bytes(0));

        NumberContractFacetSeven gwNumberContract = NumberContractFacetSeven(address(gatewayDiamond));
        assert(gwNumberContract.getNum() == 7);

        ncGetterSelectors = TestUtils.generateSelectors(vm, "NumberContractFacetEight");
        gwDiamondCut[0] = (
            IDiamond.FacetCut({
                facetAddress: address(ncFacetB),
                action: IDiamond.FacetCutAction.Replace,
                functionSelectors: ncGetterSelectors
            })
        );

        //test that other user cannot call diamondcut to replace function
        vm.prank(0x1234567890123456789012345678901234567890);
        vm.expectRevert(LibDiamond.NotOwner.selector);
        gwDiamondCutter.diamondCut(gwDiamondCut, address(0), new bytes(0));

        gwDiamondCutter.diamondCut(gwDiamondCut, address(0), new bytes(0));

        assert(gwNumberContract.getNum() == 8);

        //remove facet for getNum
        gwDiamondCut[0] = (
            IDiamond.FacetCut({
                facetAddress: 0x0000000000000000000000000000000000000000,
                action: IDiamond.FacetCutAction.Remove,
                functionSelectors: ncGetterSelectors
            })
        );

        //test that other user cannot call diamondcut to remove function
        vm.prank(0x1234567890123456789012345678901234567890);
        vm.expectRevert(LibDiamond.NotOwner.selector);
        gwDiamondCutter.diamondCut(gwDiamondCut, address(0), new bytes(0));

        gwDiamondCutter.diamondCut(gwDiamondCut, address(0), new bytes(0));

        //assert that calling getNum fails
        vm.expectRevert(abi.encodePacked(FunctionNotFound.selector, ncGetterSelectors));
        gwNumberContract.getNum();
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
            msgFee: DEFAULT_CROSS_MSG_FEE,
            minCollateral: DEFAULT_COLLATERAL_AMOUNT,
            majorityPercentage: DEFAULT_MAJORITY_PERCENTAGE,
            genesisValidators: new Validator[](0),
            activeValidatorsLimit: 100
        });

        dep = createGatewayDiamond(constructorParams);
        depGetter = GatewayGetterFacet(address(dep));
        depManager = GatewayManagerFacet(address(dep));
        depRouter = GatewayRouterFacet(address(dep));

        SubnetID memory networkName = depGetter.getNetworkName();

        require(networkName.isRoot(), "unexpected networkName");
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
            msgFee: DEFAULT_CROSS_MSG_FEE,
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

        require(networkName.isRoot() == false, "unexpected networkName");
        require(depGetter.minStake() == DEFAULT_COLLATERAL_AMOUNT, "unexpected minStake");
        require(depGetter.bottomUpCheckPeriod() == checkpointPeriod, "unexpected bottomUpCheckPeriod");
        require(depGetter.majorityPercentage() == 100, "unexpected majorityPercentage");
    }

    function testGatewayDiamond_Register_Works_SingleSubnet(uint256 subnetCollateral) public {
        vm.assume(subnetCollateral >= DEFAULT_COLLATERAL_AMOUNT && subnetCollateral < type(uint64).max);
        address subnetAddress = vm.addr(100);
        vm.prank(subnetAddress);
        vm.deal(subnetAddress, subnetCollateral);

        registerSubnet(subnetCollateral, subnetAddress);
        require(gwGetter.totalSubnets() == 1, "unexpected totalSubnets");
        Subnet[] memory subnets = gwGetter.listSubnets();
        require(subnets.length == 1, "unexpected subnets length");

        SubnetID memory subnetId = gwGetter.getNetworkName().createSubnetId(subnetAddress);

        (bool ok, Subnet memory targetSubnet) = gwGetter.getSubnet(subnetId);

        require(ok, "subnet not found");

        (SubnetID memory id, uint256 stake, , , , Status status) = getSubnet(subnetAddress);

        require(targetSubnet.status == Status.Active, "unexpected status");
        require(targetSubnet.status == status, "unexpected status value");
        require(targetSubnet.stake == stake, "unexpected stake");
        require(targetSubnet.stake == subnetCollateral, "unexpected collateral");
        require(id.equals(subnetId), "unexpected id");
    }

    function testGatewayDiamond_Register_Works_MultipleSubnets(uint8 numberOfSubnets) public {
        vm.assume(numberOfSubnets > 0);

        for (uint256 i = 1; i <= numberOfSubnets; i++) {
            address subnetAddress = vm.addr(i);
            vm.prank(subnetAddress);
            vm.deal(subnetAddress, DEFAULT_COLLATERAL_AMOUNT);

            registerSubnet(DEFAULT_COLLATERAL_AMOUNT, subnetAddress);
        }

        require(gwGetter.totalSubnets() == numberOfSubnets, "unexpected total subnets");
        Subnet[] memory subnets = gwGetter.listSubnets();
        require(subnets.length == numberOfSubnets, "unexpected length");
    }

    function testGatewayDiamond_Register_Fail_SubnetAlreadyExists() public {
        registerSubnet(DEFAULT_COLLATERAL_AMOUNT, address(this));

        vm.expectRevert(AlreadyRegisteredSubnet.selector);

        gwManager.register{value: DEFAULT_COLLATERAL_AMOUNT}(0);
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

        require(totalStaked == totalAmount, "unexpected staked amount");
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
        require(statusInactive == Status.Inactive, "unexpected status");

        vm.deal(subnetAddress, stakeAmount);
        addStake(stakeAmount, subnetAddress);

        (, uint256 staked, , , , Status statusActive) = getSubnet(subnetAddress);

        require(staked == stakeAmount, "unexpected amount");
        require(statusActive == Status.Active, "not active status");
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

        require(staked == stakeAmount, "unexpected amount");
        require(status == Status.Inactive, "unexpected inactive status");
    }

    function testGatewayDiamond_AddStake_Works_MultipleStakings(uint8 numberOfStakes) public {
        vm.assume(numberOfStakes > 0);

        address subnetAddress = vm.addr(100);
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

        require(totalStake == expectedStakedAmount, "unexpected stake");
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

        require(stake == 0, "unexpected stake");
        require(status == Status.Inactive, "unexpected status");
        require(subnetAddress.balance == fullAmount, "unexpected balance");
    }

    function testGatewayDiamond_ReleaseStake_Works_SubnetInactive() public {
        address subnetAddress = vm.addr(100);
        vm.startPrank(subnetAddress);
        vm.deal(subnetAddress, DEFAULT_COLLATERAL_AMOUNT);
        registerSubnet(DEFAULT_COLLATERAL_AMOUNT, subnetAddress);

        gwManager.releaseStake(DEFAULT_COLLATERAL_AMOUNT / 2);

        (, uint256 stake, , , , Status status) = getSubnet(subnetAddress);
        require(stake == DEFAULT_COLLATERAL_AMOUNT / 2, "unexpected stake");
        require(status == Status.Inactive, "unexpected status");
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

        require(stake == registerAmount, "unexpected stake");
        require(status == Status.Active, "unexpected status");
        require(subnetAddress.balance == partialAmount, "unexpected balance");
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

        require(stake == DEFAULT_COLLATERAL_AMOUNT - 10, "unexpected stake");
        require(status == Status.Inactive, "unexpected status");
    }

    function testGatewayDiamond_Kill_Works() public {
        address subnetAddress = CHILD_NETWORK_ADDRESS;

        vm.startPrank(subnetAddress);
        vm.deal(subnetAddress, DEFAULT_COLLATERAL_AMOUNT);

        registerSubnet(DEFAULT_COLLATERAL_AMOUNT, subnetAddress);

        require(subnetAddress.balance == 0, "unexpected balance");

        gwManager.kill();

        (SubnetID memory id, uint256 stake, uint256 nonce, , uint256 circSupply, Status status) = getSubnet(
            subnetAddress
        );

        require(id.toHash() == SubnetID(0, new address[](0)).toHash(), "unexpected ID hash");
        require(stake == 0, "unexpected stake");
        require(nonce == 0, "unexpected nonce");
        require(circSupply == 0, "unexpected circSupply");
        require(status == Status.Unset, "unexpected status");
        require(gwGetter.totalSubnets() == 0, "unexpected total subnets");
        require(subnetAddress.balance == DEFAULT_COLLATERAL_AMOUNT, "unexpected balance");
    }

    function testGatewayDiamond_Kill_Fail_SubnetNotExists() public {
        vm.expectRevert(NotRegisteredSubnet.selector);

        gwManager.kill();
    }

    function testGatewayDiamond_SendCrossMessage_Fails_NoFunds() public {
        address caller = vm.addr(100);
        vm.startPrank(caller);
        vm.deal(caller, DEFAULT_COLLATERAL_AMOUNT + DEFAULT_CROSS_MSG_FEE + 2);
        registerSubnet(DEFAULT_COLLATERAL_AMOUNT, caller);

        // vm.expectRevert(NotEnoughFunds.selector);

        // General-purpose cross-net messages are currenlty disabled.
        vm.expectRevert(abi.encodeWithSelector(MethodNotAllowed.selector, ERR_GENERAL_CROSS_MSG_DISABLED));
        gwMessenger.sendUserXnetMessage{value: DEFAULT_CROSS_MSG_FEE - 1}(
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
                    fee: DEFAULT_CROSS_MSG_FEE
                }),
                wrapped: false
            })
        );

        // vm.expectRevert(NotEnoughFee.selector);
        // General-purpose cross-net messages are currenlty disabled.
        vm.expectRevert(abi.encodeWithSelector(MethodNotAllowed.selector, ERR_GENERAL_CROSS_MSG_DISABLED));
        gwMessenger.sendUserXnetMessage{value: DEFAULT_CROSS_MSG_FEE + 1}(
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
                    fee: DEFAULT_CROSS_MSG_FEE - 1
                }),
                wrapped: false
            })
        );
    }

    function testGatewayDiamond_SendCrossMessage_Fails_Fuzz(uint256 fee) public {
        vm.assume(fee < DEFAULT_CROSS_MSG_FEE);

        address caller = vm.addr(100);
        vm.deal(caller, DEFAULT_COLLATERAL_AMOUNT + DEFAULT_CROSS_MSG_FEE + 2);
        vm.prank(caller);
        registerSubnet(DEFAULT_COLLATERAL_AMOUNT, caller);

        vm.expectRevert();
        gwMessenger.sendUserXnetMessage{value: fee - 1}(
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
                    fee: DEFAULT_CROSS_MSG_FEE
                }),
                wrapped: false
            })
        );
    }

    function testGatewayDiamond_Single_Funding() public {
        (address validatorAddress, , bytes memory publicKey) = TestUtils.newValidator(100);

        join(validatorAddress, publicKey);

        address funderAddress = address(101);
        uint256 fundAmount = 1 ether;

        vm.deal(funderAddress, fundAmount + 1);

        vm.prank(funderAddress);
        fund(funderAddress, fundAmount);
    }

    function testGatewayDiamond_Fund_Kill_Fail_CircSupplyMoreThanZero() public {
        (address validatorAddress, bytes memory publicKey) = TestUtils.deriveValidatorAddress(100);

        join(validatorAddress, publicKey);

        address funderAddress = address(101);
        uint256 fundAmount = 1 ether;

        vm.deal(funderAddress, fundAmount + 1);

        vm.startPrank(funderAddress);
        fund(funderAddress, fundAmount);
        vm.stopPrank();

        vm.startPrank(address(saManager));
        vm.expectRevert(NotEmptySubnetCircSupply.selector);
        gwManager.kill();
    }

    function testGatewayDiamond_Fund_Revert_OnZeroValue() public {
        (address validatorAddress, bytes memory publicKey) = TestUtils.deriveValidatorAddress(100);
        join(validatorAddress, publicKey);

        address funderAddress = address(101);

        (SubnetID memory subnetId, , , , , ) = getSubnet(address(saManager));

        vm.expectRevert(InvalidCrossMsgValue.selector);
        gwManager.fund{value: 0}(subnetId, FvmAddressHelper.from(funderAddress));
    }

    function testGatewayDiamond_Fund_Works_MultipleFundings(uint8 numberOfFunds) public {
        vm.assume(numberOfFunds > 10);
        vm.assume(numberOfFunds < 50);

        uint256 fundAmount = 1 ether;

        address funderAddress = address(101);

        (address validatorAddress, bytes memory publicKey) = TestUtils.deriveValidatorAddress(100);
        join(validatorAddress, publicKey);

        vm.startPrank(funderAddress);
        for (uint256 i = 0; i < numberOfFunds; i++) {
            vm.deal(funderAddress, fundAmount + 1);
            fund(funderAddress, fundAmount);
        }
    }

    function testGatewayDiamond_Fund_Fuzz_InsufficientAmount(uint256 amount) public {
        vm.assume(amount > 0);
        vm.assume(amount < DEFAULT_COLLATERAL_AMOUNT);

        address funderAddress = address(101);

        (address validatorAddress, bytes memory publicKey) = TestUtils.deriveValidatorAddress(100);
        join(validatorAddress, publicKey);

        vm.deal(funderAddress, amount);

        (SubnetID memory subnetId, , , , , ) = getSubnet(address(saManager));
        vm.prank(funderAddress);
        gwManager.fund{value: amount}(subnetId, FvmAddressHelper.from(msg.sender));
    }

    function testGatewayDiamond_Fund_Fails_NotRegistered() public {
        address funderAddress = address(101);
        uint256 fundAmount = 1 ether;

        (address validatorAddress, bytes memory publicKey) = TestUtils.deriveValidatorAddress(100);
        join(validatorAddress, publicKey);

        address[] memory wrongSubnetPath = new address[](2);
        wrongSubnetPath[0] = vm.addr(102);
        wrongSubnetPath[0] = vm.addr(103);

        address[] memory wrongPath = new address[](3);
        wrongPath[0] = address(1);
        wrongPath[1] = address(2);

        vm.deal(funderAddress, fundAmount + 1);

        vm.startPrank(funderAddress);

        SubnetID memory wrongSubnetId = SubnetID({root: ROOTNET_CHAINID, route: wrongSubnetPath});

        vm.expectRevert(NotRegisteredSubnet.selector);
        gwManager.fund{value: fundAmount}(wrongSubnetId, FvmAddressHelper.from(msg.sender));

        vm.expectRevert(NotRegisteredSubnet.selector);
        gwManager.fund{value: fundAmount}(SubnetID(ROOTNET_CHAINID, wrongPath), FvmAddressHelper.from(msg.sender));
    }

    function testGatewayDiamond_Fund_Works_BLSAccountSingleFunding() public {
        (address validatorAddress, bytes memory publicKey) = TestUtils.deriveValidatorAddress(100);
        join(validatorAddress, publicKey);

        uint256 fundAmount = 1 ether;
        vm.deal(BLS_ACCOUNT_ADDREESS, fundAmount + 1);
        vm.startPrank(BLS_ACCOUNT_ADDREESS);

        fund(BLS_ACCOUNT_ADDREESS, fundAmount);
    }

    function testGatewayDiamond_Fund_Works_ReactivatedSubnet() public {
        (address validatorAddress, uint256 privKey, bytes memory publicKey) = TestUtils.newValidator(100);
        assert(validatorAddress == vm.addr(privKey));

        join(validatorAddress, publicKey);

        vm.prank(validatorAddress);
        saManager.leave();

        join(validatorAddress, publicKey);

        address funderAddress = address(101);
        uint256 fundAmount = 1 ether;

        vm.deal(funderAddress, fundAmount + 1);
        fund(funderAddress, fundAmount);
    }

    function testGatewayDiamond_Release_Fails_InsufficientAmount() public {
        address[] memory path = new address[](2);
        path[0] = address(1);
        path[1] = address(2);

        GatewayDiamond.ConstructorParams memory constructorParams = GatewayDiamond.ConstructorParams({
            networkName: SubnetID({root: ROOTNET_CHAINID, route: path}),
            bottomUpCheckPeriod: DEFAULT_CHECKPOINT_PERIOD,
            msgFee: DEFAULT_CROSS_MSG_FEE,
            minCollateral: DEFAULT_COLLATERAL_AMOUNT,
            majorityPercentage: DEFAULT_MAJORITY_PERCENTAGE,
            genesisValidators: new Validator[](0),
            activeValidatorsLimit: 100
        });
        gatewayDiamond = createGatewayDiamond(constructorParams);
        gwGetter = GatewayGetterFacet(address(gatewayDiamond));
        gwManager = GatewayManagerFacet(address(gatewayDiamond));
        gwRouter = GatewayRouterFacet(address(gatewayDiamond));

        address callerAddress = address(100);

        vm.startPrank(callerAddress);
        vm.deal(callerAddress, 1 ether);
        vm.expectRevert(InvalidCrossMsgValue.selector);

        gwManager.release{value: 0 ether}(FvmAddressHelper.from(msg.sender));
    }

    function testGatewayDiamond_Release_Works_BLSAccount(uint256 releaseAmount, uint256 crossMsgFee) public {
        vm.assume(crossMsgFee >= DEFAULT_CROSS_MSG_FEE);
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
        gatewayDiamond = createGatewayDiamond(constructorParams);
        gwGetter = GatewayGetterFacet(address(gatewayDiamond));
        gwManager = GatewayManagerFacet(address(gatewayDiamond));
        gwRouter = GatewayRouterFacet(address(gatewayDiamond));

        vm.roll(0);
        vm.warp(0);
        vm.startPrank(BLS_ACCOUNT_ADDREESS);
        vm.deal(BLS_ACCOUNT_ADDREESS, releaseAmount + 1);
        release(releaseAmount);
        require(gwGetter.bottomUpMsgBatch(gwGetter.bottomUpMsgBatchPeriod()).msgs.length == 1, "no messages");
    }

    function testGatewayDiamond_Release_Works_EmptyCrossMsgMeta(uint256 releaseAmount, uint256 crossMsgFee) public {
        vm.assume(crossMsgFee >= DEFAULT_CROSS_MSG_FEE);
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
        gatewayDiamond = createGatewayDiamond(constructorParams);
        gwGetter = GatewayGetterFacet(address(gatewayDiamond));
        gwManager = GatewayManagerFacet(address(gatewayDiamond));
        gwRouter = GatewayRouterFacet(address(gatewayDiamond));

        address callerAddress = address(100);

        vm.roll(0);
        vm.warp(0);
        vm.startPrank(callerAddress);
        vm.deal(callerAddress, releaseAmount + 1);
        release(releaseAmount);
    }

    function testGatewayDiamond_Release_Works_NonEmptyCrossMsgMeta(uint256 releaseAmount, uint256 crossMsgFee) public {
        vm.assume(crossMsgFee >= DEFAULT_CROSS_MSG_FEE);
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
        gatewayDiamond = createGatewayDiamond(constructorParams);
        gwGetter = GatewayGetterFacet(address(gatewayDiamond));
        gwManager = GatewayManagerFacet(address(gatewayDiamond));
        gwRouter = GatewayRouterFacet(address(gatewayDiamond));

        address callerAddress = address(100);

        vm.roll(0);
        vm.warp(0);
        vm.startPrank(callerAddress);
        vm.deal(callerAddress, 2 * releaseAmount + 1);

        release(releaseAmount);
        release(releaseAmount);
    }

    function testGatewayDiamond_SendCrossMessage_Fails_NoDestination() public {
        address caller = vm.addr(100);
        vm.startPrank(caller);
        vm.deal(caller, DEFAULT_COLLATERAL_AMOUNT + DEFAULT_CROSS_MSG_FEE + 2);
        registerSubnet(DEFAULT_COLLATERAL_AMOUNT, caller);

        // vm.expectRevert(InvalidCrossMsgDstSubnet.selector);
        // General-purpose cross-net messages are currenlty disabled.
        vm.expectRevert(abi.encodeWithSelector(MethodNotAllowed.selector, ERR_GENERAL_CROSS_MSG_DISABLED));
        gwMessenger.sendUserXnetMessage{value: DEFAULT_CROSS_MSG_FEE + 1}(
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
                    fee: DEFAULT_CROSS_MSG_FEE
                }),
                wrapped: false
            })
        );
    }

    function testGatewayDiamond_SendCrossMessage_Fails_NoCurrentNetwork() public {
        address caller = vm.addr(100);
        vm.startPrank(caller);
        vm.deal(caller, DEFAULT_COLLATERAL_AMOUNT + DEFAULT_CROSS_MSG_FEE + 2);
        registerSubnet(DEFAULT_COLLATERAL_AMOUNT, caller);
        SubnetID memory destinationSubnet = gwGetter.getNetworkName();

        // vm.expectRevert(CannotSendCrossMsgToItself.selector);
        // General-purpose cross-net messages are currenlty disabled.
        vm.expectRevert(abi.encodeWithSelector(MethodNotAllowed.selector, ERR_GENERAL_CROSS_MSG_DISABLED));
        gwMessenger.sendUserXnetMessage{value: DEFAULT_CROSS_MSG_FEE + 1}(
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
                    fee: DEFAULT_CROSS_MSG_FEE
                }),
                wrapped: true
            })
        );
    }

    function testGatewayDiamond_SendCrossMessage_Fails_Failes_InvalidCrossMsgValue() public {
        address caller = vm.addr(100);
        vm.startPrank(caller);
        vm.deal(caller, DEFAULT_COLLATERAL_AMOUNT + DEFAULT_CROSS_MSG_FEE + 2);
        registerSubnet(DEFAULT_COLLATERAL_AMOUNT, caller);
        SubnetID memory destinationSubnet = gwGetter.getNetworkName().createSubnetId(caller);

        // vm.expectRevert(InvalidCrossMsgValue.selector);
        // General-purpose cross-net messages are currenlty disabled.
        vm.expectRevert(abi.encodeWithSelector(MethodNotAllowed.selector, ERR_GENERAL_CROSS_MSG_DISABLED));
        gwMessenger.sendUserXnetMessage{value: DEFAULT_CROSS_MSG_FEE + 1}(
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
                    fee: DEFAULT_CROSS_MSG_FEE
                }),
                wrapped: true
            })
        );
    }

    function testGatewayDiamond_SendCrossMessage_Fails_EmptyNetwork() public {
        address caller = vm.addr(100);
        vm.startPrank(caller);
        vm.deal(caller, DEFAULT_COLLATERAL_AMOUNT + DEFAULT_CROSS_MSG_FEE + 2);

        registerSubnet(DEFAULT_COLLATERAL_AMOUNT, caller);

        SubnetID memory destinationSubnet = SubnetID(0, new address[](0));
        // vm.expectRevert(InvalidCrossMsgDstSubnet.selector);

        // General-purpose cross-net messages are currenlty disabled.
        vm.expectRevert(abi.encodeWithSelector(MethodNotAllowed.selector, ERR_GENERAL_CROSS_MSG_DISABLED));
        gwMessenger.sendUserXnetMessage{value: DEFAULT_CROSS_MSG_FEE + 1}(
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
                    fee: DEFAULT_CROSS_MSG_FEE
                }),
                wrapped: true
            })
        );
    }

    function testGatewayDiamond_SendCrossMessage_Fails_InvalidCrossMsgFromSubnet() public {
        address caller = vm.addr(100);
        vm.startPrank(caller);
        vm.deal(caller, DEFAULT_COLLATERAL_AMOUNT + DEFAULT_CROSS_MSG_FEE + 2);
        registerSubnet(DEFAULT_COLLATERAL_AMOUNT, caller);
        SubnetID memory destinationSubnet = gwGetter.getNetworkName().createSubnetId(caller);

        // vm.expectRevert(InvalidCrossMsgFromSubnet.selector);
        // General-purpose cross-net messages are currenlty disabled.
        vm.expectRevert(abi.encodeWithSelector(MethodNotAllowed.selector, ERR_GENERAL_CROSS_MSG_DISABLED));
        gwMessenger.sendUserXnetMessage{value: DEFAULT_CROSS_MSG_FEE + 1}(
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
                    fee: DEFAULT_CROSS_MSG_FEE
                }),
                wrapped: true
            })
        );
    }

    function testGatewayDiamond_SendCrossMessage_Fails_NotEnoughFee() public {
        address caller = vm.addr(100);
        vm.startPrank(caller);
        vm.deal(caller, DEFAULT_COLLATERAL_AMOUNT + DEFAULT_CROSS_MSG_FEE);
        registerSubnet(DEFAULT_COLLATERAL_AMOUNT, caller);
        SubnetID memory destinationSubnet = gwGetter.getNetworkName().createSubnetId(caller);

        // vm.expectRevert(NotEnoughFee.selector);
        // General-purpose cross-net messages are currenlty disabled.
        vm.expectRevert(abi.encodeWithSelector(MethodNotAllowed.selector, ERR_GENERAL_CROSS_MSG_DISABLED));
        gwMessenger.sendUserXnetMessage{value: DEFAULT_CROSS_MSG_FEE}(
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
        vm.deal(caller, DEFAULT_COLLATERAL_AMOUNT + DEFAULT_CROSS_MSG_FEE);
        registerSubnet(DEFAULT_COLLATERAL_AMOUNT, caller);
        SubnetID memory destinationSubnet = gwGetter.getNetworkName().createSubnetId(caller);

        // vm.expectRevert(NotEnoughFunds.selector);
        // General-purpose cross-net messages are currenlty disabled.
        vm.expectRevert(abi.encodeWithSelector(MethodNotAllowed.selector, ERR_GENERAL_CROSS_MSG_DISABLED));
        gwMessenger.sendUserXnetMessage{value: 0}(
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
                    fee: DEFAULT_CROSS_MSG_FEE
                }),
                wrapped: true
            })
        );
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
            configurationNumber: 1,
            change: StakingChange({validator: val1, op: StakingOperation.Deposit, payload: abi.encode(amount)})
        });
        changes[1] = StakingChangeRequest({
            configurationNumber: 2,
            change: StakingChange({validator: val2, op: StakingOperation.Deposit, payload: abi.encode(amount)})
        });

        vm.startPrank(FilAddress.SYSTEM_ACTOR);

        gwRouter.storeValidatorChanges(changes);
        uint64 configNumber = gwRouter.applyFinalityChanges();
        require(configNumber == 2, "wrong config number after applying finality");
        require(gwGetter.getCurrentMembership().validators.length == 2, "current membership should be 2");
        require(gwGetter.getCurrentConfigurationNumber() == 2, "unexpected config number");
        require(gwGetter.getLastConfigurationNumber() == 0, "unexpected last config number");

        vm.stopPrank();

        // new change with a validator leaving
        changes = new StakingChangeRequest[](1);

        changes[0] = StakingChangeRequest({
            configurationNumber: 3,
            change: StakingChange({validator: val1, op: StakingOperation.Withdraw, payload: abi.encode(amount)})
        });

        vm.startPrank(FilAddress.SYSTEM_ACTOR);

        gwRouter.storeValidatorChanges(changes);
        configNumber = gwRouter.applyFinalityChanges();
        require(configNumber == 3, "wrong config number after applying finality");
        require(gwGetter.getLastConfigurationNumber() == 2, "apply result: unexpected last config number");
        require(gwGetter.getCurrentConfigurationNumber() == 3, "apply result: unexpected config number");
        require(gwGetter.getCurrentMembership().validators.length == 1, "current membership should be 1");
        require(gwGetter.getLastMembership().validators.length == 2, "last membership should be 2");

        // no changes
        configNumber = gwRouter.applyFinalityChanges();
        require(configNumber == 0, "wrong config number after applying finality");
        require(gwGetter.getLastConfigurationNumber() == 2, "no changes: unexpected last config number");
        require(gwGetter.getCurrentConfigurationNumber() == 3, "no changes: unexpected config number");
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

        require(committedFinality.height == finality.height, "heights are not equal");
        require(committedFinality.blockHash == finality.blockHash, "blockHash is not equal");
        require(gwGetter.getLatestParentFinality().height == block.number, "finality height not equal");

        vm.stopPrank();
    }

    function testGatewayDiamond_createBottomUpCheckpoint() public {
        (, address[] memory addrs, uint256[] memory weights) = TestUtils.getFourValidators(vm);

        (bytes32 membershipRoot, ) = MerkleTreeHelper.createMerkleProofsForValidators(addrs, weights);

        BottomUpCheckpoint memory old = BottomUpCheckpoint({
            subnetID: gwGetter.getNetworkName(),
            blockHeight: 0,
            blockHash: keccak256("block1"),
            nextConfigurationNumber: 1
        });

        BottomUpCheckpoint memory checkpoint = BottomUpCheckpoint({
            subnetID: gwGetter.getNetworkName(),
            blockHeight: gwGetter.bottomUpCheckPeriod(),
            blockHash: keccak256("block1"),
            nextConfigurationNumber: 1
        });

        // failed to create a checkpoint with zero membership weight
        vm.startPrank(FilAddress.SYSTEM_ACTOR);
        vm.expectRevert(ZeroMembershipWeight.selector);
        gwRouter.createBottomUpCheckpoint(checkpoint, membershipRoot, 0);
        vm.stopPrank();

        // failed create a processed checkpoint
        vm.startPrank(FilAddress.SYSTEM_ACTOR);
        vm.expectRevert(QuorumAlreadyProcessed.selector);
        gwRouter.createBottomUpCheckpoint(old, membershipRoot, weights[0] + weights[1] + weights[2]);
        vm.stopPrank();

        // create a checkpoint
        vm.startPrank(FilAddress.SYSTEM_ACTOR);
        gwRouter.createBottomUpCheckpoint(checkpoint, membershipRoot, weights[0] + weights[1] + weights[2]);
        vm.stopPrank();

        BottomUpCheckpoint memory recv = gwGetter.bottomUpCheckpoint(gwGetter.bottomUpCheckPeriod());
        require(recv.nextConfigurationNumber == 1, "nextConfigurationNumber incorrect");
        require(recv.blockHash == keccak256("block1"), "block hash incorrect");
        uint256 d = gwGetter.bottomUpCheckPeriod();

        // failed to create a checkpoint with the same height
        checkpoint = BottomUpCheckpoint({
            subnetID: gwGetter.getNetworkName(),
            blockHeight: d,
            blockHash: keccak256("block"),
            nextConfigurationNumber: 2
        });

        vm.startPrank(FilAddress.SYSTEM_ACTOR);
        vm.expectRevert(CheckpointAlreadyExists.selector);
        gwRouter.createBottomUpCheckpoint(checkpoint, membershipRoot, weights[0] + weights[1] + weights[2]);
        vm.stopPrank();

        // failed to create a checkpoint with the height not multiple to checkpoint period
        checkpoint = BottomUpCheckpoint({
            subnetID: gwGetter.getNetworkName(),
            blockHeight: d + d / 2,
            blockHash: keccak256("block2"),
            nextConfigurationNumber: 2
        });

        vm.startPrank(FilAddress.SYSTEM_ACTOR);
        vm.expectRevert(InvalidCheckpointEpoch.selector);
        gwRouter.createBottomUpCheckpoint(checkpoint, membershipRoot, weights[0] + weights[1] + weights[2]);
        vm.stopPrank();

        (bool ok, uint256 e, ) = gwGetter.getCurrentBottomUpCheckpoint();
        require(ok, "checkpoint not exist");
        require(e == d, "out height incorrect");
    }

    function testGatewayDiamond_commitBottomUpCheckpoint_InvalidCheckpointSource() public {
        BottomUpCheckpoint memory checkpoint = BottomUpCheckpoint({
            subnetID: gwGetter.getNetworkName(),
            blockHeight: gwGetter.bottomUpCheckPeriod(),
            blockHash: keccak256("block1"),
            nextConfigurationNumber: 1
        });

        vm.expectRevert(InvalidCheckpointSource.selector);
        gwRouter.commitCheckpoint(checkpoint);
    }

    function testGatewayDiamond_commitBottomUpCheckpoint_Works_NoMessages() public {
        address caller = address(saDiamond);
        vm.startPrank(caller);
        vm.deal(caller, DEFAULT_COLLATERAL_AMOUNT + DEFAULT_CROSS_MSG_FEE);
        registerSubnet(DEFAULT_COLLATERAL_AMOUNT, caller);
        vm.stopPrank();

        (SubnetID memory subnetId, , , , , ) = getSubnet(address(caller));

        BottomUpCheckpoint memory checkpoint = BottomUpCheckpoint({
            subnetID: subnetId,
            blockHeight: gwGetter.bottomUpCheckPeriod(),
            blockHash: keccak256("block1"),
            nextConfigurationNumber: 1
        });

        vm.prank(caller);
        gwRouter.commitCheckpoint(checkpoint);
    }

    function testGatewayDiamond_listIncompleteCheckpoints() public {
        (, address[] memory addrs, uint256[] memory weights) = TestUtils.getFourValidators(vm);

        (bytes32 membershipRoot, ) = MerkleTreeHelper.createMerkleProofsForValidators(addrs, weights);

        BottomUpCheckpoint memory checkpoint1 = BottomUpCheckpoint({
            subnetID: gwGetter.getNetworkName(),
            blockHeight: gwGetter.bottomUpCheckPeriod(),
            blockHash: keccak256("block1"),
            nextConfigurationNumber: 1
        });

        BottomUpCheckpoint memory checkpoint2 = BottomUpCheckpoint({
            subnetID: gwGetter.getNetworkName(),
            blockHeight: 2 * gwGetter.bottomUpCheckPeriod(),
            blockHash: keccak256("block2"),
            nextConfigurationNumber: 1
        });

        // create a checkpoint
        vm.startPrank(FilAddress.SYSTEM_ACTOR);
        gwRouter.createBottomUpCheckpoint(checkpoint1, membershipRoot, weights[0] + weights[1] + weights[2]);
        gwRouter.createBottomUpCheckpoint(checkpoint2, membershipRoot, weights[0] + weights[1] + weights[2]);
        vm.stopPrank();

        uint256[] memory heights = gwGetter.getIncompleteCheckpointHeights();

        require(heights.length == 2, "unexpected heights");
        require(heights[0] == gwGetter.bottomUpCheckPeriod(), "heights[0] == period");
        require(heights[1] == 2 * gwGetter.bottomUpCheckPeriod(), "heights[1] == 2*period");

        QuorumInfo memory info = gwGetter.getCheckpointInfo(gwGetter.bottomUpCheckPeriod());
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
        (uint256[] memory privKeys, address[] memory addrs, uint256[] memory weights) = TestUtils.getFourValidators(vm);

        (bytes32 membershipRoot, bytes32[][] memory membershipProofs) = MerkleTreeHelper
            .createMerkleProofsForValidators(addrs, weights);

        BottomUpCheckpoint memory checkpoint = BottomUpCheckpoint({
            subnetID: gwGetter.getNetworkName(),
            blockHeight: gwGetter.bottomUpCheckPeriod(),
            blockHash: keccak256("block"),
            nextConfigurationNumber: 1
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
            "checkpoint weight was not updated"
        );

        (
            BottomUpCheckpoint memory ch,
            QuorumInfo memory info,
            address[] memory signatories,
            bytes[] memory signatures
        ) = gwGetter.getCheckpointSignatureBundle(gwGetter.bottomUpCheckPeriod());
        require(ch.blockHash == keccak256("block"), "unexpected block hash");
        require(info.hash == keccak256(abi.encode(checkpoint)), "unexpected checkpoint hash");
        require(signatories.length == 3, "unexpected signatories length");
        require(signatures.length == 3, "unexpected signatures length");
    }

    function testGatewayDiamond_addCheckpointSignature_quorum() public {
        (uint256[] memory privKeys, address[] memory addrs, uint256[] memory weights) = TestUtils.getFourValidators(vm);

        (bytes32 membershipRoot, bytes32[][] memory membershipProofs) = MerkleTreeHelper
            .createMerkleProofsForValidators(addrs, weights);

        BottomUpCheckpoint memory checkpoint = BottomUpCheckpoint({
            subnetID: gwGetter.getNetworkName(),
            blockHeight: gwGetter.bottomUpCheckPeriod(),
            blockHash: keccak256("block"),
            nextConfigurationNumber: 1
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

        QuorumInfo memory info = gwGetter.getCheckpointInfo(1);
        require(!info.reached, "not reached");
        require(gwGetter.getIncompleteCheckpointHeights().length == 1, "unexpected size");

        info = gwGetter.getCheckpointInfo(1);

        (v, r, s) = vm.sign(privKeys[2], keccak256(abi.encode(checkpoint)));
        signature = abi.encodePacked(r, s, v);

        vm.startPrank(vm.addr(privKeys[2]));
        gwRouter.addCheckpointSignature(checkpoint.blockHeight, membershipProofs[2], weights[2], signature);
        vm.stopPrank();

        info = gwGetter.getCheckpointInfo(checkpoint.blockHeight);
        require(info.reached, "not reached");
        require(gwGetter.getIncompleteCheckpointHeights().length == 0, "unexpected size");

        require(
            gwGetter.getCheckpointCurrentWeight(checkpoint.blockHeight) == totalWeight(weights),
            "checkpoint weight was not updated"
        );
        (v, r, s) = vm.sign(privKeys[3], keccak256(abi.encode(checkpoint)));
        signature = abi.encodePacked(r, s, v);

        vm.startPrank(vm.addr(privKeys[3]));
        gwRouter.addCheckpointSignature(checkpoint.blockHeight, membershipProofs[3], weights[3], signature);
        vm.stopPrank();
    }

    function testGatewayDiamond_addCheckpointSignature_notAuthorized() public {
        (uint256[] memory privKeys, address[] memory addrs, uint256[] memory weights) = TestUtils.getFourValidators(vm);

        (bytes32 membershipRoot, bytes32[][] memory membershipProofs) = MerkleTreeHelper
            .createMerkleProofsForValidators(addrs, weights);

        BottomUpCheckpoint memory checkpoint = BottomUpCheckpoint({
            subnetID: gwGetter.getNetworkName(),
            blockHeight: gwGetter.bottomUpCheckPeriod(),
            blockHash: keccak256("block"),
            nextConfigurationNumber: 1
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

        uint256 h = gwGetter.bottomUpCheckPeriod();
        vm.startPrank(vm.addr(privKeys[1]));
        vm.expectRevert(abi.encodeWithSelector(NotAuthorized.selector, vm.addr(privKeys[0])));
        gwRouter.addCheckpointSignature(h, membershipProofs[2], weights[2], signature);
        vm.stopPrank();
    }

    function testGatewayDiamond_addCheckpointSignature_invalidSignature_replayedSignature() public {
        (uint256[] memory privKeys, address[] memory addrs, uint256[] memory weights) = TestUtils.getFourValidators(vm);

        (bytes32 membershipRoot, bytes32[][] memory membershipProofs) = MerkleTreeHelper
            .createMerkleProofsForValidators(addrs, weights);

        BottomUpCheckpoint memory checkpoint = BottomUpCheckpoint({
            subnetID: gwGetter.getNetworkName(),
            blockHeight: gwGetter.bottomUpCheckPeriod(),
            blockHash: keccak256("block"),
            nextConfigurationNumber: 1
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

        uint256 h = gwGetter.bottomUpCheckPeriod();
        vm.startPrank(vm.addr(privKeys[0]));

        // send incorrect signature
        vm.expectRevert(InvalidSignature.selector);
        gwRouter.addCheckpointSignature(h, membershipProofs[0], weights[0], new bytes(0));

        // send correct signature
        gwRouter.addCheckpointSignature(h, membershipProofs[0], weights[0], signature);

        // replay the previous signature
        vm.expectRevert(SignatureReplay.selector);
        gwRouter.addCheckpointSignature(h, membershipProofs[0], weights[0], signature);

        vm.stopPrank();
    }

    function testGatewayDiamond_addCheckpointSignature_incorrectCheckpoint() public {
        (uint256[] memory privKeys, address[] memory addrs, uint256[] memory weights) = TestUtils.getFourValidators(vm);

        (bytes32 membershipRoot, bytes32[][] memory membershipProofs) = MerkleTreeHelper
            .createMerkleProofsForValidators(addrs, weights);

        BottomUpCheckpoint memory checkpoint = BottomUpCheckpoint({
            subnetID: gwGetter.getNetworkName(),
            blockHeight: gwGetter.bottomUpCheckPeriod(),
            blockHash: keccak256("block"),
            nextConfigurationNumber: 1
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

        // send correct signature for incorrect height
        vm.expectRevert(QuorumAlreadyProcessed.selector);
        gwRouter.addCheckpointSignature(0, membershipProofs[0], weights[0], signature);

        // send correct signature for incorrect height
        vm.expectRevert(CheckpointNotCreated.selector);
        gwRouter.addCheckpointSignature(100, membershipProofs[0], weights[0], signature);

        vm.stopPrank();
    }

    function testGatewayDiamond_garbage_collect_bottomUpCheckpoints() public {
        (, address[] memory addrs, uint256[] memory weights) = TestUtils.getFourValidators(vm);

        (bytes32 membershipRoot, ) = MerkleTreeHelper.createMerkleProofsForValidators(addrs, weights);

        uint256 index = gwGetter.getCheckpointRetentionHeight();
        require(index == 1, "unexpected height");

        BottomUpCheckpoint memory checkpoint;

        // create a checkpoint
        uint64 n = 10;
        vm.startPrank(FilAddress.SYSTEM_ACTOR);
        for (uint64 i = 1; i <= n; i++) {
            checkpoint = BottomUpCheckpoint({
                subnetID: gwGetter.getNetworkName(),
                blockHeight: i * gwGetter.bottomUpCheckPeriod(),
                blockHash: keccak256("block"),
                nextConfigurationNumber: 1
            });

            gwRouter.createBottomUpCheckpoint(checkpoint, membershipRoot, 10);
        }
        vm.stopPrank();

        index = gwGetter.getCheckpointRetentionHeight();
        require(index == 1, "retention height is not 1");

        uint256[] memory heights = gwGetter.getIncompleteCheckpointHeights();
        require(heights.length == n, "heights.len is not n");

        vm.startPrank(FilAddress.SYSTEM_ACTOR);
        gwRouter.pruneBottomUpCheckpoints(4);
        vm.stopPrank();

        index = gwGetter.getCheckpointRetentionHeight();
        require(index == 4, "height was not updated");
        heights = gwGetter.getIncompleteCheckpointHeights();
        require(heights.length == n, "index is not the same");
    }

    function testGatewayDiamond_create_bottomUpMsgBatch() public {
        (, address[] memory addrs, uint256[] memory weights) = TestUtils.getFourValidators(vm);

        (bytes32 membershipRoot, ) = MerkleTreeHelper.createMerkleProofsForValidators(addrs, weights);
        BottomUpMsgBatch memory old = BottomUpMsgBatch({
            subnetID: gwGetter.getNetworkName(),
            blockHeight: 0,
            msgs: newListOfMessages(10)
        });

        uint256 d = gwGetter.bottomUpMsgBatchPeriod();
        vm.roll(d + 1);
        BottomUpMsgBatch memory batch = BottomUpMsgBatch({
            subnetID: gwGetter.getNetworkName(),
            blockHeight: d,
            msgs: newListOfMessages(10)
        });

        // failed to create a batch with zero membership weight
        vm.startPrank(FilAddress.SYSTEM_ACTOR);
        vm.expectRevert(ZeroMembershipWeight.selector);
        gwRouter.createBottomUpMsgBatch(batch, membershipRoot, 0);
        vm.stopPrank();

        // failed create a processed batch
        vm.startPrank(FilAddress.SYSTEM_ACTOR);
        vm.expectRevert(QuorumAlreadyProcessed.selector);
        gwRouter.createBottomUpMsgBatch(old, membershipRoot, weights[0] + weights[1] + weights[2]);
        vm.stopPrank();

        // create a batch that hasn't been fully filled (trigger at the batch period).
        vm.startPrank(FilAddress.SYSTEM_ACTOR);
        gwRouter.createBottomUpMsgBatch(batch, membershipRoot, weights[0] + weights[1] + weights[2]);
        vm.stopPrank();

        BottomUpMsgBatch memory recv = gwGetter.bottomUpMsgBatch(gwGetter.bottomUpMsgBatchPeriod());
        require(recv.msgs.length == 10, "msgs length incorrect");

        // failed to create a batch with the same height
        vm.startPrank(FilAddress.SYSTEM_ACTOR);
        vm.expectRevert(BatchAlreadyExists.selector);
        gwRouter.createBottomUpMsgBatch(batch, membershipRoot, weights[0] + weights[1] + weights[2]);
        vm.stopPrank();

        // failed to create a batch with the height not multiple of the batch period
        vm.roll(2 * d + 1);
        batch = BottomUpMsgBatch({
            subnetID: gwGetter.getNetworkName(),
            blockHeight: d + d / 2,
            msgs: newListOfMessages(10)
        });

        vm.startPrank(FilAddress.SYSTEM_ACTOR);
        vm.expectRevert(InvalidBatchEpoch.selector);
        gwRouter.createBottomUpMsgBatch(batch, membershipRoot, weights[0] + weights[1] + weights[2]);
        vm.stopPrank();

        BottomUpMsgBatch memory b = gwGetter.bottomUpMsgBatch(2 * d);
        require(b.blockHeight == 0, "batch shouldn't exist");

        // failed to create a batch with no messages
        batch = BottomUpMsgBatch({subnetID: gwGetter.getNetworkName(), blockHeight: 2 * d, msgs: new CrossMsg[](0)});

        vm.startPrank(FilAddress.SYSTEM_ACTOR);
        vm.expectRevert(BatchWithNoMessages.selector);
        gwRouter.createBottomUpMsgBatch(batch, membershipRoot, weights[0] + weights[1] + weights[2]);
        vm.stopPrank();

        // failed to create a batch with too many messages
        batch = BottomUpMsgBatch({
            subnetID: gwGetter.getNetworkName(),
            blockHeight: 2 * d,
            msgs: newListOfMessages(gwGetter.maxMsgsPerBottomUpBatch() + 1)
        });

        vm.startPrank(FilAddress.SYSTEM_ACTOR);
        vm.expectRevert(MaxMsgsPerBatchExceeded.selector);
        gwRouter.createBottomUpMsgBatch(batch, membershipRoot, weights[0] + weights[1] + weights[2]);
        vm.stopPrank();
    }

    function testGatewayDiamond_commitBatch_InvalidCheckpointSource() public {
        BottomUpMsgBatch memory batch = BottomUpMsgBatch({
            subnetID: gwGetter.getNetworkName(),
            blockHeight: gwGetter.bottomUpMsgBatchPeriod(),
            msgs: newListOfMessages(10)
        });

        vm.expectRevert(InvalidBatchSource.selector);
        gwRouter.execBottomUpMsgBatch(batch);
    }

    function testGatewayDiamond_commitCheckpoint_Fails_NoMessages() public {
        address caller = address(saDiamond);
        vm.startPrank(caller);
        vm.deal(caller, DEFAULT_COLLATERAL_AMOUNT + DEFAULT_CROSS_MSG_FEE);
        registerSubnet(DEFAULT_COLLATERAL_AMOUNT, caller);
        vm.stopPrank();

        (SubnetID memory subnetId, , , , , ) = getSubnet(address(caller));

        BottomUpMsgBatch memory batch = BottomUpMsgBatch({
            subnetID: subnetId,
            blockHeight: gwGetter.bottomUpMsgBatchPeriod(),
            msgs: new CrossMsg[](0)
        });

        vm.prank(caller);
        vm.expectRevert(BatchWithNoMessages.selector);
        gwRouter.execBottomUpMsgBatch(batch);
    }

    function testGatewayDiamond_listIncompleteMsgBatches() public {
        (, address[] memory addrs, uint256[] memory weights) = TestUtils.getFourValidators(vm);

        (bytes32 membershipRoot, ) = MerkleTreeHelper.createMerkleProofsForValidators(addrs, weights);

        uint256 d = gwGetter.bottomUpMsgBatchPeriod();
        vm.roll(2 * d + 1);
        BottomUpMsgBatch memory batch1 = BottomUpMsgBatch({
            subnetID: gwGetter.getNetworkName(),
            blockHeight: d,
            msgs: newListOfMessages(10)
        });

        BottomUpMsgBatch memory batch2 = BottomUpMsgBatch({
            subnetID: gwGetter.getNetworkName(),
            blockHeight: 2 * d,
            msgs: newListOfMessages(10)
        });

        // create a batch
        vm.startPrank(FilAddress.SYSTEM_ACTOR);
        gwRouter.createBottomUpMsgBatch(batch1, membershipRoot, weights[0] + weights[1] + weights[2]);
        gwRouter.createBottomUpMsgBatch(batch2, membershipRoot, weights[0] + weights[1] + weights[2]);
        vm.stopPrank();

        uint256[] memory heights = gwGetter.getIncompleteMsgBatchHeights();

        require(heights.length == 2, "unexpected heights");
        require(heights[0] == d, "heights[0] == period");
        require(heights[1] == 2 * d, "heights[1] == 2*period");

        QuorumInfo memory info = gwGetter.getBottomUpMsgBatchInfo(d);
        require(info.rootHash == membershipRoot, "info.rootHash == membershipRoot");
        require(
            info.threshold == gwGetter.getQuorumThreshold(weights[0] + weights[1] + weights[2]),
            "batch 1 correct threshold"
        );

        info = gwGetter.getBottomUpMsgBatchInfo(2 * d);
        require(info.rootHash == membershipRoot, "info.rootHash == membershipRoot");
        require(
            info.threshold == gwGetter.getQuorumThreshold(weights[0] + weights[1] + weights[2]),
            "batch 2 correct threshold"
        );

        BottomUpMsgBatch[] memory incomplete = gwGetter.getIncompleteMsgBatches();
        require(incomplete.length == 2, "incomplete.length == 2");
        require(incomplete[0].blockHeight == d, "incomplete[0].blockHeight");
        require(incomplete[1].blockHeight == 2 * d, "incomplete[1].blockHeight");
    }

    function testGatewayDiamond_addMsgBatchSignature_newMsgBatch() public {
        (uint256[] memory privKeys, address[] memory addrs, uint256[] memory weights) = TestUtils.getFourValidators(vm);

        (bytes32 membershipRoot, bytes32[][] memory membershipProofs) = MerkleTreeHelper
            .createMerkleProofsForValidators(addrs, weights);

        uint256 d = gwGetter.bottomUpMsgBatchPeriod();
        vm.roll(d + 1);
        BottomUpMsgBatch memory batch = BottomUpMsgBatch({
            subnetID: gwGetter.getNetworkName(),
            blockHeight: d,
            msgs: newListOfMessages(10)
        });

        // create a batch
        vm.startPrank(FilAddress.SYSTEM_ACTOR);
        gwRouter.createBottomUpMsgBatch(batch, membershipRoot, weights[0] + weights[1] + weights[2]);
        vm.stopPrank();

        // adds signatures
        uint8 v;
        bytes32 r;
        bytes32 s;
        bytes memory signature;

        for (uint64 i = 0; i < 3; i++) {
            (v, r, s) = vm.sign(privKeys[i], keccak256(abi.encode(batch)));
            signature = abi.encodePacked(r, s, v);

            vm.startPrank(vm.addr(privKeys[i]));
            gwRouter.addBottomUpMsgBatchSignature(batch.blockHeight, membershipProofs[i], weights[i], signature);
            vm.stopPrank();
        }

        require(
            gwGetter.getBottomUpMsgBatchCurrentWeight(batch.blockHeight) == totalWeight(weights),
            "batch weight was not updated"
        );

        (
            BottomUpMsgBatch memory b,
            QuorumInfo memory info,
            address[] memory signatories,
            bytes[] memory signatures
        ) = gwGetter.getBottomUpMsgBatchSignatureBundle(d);
        require(info.hash == keccak256(abi.encode(b)), "unexpected batch hash");
        require(keccak256(abi.encode(batch)) == keccak256(abi.encode(b)), "unexpected batch hash");
        require(signatories.length == 3, "unexpected signatories length");
        require(signatures.length == 3, "unexpected signatures length");
    }

    function testGatewayDiamond_addMsgBatchSignature_quorum() public {
        (uint256[] memory privKeys, address[] memory addrs, uint256[] memory weights) = TestUtils.getFourValidators(vm);

        (bytes32 membershipRoot, bytes32[][] memory membershipProofs) = MerkleTreeHelper
            .createMerkleProofsForValidators(addrs, weights);

        uint256 d = gwGetter.bottomUpMsgBatchPeriod();
        vm.roll(d + 1);
        BottomUpMsgBatch memory batch = BottomUpMsgBatch({
            subnetID: gwGetter.getNetworkName(),
            blockHeight: d,
            msgs: newListOfMessages(10)
        });

        // create a batch
        vm.startPrank(FilAddress.SYSTEM_ACTOR);
        gwRouter.createBottomUpMsgBatch(batch, membershipRoot, weights[0] + weights[1] + weights[2]);
        vm.stopPrank();

        // adds signatures

        uint8 v;
        bytes32 r;
        bytes32 s;
        bytes memory signature;

        for (uint64 i = 0; i < 2; i++) {
            (v, r, s) = vm.sign(privKeys[i], keccak256(abi.encode(batch)));
            signature = abi.encodePacked(r, s, v);

            vm.startPrank(vm.addr(privKeys[i]));
            gwRouter.addBottomUpMsgBatchSignature(batch.blockHeight, membershipProofs[i], weights[i], signature);
            vm.stopPrank();
        }

        QuorumInfo memory info = gwGetter.getBottomUpMsgBatchInfo(1);
        require(!info.reached, "not reached");
        require(gwGetter.getIncompleteMsgBatchHeights().length == 1, "unexpected size");

        info = gwGetter.getBottomUpMsgBatchInfo(1);

        (v, r, s) = vm.sign(privKeys[2], keccak256(abi.encode(batch)));
        signature = abi.encodePacked(r, s, v);

        vm.startPrank(vm.addr(privKeys[2]));
        gwRouter.addBottomUpMsgBatchSignature(batch.blockHeight, membershipProofs[2], weights[2], signature);
        vm.stopPrank();

        info = gwGetter.getBottomUpMsgBatchInfo(batch.blockHeight);
        require(info.reached, "not reached");
        require(gwGetter.getIncompleteMsgBatchHeights().length == 0, "unexpected size");

        require(
            gwGetter.getBottomUpMsgBatchCurrentWeight(batch.blockHeight) == totalWeight(weights),
            "batch weight was not updated"
        );
        (v, r, s) = vm.sign(privKeys[3], keccak256(abi.encode(batch)));
        signature = abi.encodePacked(r, s, v);

        vm.startPrank(vm.addr(privKeys[3]));
        gwRouter.addBottomUpMsgBatchSignature(batch.blockHeight, membershipProofs[3], weights[3], signature);
        vm.stopPrank();
    }

    function testGatewayDiamond_addMsgBatch_notAuthorized() public {
        (uint256[] memory privKeys, address[] memory addrs, uint256[] memory weights) = TestUtils.getFourValidators(vm);

        (bytes32 membershipRoot, bytes32[][] memory membershipProofs) = MerkleTreeHelper
            .createMerkleProofsForValidators(addrs, weights);

        uint256 d = gwGetter.bottomUpMsgBatchPeriod();
        vm.roll(d + 1);
        BottomUpMsgBatch memory batch = BottomUpMsgBatch({
            subnetID: gwGetter.getNetworkName(),
            blockHeight: d,
            msgs: newListOfMessages(10)
        });

        // create a batch
        vm.startPrank(FilAddress.SYSTEM_ACTOR);
        gwRouter.createBottomUpMsgBatch(batch, membershipRoot, 10);
        vm.stopPrank();

        uint8 v;
        bytes32 r;
        bytes32 s;
        bytes memory signature;

        (v, r, s) = vm.sign(privKeys[0], keccak256(abi.encode(batch)));
        signature = abi.encodePacked(r, s, v);

        vm.startPrank(vm.addr(privKeys[1]));
        vm.expectRevert(abi.encodeWithSelector(NotAuthorized.selector, vm.addr(privKeys[0])));
        gwRouter.addBottomUpMsgBatchSignature(d, membershipProofs[2], weights[2], signature);
        vm.stopPrank();
    }

    function testGatewayDiamond_addMsgBatchSignature_invalidSignature_replayedSignature() public {
        (uint256[] memory privKeys, address[] memory addrs, uint256[] memory weights) = TestUtils.getFourValidators(vm);

        (bytes32 membershipRoot, bytes32[][] memory membershipProofs) = MerkleTreeHelper
            .createMerkleProofsForValidators(addrs, weights);

        uint256 d = gwGetter.bottomUpMsgBatchPeriod();
        vm.roll(d + 1);
        BottomUpMsgBatch memory batch = BottomUpMsgBatch({
            subnetID: gwGetter.getNetworkName(),
            blockHeight: d,
            msgs: newListOfMessages(10)
        });

        // create a batch
        vm.startPrank(FilAddress.SYSTEM_ACTOR);
        gwRouter.createBottomUpMsgBatch(batch, membershipRoot, 10);
        vm.stopPrank();

        uint8 v;
        bytes32 r;
        bytes32 s;
        bytes memory signature;

        (v, r, s) = vm.sign(privKeys[0], keccak256(abi.encode(batch)));
        signature = abi.encodePacked(r, s, v);

        vm.startPrank(vm.addr(privKeys[0]));

        // send incorrect signature
        vm.expectRevert(InvalidSignature.selector);
        gwRouter.addBottomUpMsgBatchSignature(d, membershipProofs[0], weights[0], new bytes(0));

        // send correct signature
        gwRouter.addBottomUpMsgBatchSignature(d, membershipProofs[0], weights[0], signature);

        // replay the previous signature
        vm.expectRevert(SignatureReplay.selector);
        gwRouter.addBottomUpMsgBatchSignature(d, membershipProofs[0], weights[0], signature);

        vm.stopPrank();
    }

    function testGatewayDiamond_addMsgBatchSignature_incorrectbatch() public {
        (uint256[] memory privKeys, address[] memory addrs, uint256[] memory weights) = TestUtils.getFourValidators(vm);

        (bytes32 membershipRoot, bytes32[][] memory membershipProofs) = MerkleTreeHelper
            .createMerkleProofsForValidators(addrs, weights);

        uint256 d = gwGetter.bottomUpMsgBatchPeriod();
        vm.roll(d + 1);
        BottomUpMsgBatch memory batch = BottomUpMsgBatch({
            subnetID: gwGetter.getNetworkName(),
            blockHeight: d,
            msgs: newListOfMessages(10)
        });

        // create a batch
        vm.startPrank(FilAddress.SYSTEM_ACTOR);
        gwRouter.createBottomUpMsgBatch(batch, membershipRoot, 10);
        vm.stopPrank();

        uint8 v;
        bytes32 r;
        bytes32 s;
        bytes memory signature;

        (v, r, s) = vm.sign(privKeys[0], keccak256(abi.encode(batch)));
        signature = abi.encodePacked(r, s, v);

        vm.startPrank(vm.addr(privKeys[0]));

        // send correct signature for incorrect height
        vm.expectRevert(QuorumAlreadyProcessed.selector);
        gwRouter.addBottomUpMsgBatchSignature(0, membershipProofs[0], weights[0], signature);

        // send correct signature for incorrect height
        vm.expectRevert(BatchNotCreated.selector);
        gwRouter.addBottomUpMsgBatchSignature(d + 1, membershipProofs[0], weights[0], signature);

        gwRouter.addBottomUpMsgBatchSignature(d, membershipProofs[0], weights[0], signature);

        vm.stopPrank();
    }

    function testGatewayDiamond_garbage_collect_msgBatch() public {
        (, address[] memory addrs, uint256[] memory weights) = TestUtils.getFourValidators(vm);

        (bytes32 membershipRoot, ) = MerkleTreeHelper.createMerkleProofsForValidators(addrs, weights);

        uint256 index = gwGetter.getBottomUpMsgRetentionHeight();
        require(index == 1, "unexpected height");

        BottomUpMsgBatch memory batch;

        // create a batch
        uint256 d = gwGetter.bottomUpMsgBatchPeriod();
        uint64 n = 10;
        vm.roll(n * d + n);
        vm.startPrank(FilAddress.SYSTEM_ACTOR);
        for (uint64 i = 1; i <= n; i++) {
            batch = BottomUpMsgBatch({
                subnetID: gwGetter.getNetworkName(),
                blockHeight: i * d,
                msgs: newListOfMessages(10)
            });

            gwRouter.createBottomUpMsgBatch(batch, membershipRoot, 10);
        }
        vm.stopPrank();

        index = gwGetter.getBottomUpMsgRetentionHeight();
        require(index == 1, "retention height is not 1");

        uint256[] memory heights = gwGetter.getIncompleteMsgBatchHeights();
        require(heights.length == n, "heights.len is not n");

        vm.startPrank(FilAddress.SYSTEM_ACTOR);
        gwRouter.pruneBottomUpMsgBatches(4);
        vm.stopPrank();

        index = gwGetter.getBottomUpMsgRetentionHeight();
        require(index == 4, "height was not updated");
        heights = gwGetter.getIncompleteMsgBatchHeights();
        require(heights.length == n, "index is not the same");
    }

    function testGatewayDiamond_execMsgBatch_WithMessages() public {
        address caller = address(saDiamond);
        address from = address(100);
        vm.startPrank(caller);
        vm.deal(caller, 2 * DEFAULT_COLLATERAL_AMOUNT + DEFAULT_CROSS_MSG_FEE);
        registerSubnet(DEFAULT_COLLATERAL_AMOUNT, caller);
        vm.stopPrank();

        uint256 amount = 1;

        (SubnetID memory subnetId, , , , , ) = getSubnet(address(caller));
        (bool exist, Subnet memory subnetInfo) = gwGetter.getSubnet(subnetId);
        require(exist, "subnet does not exist");
        require(subnetInfo.circSupply == 0, "unexpected initial circulation supply");

        gwManager.fund{value: DEFAULT_COLLATERAL_AMOUNT}(subnetId, FvmAddressHelper.from(address(caller)));
        (, subnetInfo) = gwGetter.getSubnet(subnetId);
        require(subnetInfo.circSupply == DEFAULT_COLLATERAL_AMOUNT, "unexpected circulation supply after funding");

        CrossMsg[] memory msgs = new CrossMsg[](10);
        for (uint64 i = 0; i < 10; i++) {
            msgs[i] = CrossMsg({
                message: StorableMsg({
                    from: IPCAddress({subnetId: subnetId, rawAddress: FvmAddressHelper.from(from)}),
                    to: IPCAddress({subnetId: gwGetter.getNetworkName(), rawAddress: FvmAddressHelper.from(from)}),
                    value: amount,
                    nonce: i,
                    method: METHOD_SEND,
                    params: EMPTY_BYTES,
                    fee: DEFAULT_CROSS_MSG_FEE
                }),
                wrapped: false
            });
        }

        uint256 d = gwGetter.bottomUpMsgBatchPeriod();
        vm.roll(d + 1);
        BottomUpMsgBatch memory batch = BottomUpMsgBatch({subnetID: subnetId, blockHeight: d, msgs: msgs});

        vm.prank(caller);
        gwRouter.execBottomUpMsgBatch(batch);

        (, subnetInfo) = gwGetter.getSubnet(subnetId);
        require(
            subnetInfo.circSupply == DEFAULT_COLLATERAL_AMOUNT - 10 * DEFAULT_CROSS_MSG_FEE - 10 * amount,
            "unexpected circulation supply"
        );
    }

    function testGatewayDiamond_execMsgBatch_Fails_WrongNumberMessages() public {
        address caller = address(saDiamond);
        address from = address(100);
        vm.startPrank(caller);
        vm.deal(caller, 2 * DEFAULT_COLLATERAL_AMOUNT + DEFAULT_CROSS_MSG_FEE);
        registerSubnet(DEFAULT_COLLATERAL_AMOUNT, caller);
        vm.stopPrank();

        uint256 amount = 1;

        (SubnetID memory subnetId, , , , , ) = getSubnet(address(caller));
        (bool exist, Subnet memory subnetInfo) = gwGetter.getSubnet(subnetId);
        require(exist, "subnet does not exist");
        require(subnetInfo.circSupply == 0, "unexpected initial circulation supply");

        gwManager.fund{value: DEFAULT_COLLATERAL_AMOUNT}(subnetId, FvmAddressHelper.from(address(caller)));
        (, subnetInfo) = gwGetter.getSubnet(subnetId);
        require(subnetInfo.circSupply == DEFAULT_COLLATERAL_AMOUNT, "unexpected circulation supply after funding");

        uint64 size = gwGetter.maxMsgsPerBottomUpBatch() + 1;
        CrossMsg[] memory msgs = new CrossMsg[](size);
        for (uint64 i = 0; i < size; i++) {
            msgs[i] = CrossMsg({
                message: StorableMsg({
                    from: IPCAddress({subnetId: subnetId, rawAddress: FvmAddressHelper.from(from)}),
                    to: IPCAddress({subnetId: gwGetter.getNetworkName(), rawAddress: FvmAddressHelper.from(from)}),
                    value: amount,
                    nonce: i,
                    method: METHOD_SEND,
                    params: EMPTY_BYTES,
                    fee: DEFAULT_CROSS_MSG_FEE
                }),
                wrapped: false
            });
        }

        // fail with exceeded messages
        uint256 d = gwGetter.bottomUpMsgBatchPeriod();
        vm.roll(d + 1);
        BottomUpMsgBatch memory batch = BottomUpMsgBatch({subnetID: subnetId, blockHeight: d, msgs: msgs});

        vm.prank(caller);
        vm.expectRevert(MaxMsgsPerBatchExceeded.selector);
        gwRouter.execBottomUpMsgBatch(batch);

        // fail with no messages
        batch = BottomUpMsgBatch({subnetID: subnetId, blockHeight: d, msgs: new CrossMsg[](0)});

        vm.prank(caller);
        vm.expectRevert(BatchWithNoMessages.selector);
        gwRouter.execBottomUpMsgBatch(batch);
    }

    function testGatewayDiamond_PopulateBottomUpMsgBatch_Works() public {
        uint256 releaseAmount = 10;
        address from = address(100);

        address[] memory path = new address[](2);
        path[0] = makeAddr("root");
        path[1] = makeAddr("subnet_one");

        GatewayDiamond.ConstructorParams memory constructorParams = GatewayDiamond.ConstructorParams({
            networkName: SubnetID({root: ROOTNET_CHAINID, route: path}),
            bottomUpCheckPeriod: DEFAULT_CHECKPOINT_PERIOD,
            msgFee: DEFAULT_CROSS_MSG_FEE,
            minCollateral: DEFAULT_COLLATERAL_AMOUNT,
            majorityPercentage: DEFAULT_MAJORITY_PERCENTAGE,
            genesisValidators: new Validator[](0),
            activeValidatorsLimit: 100
        });
        gatewayDiamond = createGatewayDiamond(constructorParams);
        gwGetter = GatewayGetterFacet(address(gatewayDiamond));
        gwManager = GatewayManagerFacet(address(gatewayDiamond));
        gwRouter = GatewayRouterFacet(address(gatewayDiamond));

        uint256 d = gwGetter.bottomUpMsgBatchPeriod();

        // a few messags in first batch
        uint64 numMsgs = 10;
        vm.roll(1);
        vm.startPrank(from);
        vm.deal(from, numMsgs * (releaseAmount + DEFAULT_CROSS_MSG_FEE));

        for (uint64 i = 0; i < numMsgs; i++) {
            release(releaseAmount);
        }
        require(gwGetter.bottomUpMsgBatch(d).msgs.length == numMsgs, "no messages");

        numMsgs = gwGetter.maxMsgsPerBottomUpBatch() + 10;
        vm.roll(d + 1);
        vm.startPrank(from);
        vm.deal(from, numMsgs * (releaseAmount + DEFAULT_CROSS_MSG_FEE));

        for (uint64 i = 0; i < numMsgs; i++) {
            release(releaseAmount);
        }
        // one batch should be overflow in d+1 and the rest of the messages should have been
        // added to the next batch
        require(
            gwGetter.bottomUpMsgBatch(d + 1).msgs.length == gwGetter.maxMsgsPerBottomUpBatch(),
            "wrong number of messages in full batch"
        );
        require(gwGetter.bottomUpMsgBatch(2 * d).msgs.length == 10, "wrong number of messages after full batch");
    }

    function newListOfMessages(uint64 size) internal view returns (CrossMsg[] memory msgs) {
        msgs = new CrossMsg[](size);
        for (uint64 i = 0; i < size; i++) {
            msgs[i] = CrossMsg({
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
                    fee: DEFAULT_CROSS_MSG_FEE
                }),
                wrapped: false
            });
        }
    }

    function callback() public view {}
}
