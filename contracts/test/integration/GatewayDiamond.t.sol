// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity ^0.8.23;

import "forge-std/Test.sol";

import "../../contracts/errors/IPCErrors.sol";
import {NumberContractFacetSeven} from "../helpers/contracts/NumberContractFacetSeven.sol";
import {NumberContractFacetEight} from "../helpers/contracts/NumberContractFacetEight.sol";
import {EMPTY_BYTES, METHOD_SEND} from "../../contracts/constants/Constants.sol";
import {IERC165} from "../../contracts/interfaces/IERC165.sol";
import {IDiamond} from "../../contracts/interfaces/IDiamond.sol";
import {IDiamondLoupe} from "../../contracts/interfaces/IDiamondLoupe.sol";
import {IDiamondCut} from "../../contracts/interfaces/IDiamondCut.sol";
import {QuorumInfo} from "../../contracts/structs/Quorum.sol";
import {IpcEnvelope, BottomUpCheckpoint, ParentFinality} from "../../contracts/structs/CrossNet.sol";
import {FvmAddress} from "../../contracts/structs/FvmAddress.sol";
import {SubnetID, Subnet, IPCAddress, Validator, PowerChange, PowerChangeRequest, Asset, PowerOperation} from "../../contracts/structs/Subnet.sol";
import {SubnetIDHelper} from "../../contracts/lib/SubnetIDHelper.sol";
import {FvmAddressHelper} from "../../contracts/lib/FvmAddressHelper.sol";
import {CrossMsgHelper} from "../../contracts/lib/CrossMsgHelper.sol";
import {FilAddress} from "fevmate/contracts/utils/FilAddress.sol";
import {GatewayDiamond, FunctionNotFound} from "../../contracts/GatewayDiamond.sol";
import {GatewayGetterFacet} from "../../contracts/gateway/GatewayGetterFacet.sol";
import {GatewayManagerFacet} from "../../contracts/gateway/GatewayManagerFacet.sol";

import {CheckpointingFacet} from "../../contracts/gateway/router/CheckpointingFacet.sol";
import {XnetMessagingFacet} from "../../contracts/gateway/router/XnetMessagingFacet.sol";
import {TopDownFinalityFacet} from "../../contracts/gateway/router/TopDownFinalityFacet.sol";

import {ERR_GENERAL_CROSS_MSG_DISABLED} from "../../contracts/gateway/GatewayMessengerFacet.sol";
import {DiamondCutFacet} from "../../contracts/diamond/DiamondCutFacet.sol";
import {LibDiamond} from "../../contracts/lib/LibDiamond.sol";
import {MerkleTreeHelper} from "../helpers/MerkleTreeHelper.sol";
import {TestUtils, MockIpcContract, MockIpcContractPayable} from "../helpers/TestUtils.sol";
import {IntegrationTestBase, SubnetWithNativeTokenMock} from "../IntegrationTestBase.sol";
import {SelectorLibrary} from "../helpers/SelectorLibrary.sol";
import {GatewayFacetsHelper} from "../helpers/GatewayFacetsHelper.sol";

import {SubnetActorDiamond} from "../../contracts/SubnetActorDiamond.sol";
import {SubnetActorFacetsHelper} from "../helpers/SubnetActorFacetsHelper.sol";

import {FullActivityRollup, Consensus} from "../../contracts/structs/Activity.sol";
import {ActivityHelper} from "../helpers/ActivityHelper.sol";
import {BottomUpBatchHelper} from "../helpers/BottomUpBatchHelper.sol";

contract GatewayActorDiamondTest is Test, IntegrationTestBase, SubnetWithNativeTokenMock {
    using SubnetIDHelper for SubnetID;
    using CrossMsgHelper for IpcEnvelope;
    using FvmAddressHelper for FvmAddress;
    using GatewayFacetsHelper for GatewayDiamond;
    using SubnetActorFacetsHelper for SubnetActorDiamond;

    function setUp() public override {
        super.setUp();
    }

    function testGatewayDiamond_TransferOwnership() public {
        address owner = gatewayDiamond.ownership().owner();

        vm.expectRevert(LibDiamond.InvalidAddress.selector);
        gatewayDiamond.ownership().transferOwnership(address(0));

        gatewayDiamond.ownership().transferOwnership(address(1));

        address newOwner = gatewayDiamond.ownership().owner();
        require(owner != newOwner, "ownership should be updated");
        require(newOwner == address(1), "new owner not address 1");

        vm.expectRevert(NotOwner.selector);
        gatewayDiamond.ownership().transferOwnership(address(1));
    }

    function testGatewayDiamond_Constructor() public view {
        require(gatewayDiamond.getter().totalSubnets() == 0, "unexpected totalSubnets");
        require(gatewayDiamond.getter().bottomUpNonce() == 0, "unexpected bottomUpNonce");
        require(
            gatewayDiamond.getter().bottomUpCheckPeriod() == DEFAULT_CHECKPOINT_PERIOD,
            "unexpected bottomUpCheckPeriod"
        );
        require(
            gatewayDiamond.getter().getNetworkName().equals(SubnetID({root: ROOTNET_CHAINID, route: new address[](0)})),
            "unexpected getNetworkName"
        );
        require(
            gatewayDiamond.getter().majorityPercentage() == DEFAULT_MAJORITY_PERCENTAGE,
            "unexpected majorityPercentage"
        );

        IpcEnvelope memory storableMsg = gatewayDiamond.getter().postbox(0);
        IpcEnvelope memory msg1;
        require(msg1.toHash() == storableMsg.toHash(), "unexpected hash");
    }

    function testGatewayDiamond_NewGatewayWithDefaultParams() public view {
        GatewayDiamond.ConstructorParams memory params = defaultGatewayParams();

        require(
            gatewayDiamond.getter().bottomUpCheckPeriod() == params.bottomUpCheckPeriod,
            "unexpected bottom-up period"
        );
        require(
            gatewayDiamond.getter().majorityPercentage() == params.majorityPercentage,
            "unexpected majority percentage"
        );
    }

    function testGatewayDiamond_LoupeFunction() public view {
        require(gatewayDiamond.diamondLouper().facets().length == 9, "unexpected length");
        require(
            gatewayDiamond.diamondLouper().supportsInterface(type(IERC165).interfaceId) == true,
            "IERC165 not supported"
        );
        require(
            gatewayDiamond.diamondLouper().supportsInterface(type(IDiamondCut).interfaceId) == true,
            "IDiamondCut not supported"
        );
        require(
            gatewayDiamond.diamondLouper().supportsInterface(type(IDiamondLoupe).interfaceId) == true,
            "IDiamondLoupe not supported"
        );
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

        bytes4[] memory ncGetterSelectors = new bytes4[](1);
        ncGetterSelectors[0] = NumberContractFacetSeven.getNum.selector;

        gwDiamondCut[0] = (
            IDiamond.FacetCut({
                facetAddress: address(ncFacetA),
                action: IDiamond.FacetCutAction.Add,
                functionSelectors: ncGetterSelectors
            })
        );
        //test that other user cannot call diamondcut to add function
        vm.prank(0x1234567890123456789012345678901234567890);
        vm.expectRevert(NotOwner.selector);
        gwDiamondCutter.diamondCut(gwDiamondCut, address(0), new bytes(0));

        gwDiamondCutter.diamondCut(gwDiamondCut, address(0), new bytes(0));

        NumberContractFacetSeven gwNumberContract = NumberContractFacetSeven(address(gatewayDiamond));
        assert(gwNumberContract.getNum() == 7);

        ncGetterSelectors = new bytes4[](1);
        ncGetterSelectors[0] = NumberContractFacetEight.getNum.selector;

        gwDiamondCut[0] = (
            IDiamond.FacetCut({
                facetAddress: address(ncFacetB),
                action: IDiamond.FacetCutAction.Replace,
                functionSelectors: ncGetterSelectors
            })
        );

        //test that other user cannot call diamondcut to replace function
        vm.prank(0x1234567890123456789012345678901234567890);
        vm.expectRevert(NotOwner.selector);
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
        vm.expectRevert(NotOwner.selector);
        gwDiamondCutter.diamondCut(gwDiamondCut, address(0), new bytes(0));

        gwDiamondCutter.diamondCut(gwDiamondCut, address(0), new bytes(0));

        //assert that calling getNum fails
        vm.expectRevert(abi.encodePacked(FunctionNotFound.selector, ncGetterSelectors));
        gwNumberContract.getNum();
    }

    function testGatewayDiamond_Deployment_Works_Root(uint64 checkpointPeriod) public {
        vm.assume(checkpointPeriod >= DEFAULT_CHECKPOINT_PERIOD);

        GatewayDiamond.ConstructorParams memory constructorParams = GatewayDiamond.ConstructorParams({
            networkName: SubnetID({root: ROOTNET_CHAINID, route: new address[](0)}),
            bottomUpCheckPeriod: checkpointPeriod,
            majorityPercentage: DEFAULT_MAJORITY_PERCENTAGE,
            genesisValidators: new Validator[](0),
            activeValidatorsLimit: 100,
            commitSha: DEFAULT_COMMIT_SHA
        });

        GatewayDiamond dep = createGatewayDiamond(constructorParams);
        GatewayGetterFacet depGetter = dep.getter();

        SubnetID memory networkName = depGetter.getNetworkName();

        require(depGetter.getCommitSha() == bytes32(DEFAULT_COMMIT_SHA), "unexpected commit sha");
        require(networkName.isRoot(), "unexpected networkName");
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

        GatewayGetterFacet depGetter = new GatewayGetterFacet();
        GatewayManagerFacet depManager = new GatewayManagerFacet();

        GatewayDiamond.ConstructorParams memory constructorParams = GatewayDiamond.ConstructorParams({
            networkName: SubnetID({root: ROOTNET_CHAINID, route: path}),
            bottomUpCheckPeriod: checkpointPeriod,
            majorityPercentage: 100,
            genesisValidators: new Validator[](0),
            activeValidatorsLimit: 100,
            commitSha: DEFAULT_COMMIT_SHA
        });

        IDiamond.FacetCut[] memory diamondCut = new IDiamond.FacetCut[](2);

        diamondCut[0] = (
            IDiamond.FacetCut({
                facetAddress: address(depManager),
                action: IDiamond.FacetCutAction.Add,
                functionSelectors: gwManagerSelectors
            })
        );

        diamondCut[1] = (
            IDiamond.FacetCut({
                facetAddress: address(depGetter),
                action: IDiamond.FacetCutAction.Add,
                functionSelectors: gwGetterSelectors
            })
        );

        GatewayDiamond dep = new GatewayDiamond(diamondCut, constructorParams);
        depGetter = dep.getter();
        depManager = dep.manager();

        SubnetID memory networkName = depGetter.getNetworkName();

        require(networkName.isRoot() == false, "unexpected networkName");
        require(depGetter.bottomUpCheckPeriod() == checkpointPeriod, "unexpected bottomUpCheckPeriod");
        require(depGetter.majorityPercentage() == 100, "unexpected majorityPercentage");
    }

    function testGatewayDiamond_Register_Works_SingleSubnet(uint256 subnetCollateral) public {
        vm.assume(subnetCollateral < type(uint64).max);
        address subnetAddress = address(new SubnetWithNativeTokenMock());
        vm.prank(subnetAddress);
        vm.deal(subnetAddress, subnetCollateral);

        registerSubnet(subnetCollateral, subnetAddress);
        require(gatewayDiamond.getter().totalSubnets() == 1, "unexpected totalSubnets");
        Subnet[] memory subnets = gatewayDiamond.getter().listSubnets();
        require(subnets.length == 1, "unexpected subnets length");

        SubnetID memory subnetId = gatewayDiamond.getter().getNetworkName().createSubnetId(subnetAddress);

        (bool ok, Subnet memory targetSubnet) = gatewayDiamond.getter().getSubnet(subnetId);

        require(ok, "subnet not found");

        (SubnetID memory id, uint256 stake, , , ) = getSubnet(subnetAddress);

        require(targetSubnet.stake == stake, "unexpected stake");
        require(targetSubnet.stake == subnetCollateral, "unexpected collateral");
        require(id.equals(subnetId), "unexpected id");
    }

    function testGatewayDiamond_Register_Works_MultipleSubnets(uint8 numberOfSubnets) public {
        vm.assume(numberOfSubnets > 0);

        for (uint256 i = 1; i <= numberOfSubnets; i++) {
            address subnetAddress = address(new SubnetWithNativeTokenMock());
            vm.prank(subnetAddress);
            vm.deal(subnetAddress, DEFAULT_COLLATERAL_AMOUNT);

            registerSubnet(DEFAULT_COLLATERAL_AMOUNT, subnetAddress);
        }

        require(gatewayDiamond.getter().totalSubnets() == numberOfSubnets, "unexpected total subnets");
        Subnet[] memory subnets = gatewayDiamond.getter().listSubnets();
        require(subnets.length == numberOfSubnets, "unexpected length");
    }

    function testGatewayDiamond_Register_Fail_SubnetAlreadyExists() public {
        registerSubnet(DEFAULT_COLLATERAL_AMOUNT, address(this));

        vm.expectRevert(AlreadyRegisteredSubnet.selector);

        gatewayDiamond.manager().register{value: DEFAULT_COLLATERAL_AMOUNT}(0, DEFAULT_COLLATERAL_AMOUNT);
    }

    function testGatewayDiamond_AddStake_Works_SingleStaking(uint256 stakeAmount, uint256 registerAmount) public {
        address subnetAddress = address(new SubnetWithNativeTokenMock());
        vm.assume(registerAmount < type(uint64).max);
        vm.assume(stakeAmount > 0 && stakeAmount < type(uint256).max - registerAmount);

        uint256 totalAmount = stakeAmount + registerAmount;

        vm.startPrank(subnetAddress);
        vm.deal(subnetAddress, totalAmount);

        registerSubnet(registerAmount, subnetAddress);

        vm.startPrank(subnetAddress);
        addStake(stakeAmount, subnetAddress);

        (, uint256 totalStaked, , , ) = getSubnet(subnetAddress);

        require(totalStaked == totalAmount, "unexpected staked amount");
    }

    function testGatewayDiamond_AddStake_Works_Reactivate() public {
        address subnetAddress = address(new SubnetWithNativeTokenMock());
        uint256 registerAmount = DEFAULT_COLLATERAL_AMOUNT;
        uint256 stakeAmount = DEFAULT_COLLATERAL_AMOUNT;

        vm.startPrank(subnetAddress);
        vm.deal(subnetAddress, registerAmount);

        registerSubnet(registerAmount, subnetAddress);

        vm.startPrank(subnetAddress);
        gatewayDiamond.manager().releaseStake(registerAmount);

        vm.deal(subnetAddress, stakeAmount);
        addStake(stakeAmount, subnetAddress);

        (, uint256 staked, , , ) = getSubnet(subnetAddress);

        require(staked == stakeAmount, "unexpected amount");
    }

    function testGatewayDiamond_AddStake_Works_NotEnoughFundsToReactivate() public {
        address subnetAddress = address(new SubnetWithNativeTokenMock());
        uint256 registerAmount = DEFAULT_COLLATERAL_AMOUNT;
        uint256 stakeAmount = DEFAULT_COLLATERAL_AMOUNT - 1;

        vm.startPrank(subnetAddress);
        vm.deal(subnetAddress, registerAmount);

        registerSubnet(registerAmount, subnetAddress);

        vm.startPrank(subnetAddress);
        gatewayDiamond.manager().releaseStake(registerAmount);

        vm.deal(subnetAddress, stakeAmount);
        addStake(stakeAmount, subnetAddress);

        (, uint256 staked, , , ) = getSubnet(subnetAddress);

        require(staked == stakeAmount, "unexpected amount");
    }

    function testGatewayDiamond_AddStake_Works_MultipleStakings(uint8 numberOfStakes) public {
        vm.assume(numberOfStakes > 0);

        address subnetAddress = address(new SubnetWithNativeTokenMock());
        uint256 singleStakeAmount = 1 ether;
        uint256 registerAmount = DEFAULT_COLLATERAL_AMOUNT;
        uint256 expectedStakedAmount = registerAmount;

        vm.startPrank(subnetAddress);
        vm.deal(subnetAddress, registerAmount + singleStakeAmount * numberOfStakes);

        registerSubnet(registerAmount, subnetAddress);

        vm.startPrank(subnetAddress);
        for (uint256 i = 0; i < numberOfStakes; i++) {
            addStake(singleStakeAmount, subnetAddress);

            expectedStakedAmount += singleStakeAmount;
        }

        (, uint256 totalStake, , , ) = getSubnet(subnetAddress);

        require(totalStake == expectedStakedAmount, "unexpected stake");
    }

    function testGatewayDiamond_AddStake_Fail_ZeroAmount() public {
        registerSubnet(DEFAULT_COLLATERAL_AMOUNT, address(this));

        vm.expectRevert(NotEnoughFunds.selector);

        gatewayDiamond.manager().addStake{value: 0}(0);
    }

    function testGatewayDiamond_AddStake_Fail_SubnetNotExists() public {
        vm.expectRevert(NotRegisteredSubnet.selector);

        gatewayDiamond.manager().addStake{value: 1}(1);
    }

    function testGatewayDiamond_ReleaseStake_Works_FullAmount(uint256 stakeAmount) public {
        address subnetAddress = CHILD_NETWORK_ADDRESS;
        uint256 registerAmount = DEFAULT_COLLATERAL_AMOUNT;

        vm.assume(stakeAmount > 0 && stakeAmount < type(uint256).max - registerAmount);

        uint256 fullAmount = stakeAmount + registerAmount;

        vm.startPrank(subnetAddress);
        vm.deal(subnetAddress, fullAmount);

        registerSubnet(registerAmount, subnetAddress);

        vm.startPrank(subnetAddress);

        addStake(stakeAmount, subnetAddress);
        gatewayDiamond.manager().releaseStake(fullAmount);

        (, uint256 stake, , , ) = getSubnet(subnetAddress);

        require(stake == 0, "unexpected stake");
        require(subnetAddress.balance == fullAmount, "unexpected balance");
    }

    function testGatewayDiamond_ReleaseStake_Works_SubnetInactive() public {
        address subnetAddress = address(new SubnetWithNativeTokenMock());
        vm.startPrank(subnetAddress);
        vm.deal(subnetAddress, DEFAULT_COLLATERAL_AMOUNT);
        registerSubnet(DEFAULT_COLLATERAL_AMOUNT, subnetAddress);

        vm.startPrank(subnetAddress);
        gatewayDiamond.manager().releaseStake(DEFAULT_COLLATERAL_AMOUNT / 2);

        (, uint256 stake, , , ) = getSubnet(subnetAddress);
        require(stake == DEFAULT_COLLATERAL_AMOUNT / 2, "unexpected stake");
    }

    function testGatewayDiamond_ReleaseStake_Works_PartialAmount(uint256 partialAmount) public {
        address subnetAddress = CHILD_NETWORK_ADDRESS;
        uint256 registerAmount = DEFAULT_COLLATERAL_AMOUNT;

        vm.assume(partialAmount > registerAmount && partialAmount < type(uint256).max - registerAmount);

        uint256 totalAmount = partialAmount + registerAmount;

        vm.startPrank(subnetAddress);
        vm.deal(subnetAddress, totalAmount);

        registerSubnet(registerAmount, subnetAddress);

        vm.startPrank(subnetAddress);

        addStake(partialAmount, subnetAddress);
        gatewayDiamond.manager().releaseStake(partialAmount);

        (, uint256 stake, , , ) = getSubnet(subnetAddress);

        require(stake == registerAmount, "unexpected stake");
        require(subnetAddress.balance == partialAmount, "unexpected balance");
    }

    function testGatewayDiamond_ReleaseStake_Fail_ZeroAmount() public {
        registerSubnet(DEFAULT_COLLATERAL_AMOUNT, address(this));

        vm.expectRevert(CannotReleaseZero.selector);

        gatewayDiamond.manager().releaseStake(0);
    }

    function testGatewayDiamond_ReleaseStake_Fail_InsufficientSubnetBalance(
        uint256 releaseAmount,
        uint256 subnetBalance
    ) public {
        vm.assume(subnetBalance > DEFAULT_COLLATERAL_AMOUNT);
        vm.assume(releaseAmount > subnetBalance && releaseAmount < type(uint256).max - subnetBalance);

        address subnetAddress = address(new SubnetWithNativeTokenMock());
        vm.startPrank(subnetAddress);
        vm.deal(subnetAddress, releaseAmount);

        registerSubnet(subnetBalance, subnetAddress);

        vm.expectRevert(NotEnoughFundsToRelease.selector);

        vm.startPrank(subnetAddress);
        gatewayDiamond.manager().releaseStake(releaseAmount);
    }

    function testGatewayDiamond_ReleaseStake_Fail_NotRegisteredSubnet() public {
        vm.expectRevert(NotRegisteredSubnet.selector);

        gatewayDiamond.manager().releaseStake(1);
    }

    function testGatewayDiamond_ReleaseStake_Works_TransitionToInactive() public {
        address subnetAddress = address(new SubnetWithNativeTokenMock());

        vm.startPrank(subnetAddress);
        vm.deal(subnetAddress, DEFAULT_COLLATERAL_AMOUNT);

        registerSubnet(DEFAULT_COLLATERAL_AMOUNT, subnetAddress);

        vm.startPrank(subnetAddress);
        gatewayDiamond.manager().releaseStake(10);

        (, uint256 stake, , , ) = getSubnet(subnetAddress);

        require(stake == DEFAULT_COLLATERAL_AMOUNT - 10, "unexpected stake");
    }

    function testGatewayDiamond_Kill_Works() public {
        address subnetAddress = CHILD_NETWORK_ADDRESS;

        vm.startPrank(subnetAddress);
        vm.deal(subnetAddress, DEFAULT_COLLATERAL_AMOUNT);

        registerSubnet(DEFAULT_COLLATERAL_AMOUNT, subnetAddress);

        require(subnetAddress.balance == 0, "unexpected balance");

        vm.startPrank(subnetAddress);
        gatewayDiamond.manager().kill();

        (SubnetID memory id, uint256 stake, uint256 nonce, , uint256 circSupply) = getSubnet(subnetAddress);

        require(id.toHash() == SubnetID(0, new address[](0)).toHash(), "unexpected ID hash");
        require(stake == 0, "unexpected stake");
        require(nonce == 0, "unexpected nonce");
        require(circSupply == 0, "unexpected circSupply");
        require(gatewayDiamond.getter().totalSubnets() == 0, "unexpected total subnets");
        require(subnetAddress.balance == DEFAULT_COLLATERAL_AMOUNT, "unexpected balance");
        require(gatewayDiamond.getter().listSubnets().length == 0, "unexpected number of subnets");
        require(gatewayDiamond.getter().getSubnetKeys().length == 0, "unexpected number of subnet keys");
    }

    function testGatewayDiamond_Kill_Fail_SubnetNotExists() public {
        vm.expectRevert(NotRegisteredSubnet.selector);

        gatewayDiamond.manager().kill();
    }

    function testGatewayDiamond_SendCrossMessage_Fails_Fuzz(uint256 fee) public {
        vm.assume(fee < DEFAULT_CROSS_MSG_FEE);

        address caller = CHILD_NETWORK_ADDRESS;
        vm.deal(caller, DEFAULT_COLLATERAL_AMOUNT + DEFAULT_CROSS_MSG_FEE + 2);
        vm.prank(caller);
        registerSubnet(DEFAULT_COLLATERAL_AMOUNT, caller);

        vm.expectRevert();
        gatewayDiamond.messenger().sendContractXnetMessage{value: fee - 1}(
            TestUtils.newXnetCallMsg(
                IPCAddress({
                    subnetId: SubnetID({root: ROOTNET_CHAINID, route: new address[](0)}),
                    rawAddress: FvmAddressHelper.from(caller)
                }),
                IPCAddress({
                    subnetId: SubnetID({root: 0, route: new address[](0)}),
                    rawAddress: FvmAddressHelper.from(caller)
                }),
                1,
                0
            )
        );
    }

    function testGatewayDiamond_Single_Funding() public {
        (address validatorAddress, , bytes memory publicKey) = TestUtils.newValidator(0);

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

        vm.startPrank(address(saDiamond));
        vm.expectRevert(NotEmptySubnetCircSupply.selector);
        gatewayDiamond.manager().kill();
    }

    function testGatewayDiamond_Fund_Revert_OnZeroValue() public {
        (address validatorAddress, bytes memory publicKey) = TestUtils.deriveValidatorAddress(100);
        join(validatorAddress, publicKey);

        address funderAddress = address(101);

        (SubnetID memory subnetId, , , , ) = getSubnet(address(saDiamond));

        vm.expectRevert(abi.encodeWithSelector(InvalidXnetMessage.selector, InvalidXnetMessageReason.Value));
        gatewayDiamond.manager().fund{value: 0}(subnetId, FvmAddressHelper.from(funderAddress));
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

        (SubnetID memory subnetId, , , , ) = getSubnet(address(saDiamond));
        vm.prank(funderAddress);
        gatewayDiamond.manager().fund{value: amount}(subnetId, FvmAddressHelper.from(msg.sender));
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
        gatewayDiamond.manager().fund{value: fundAmount}(wrongSubnetId, FvmAddressHelper.from(msg.sender));

        vm.expectRevert(NotRegisteredSubnet.selector);
        gatewayDiamond.manager().fund{value: fundAmount}(
            SubnetID(ROOTNET_CHAINID, wrongPath),
            FvmAddressHelper.from(msg.sender)
        );
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
        (address validatorAddress, uint256 privKey, bytes memory publicKey) = TestUtils.newValidator(0);
        assert(validatorAddress == vm.addr(privKey));

        join(validatorAddress, publicKey);

        vm.prank(validatorAddress);
        saDiamond.manager().leave();

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
            majorityPercentage: DEFAULT_MAJORITY_PERCENTAGE,
            genesisValidators: new Validator[](0),
            activeValidatorsLimit: 100,
            commitSha: DEFAULT_COMMIT_SHA
        });
        gatewayDiamond = createGatewayDiamond(constructorParams);

        address callerAddress = address(100);

        vm.startPrank(callerAddress);
        vm.deal(callerAddress, 1 ether);
        vm.expectRevert(abi.encodeWithSelector(InvalidXnetMessage.selector, InvalidXnetMessageReason.Value));

        gatewayDiamond.manager().release{value: 0 ether}(FvmAddressHelper.from(msg.sender));
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
            majorityPercentage: DEFAULT_MAJORITY_PERCENTAGE,
            genesisValidators: new Validator[](0),
            activeValidatorsLimit: 100,
            commitSha: DEFAULT_COMMIT_SHA
        });

        gatewayDiamond = createGatewayDiamond(constructorParams);

        vm.roll(0);
        vm.warp(0);
        vm.startPrank(BLS_ACCOUNT_ADDREESS);
        vm.deal(BLS_ACCOUNT_ADDREESS, releaseAmount + 1);
        release(releaseAmount);
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
            majorityPercentage: DEFAULT_MAJORITY_PERCENTAGE,
            genesisValidators: new Validator[](0),
            activeValidatorsLimit: 100,
            commitSha: DEFAULT_COMMIT_SHA
        });

        gatewayDiamond = createGatewayDiamond(constructorParams);
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
            majorityPercentage: DEFAULT_MAJORITY_PERCENTAGE,
            genesisValidators: new Validator[](0),
            activeValidatorsLimit: 100,
            commitSha: DEFAULT_COMMIT_SHA
        });

        gatewayDiamond = createGatewayDiamond(constructorParams);

        address callerAddress = address(100);

        vm.roll(0);
        vm.warp(0);
        vm.startPrank(callerAddress);
        vm.deal(callerAddress, 2 * releaseAmount + 1);

        release(releaseAmount);
        release(releaseAmount);
    }

    function testGatewayDiamond_SendCrossMessage_Fails_NoDestination() public {
        address caller = address(new MockIpcContract());
        vm.startPrank(caller);
        vm.deal(caller, DEFAULT_COLLATERAL_AMOUNT + DEFAULT_CROSS_MSG_FEE + 2);
        registerSubnet(DEFAULT_COLLATERAL_AMOUNT, caller);

        vm.expectRevert(abi.encodeWithSelector(InvalidXnetMessage.selector, InvalidXnetMessageReason.DstSubnet));
        gatewayDiamond.messenger().sendContractXnetMessage{value: 1}(
            TestUtils.newXnetCallMsg(
                IPCAddress({
                    subnetId: SubnetID({root: ROOTNET_CHAINID, route: new address[](0)}),
                    rawAddress: FvmAddressHelper.from(caller)
                }),
                IPCAddress({
                    subnetId: SubnetID({root: 0, route: new address[](0)}),
                    rawAddress: FvmAddressHelper.from(caller)
                }),
                1,
                0
            )
        );
    }

    function testGatewayDiamond_SendCrossMessage_Fails_NoCurrentNetwork() public {
        address caller = address(new MockIpcContract());
        vm.startPrank(caller);
        vm.deal(caller, DEFAULT_COLLATERAL_AMOUNT + DEFAULT_CROSS_MSG_FEE + 2);
        registerSubnet(DEFAULT_COLLATERAL_AMOUNT, caller);
        SubnetID memory destinationSubnet = gatewayDiamond.getter().getNetworkName();

        vm.expectRevert(abi.encodeWithSelector(InvalidXnetMessage.selector, InvalidXnetMessageReason.ReflexiveSend));
        gatewayDiamond.messenger().sendContractXnetMessage{value: 1}(
            TestUtils.newXnetCallMsg(
                IPCAddress({
                    subnetId: SubnetID({root: ROOTNET_CHAINID, route: new address[](0)}),
                    rawAddress: FvmAddressHelper.from(caller)
                }),
                IPCAddress({subnetId: destinationSubnet, rawAddress: FvmAddressHelper.from(caller)}),
                1,
                0
            )
        );
    }

    // TODO: this is no longer possible because EOA cannot be subnet
    // function testGatewayDiamond_SendCrossMessage_Fails_EoACaller() public {
    //     address caller = vm.addr(100);
    //     vm.startPrank(caller);
    //     vm.deal(caller, DEFAULT_COLLATERAL_AMOUNT + DEFAULT_CROSS_MSG_FEE + 2);

    //     registerSubnet(DEFAULT_COLLATERAL_AMOUNT, caller);

    //     SubnetID memory destinationSubnet = SubnetID(0, new address[](0));
    //     vm.expectRevert(abi.encodeWithSelector(InvalidXnetMessage.selector, InvalidXnetMessageReason.Sender));

    //     gatewayDiamond.messenger().sendContractXnetMessage{value: DEFAULT_CROSS_MSG_FEE}(
    //         TestUtils.newXnetCallMsg(
    //             IPCAddress({
    //                 subnetId: SubnetID({root: ROOTNET_CHAINID, route: new address[](0)}),
    //                 rawAddress: FvmAddressHelper.from(caller)
    //             }),
    //             IPCAddress({subnetId: destinationSubnet, rawAddress: FvmAddressHelper.from(caller)}),
    //             1,
    //             0
    //         )
    //     );
    // }

    function testGatewayDiamond_SendCrossMessage_Fails_EmptyNetwork() public {
        // Caller of general-purpose messages must be a contract, not a EoA
        address caller = address(new MockIpcContract());
        vm.startPrank(caller);
        vm.deal(caller, DEFAULT_COLLATERAL_AMOUNT + DEFAULT_CROSS_MSG_FEE + 2);

        registerSubnet(DEFAULT_COLLATERAL_AMOUNT, caller);

        SubnetID memory destinationSubnet = SubnetID(0, new address[](0));

        vm.expectRevert(abi.encodeWithSelector(InvalidXnetMessage.selector, InvalidXnetMessageReason.DstSubnet));

        gatewayDiamond.messenger().sendContractXnetMessage{value: 1}(
            TestUtils.newXnetCallMsg(
                IPCAddress({
                    subnetId: SubnetID({root: ROOTNET_CHAINID, route: new address[](0)}),
                    rawAddress: FvmAddressHelper.from(caller)
                }),
                IPCAddress({subnetId: destinationSubnet, rawAddress: FvmAddressHelper.from(caller)}),
                1,
                0
            )
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

        gatewayDiamond.topDownFinalizer().commitParentFinality(finality);
    }

    function testGatewayDiamond_applyFinality_works() public {
        // changes included for two validators joining
        address val1 = vm.addr(100);
        address val2 = vm.addr(101);
        uint256 amount = 10000;
        PowerChangeRequest[] memory changes = new PowerChangeRequest[](2);

        changes[0] = PowerChangeRequest({
            configurationNumber: 1,
            change: PowerChange({validator: val1, op: PowerOperation.SetPower, payload: abi.encode(amount)})
        });
        changes[1] = PowerChangeRequest({
            configurationNumber: 2,
            change: PowerChange({validator: val2, op: PowerOperation.SetPower, payload: abi.encode(amount)})
        });

        vm.startPrank(FilAddress.SYSTEM_ACTOR);

        gatewayDiamond.topDownFinalizer().storeValidatorChanges(changes);
        uint64 configNumber = gatewayDiamond.topDownFinalizer().applyFinalityChanges();
        require(configNumber == 2, "wrong config number after applying finality");
        require(
            gatewayDiamond.getter().getCurrentMembership().validators.length == 2,
            "current membership should be 2"
        );
        require(gatewayDiamond.getter().getCurrentConfigurationNumber() == 2, "unexpected config number");
        require(gatewayDiamond.getter().getLastConfigurationNumber() == 0, "unexpected last config number");

        vm.stopPrank();

        // new change with a validator leaving
        changes = new PowerChangeRequest[](1);

        changes[0] = PowerChangeRequest({
            configurationNumber: 3,
            change: PowerChange({validator: val1, op: PowerOperation.SetPower, payload: abi.encode(0)})
        });

        vm.startPrank(FilAddress.SYSTEM_ACTOR);

        gatewayDiamond.topDownFinalizer().storeValidatorChanges(changes);
        configNumber = gatewayDiamond.topDownFinalizer().applyFinalityChanges();
        require(configNumber == 3, "wrong config number after applying finality");
        require(
            gatewayDiamond.getter().getLastConfigurationNumber() == 2,
            "apply result: unexpected last config number"
        );
        require(gatewayDiamond.getter().getCurrentConfigurationNumber() == 3, "apply result: unexpected config number");
        require(
            gatewayDiamond.getter().getCurrentMembership().validators.length == 1,
            "current membership should be 1"
        );
        require(gatewayDiamond.getter().getLastMembership().validators.length == 2, "last membership should be 2");

        // no changes
        configNumber = gatewayDiamond.topDownFinalizer().applyFinalityChanges();
        require(configNumber == 0, "wrong config number after applying finality");
        require(gatewayDiamond.getter().getLastConfigurationNumber() == 2, "no changes: unexpected last config number");
        require(gatewayDiamond.getter().getCurrentConfigurationNumber() == 3, "no changes: unexpected config number");
        require(
            gatewayDiamond.getter().getCurrentMembership().validators.length == 1,
            "current membership should be 1"
        );
        require(gatewayDiamond.getter().getLastMembership().validators.length == 2, "last membership should be 2");

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
        // increase the block number so that current block number is
        // not the same as init committed parent finality height
        vm.roll(10);

        ParentFinality memory finality = ParentFinality({height: block.number, blockHash: bytes32(0)});

        gatewayDiamond.topDownFinalizer().commitParentFinality(finality);
        ParentFinality memory committedFinality = gatewayDiamond.getter().getParentFinality(block.number);

        require(committedFinality.height == finality.height, "heights are not equal");
        require(committedFinality.blockHash == finality.blockHash, "blockHash is not equal");
        require(gatewayDiamond.getter().getLatestParentFinality().height == block.number, "finality height not equal");

        vm.stopPrank();
    }

    function testGatewayDiamond_createBottomUpCheckpoint() public {
        (, address[] memory addrs, uint256[] memory weights) = TestUtils.getFourValidators(vm);

        (bytes32 membershipRoot, ) = MerkleTreeHelper.createMerkleProofsForValidators(addrs, weights);

        BottomUpCheckpoint memory old = BottomUpCheckpoint({
            subnetID: gatewayDiamond.getter().getNetworkName(),
            blockHeight: 0,
            blockHash: keccak256("block1"),
            nextConfigurationNumber: 1,
            msgs: BottomUpBatchHelper.makeCommitment(BottomUpBatchHelper.makeEmptyBatch()),
            activity: ActivityHelper.newCompressedActivityRollup(1, 3, bytes32(uint256(0)))
        });

        BottomUpCheckpoint memory checkpoint = BottomUpCheckpoint({
            subnetID: gatewayDiamond.getter().getNetworkName(),
            blockHeight: gatewayDiamond.getter().bottomUpCheckPeriod(),
            blockHash: keccak256("block1"),
            nextConfigurationNumber: 1,
            msgs: BottomUpBatchHelper.makeCommitment(BottomUpBatchHelper.makeEmptyBatch()),
            activity: ActivityHelper.newCompressedActivityRollup(1, 3, bytes32(uint256(0)))
        });

        // failed to create a checkpoint with zero membership weight
        vm.startPrank(FilAddress.SYSTEM_ACTOR);
        vm.expectRevert(ZeroMembershipWeight.selector);
        gatewayDiamond.checkpointer().createBottomUpCheckpoint(
            checkpoint,
            membershipRoot,
            0,
            BottomUpBatchHelper.makeEmptyBatch(),
            ActivityHelper.dummyActivityRollup()
        );
        vm.stopPrank();

        // failed create a processed checkpoint
        vm.startPrank(FilAddress.SYSTEM_ACTOR);
        vm.expectRevert(QuorumAlreadyProcessed.selector);
        gatewayDiamond.checkpointer().createBottomUpCheckpoint(
            old,
            membershipRoot,
            weights[0] + weights[1] + weights[2],
            BottomUpBatchHelper.makeEmptyBatch(),
            ActivityHelper.dummyActivityRollup()
        );
        vm.stopPrank();

        // create a checkpoint
        vm.startPrank(FilAddress.SYSTEM_ACTOR);
        gatewayDiamond.checkpointer().createBottomUpCheckpoint(
            checkpoint,
            membershipRoot,
            weights[0] + weights[1] + weights[2],
            BottomUpBatchHelper.makeEmptyBatch(),
            ActivityHelper.dummyActivityRollup()
        );
        vm.stopPrank();

        BottomUpCheckpoint memory recv = gatewayDiamond.getter().bottomUpCheckpoint(
            gatewayDiamond.getter().bottomUpCheckPeriod()
        );
        require(recv.nextConfigurationNumber == 1, "nextConfigurationNumber incorrect");
        require(recv.blockHash == keccak256("block1"), "block hash incorrect");
        uint256 d = gatewayDiamond.getter().bottomUpCheckPeriod();

        // failed to create a checkpoint with the same height
        checkpoint = BottomUpCheckpoint({
            subnetID: gatewayDiamond.getter().getNetworkName(),
            blockHeight: d,
            blockHash: keccak256("block"),
            nextConfigurationNumber: 2,
            msgs: BottomUpBatchHelper.makeCommitment(BottomUpBatchHelper.makeEmptyBatch()),
            activity: ActivityHelper.newCompressedActivityRollup(1, 3, bytes32(uint256(0)))
        });

        vm.startPrank(FilAddress.SYSTEM_ACTOR);
        vm.expectRevert(CheckpointAlreadyExists.selector);
        gatewayDiamond.checkpointer().createBottomUpCheckpoint(
            checkpoint,
            membershipRoot,
            weights[0] + weights[1] + weights[2],
            BottomUpBatchHelper.makeEmptyBatch(),
            ActivityHelper.dummyActivityRollup()
        );
        vm.stopPrank();

        (bool ok, uint256 e, ) = gatewayDiamond.getter().getCurrentBottomUpCheckpoint();
        require(ok, "checkpoint not exist");
        require(e == d, "out height incorrect");
    }

    function testGatewayDiamond_commitBottomUpCheckpoint_InvalidCheckpointSource() public {
        BottomUpCheckpoint memory checkpoint = BottomUpCheckpoint({
            subnetID: gatewayDiamond.getter().getNetworkName(),
            blockHeight: gatewayDiamond.getter().bottomUpCheckPeriod(),
            blockHash: keccak256("block1"),
            nextConfigurationNumber: 1,
            msgs: BottomUpBatchHelper.makeCommitment(BottomUpBatchHelper.makeEmptyBatch()),
            activity: ActivityHelper.newCompressedActivityRollup(1, 3, bytes32(uint256(0)))
        });

        vm.expectRevert(InvalidCheckpointSource.selector);
        gatewayDiamond.checkpointer().commitCheckpoint(checkpoint);
    }

    function testGatewayDiamond_commitBottomUpCheckpoint_Works_NoMessages() public {
        address caller = address(saDiamond);
        vm.startPrank(caller);
        vm.deal(caller, DEFAULT_COLLATERAL_AMOUNT + DEFAULT_CROSS_MSG_FEE);
        registerSubnet(DEFAULT_COLLATERAL_AMOUNT, caller);
        vm.stopPrank();

        (SubnetID memory subnetId, , , , ) = getSubnet(address(caller));

        BottomUpCheckpoint memory checkpoint = BottomUpCheckpoint({
            subnetID: subnetId,
            blockHeight: gatewayDiamond.getter().bottomUpCheckPeriod(),
            blockHash: keccak256("block1"),
            nextConfigurationNumber: 1,
            msgs: BottomUpBatchHelper.makeCommitment(BottomUpBatchHelper.makeEmptyBatch()),
            activity: ActivityHelper.newCompressedActivityRollup(1, 3, bytes32(uint256(0)))
        });

        vm.prank(caller);
        gatewayDiamond.checkpointer().commitCheckpoint(checkpoint);
    }

    function testGatewayDiamond_commitBottomUpCheckpoint_Works_WithMessages() public {
        address caller = address(saDiamond);
        address recipient = address(new MockIpcContractPayable());

        vm.startPrank(caller);
        vm.deal(caller, DEFAULT_COLLATERAL_AMOUNT + DEFAULT_CROSS_MSG_FEE);
        registerSubnet(DEFAULT_COLLATERAL_AMOUNT, caller);
        vm.stopPrank();

        uint256 amount = 1;

        (SubnetID memory subnetId, , , , ) = getSubnet(address(caller));
        (bool exist, Subnet memory subnetInfo) = gatewayDiamond.getter().getSubnet(subnetId);
        require(exist, "subnet does not exist");
        require(subnetInfo.circSupply == 0, "unexpected initial circulation supply");

        gatewayDiamond.manager().fund{value: DEFAULT_COLLATERAL_AMOUNT}(
            subnetId,
            FvmAddressHelper.from(address(caller))
        );
        (, subnetInfo) = gatewayDiamond.getter().getSubnet(subnetId);
        require(subnetInfo.circSupply == DEFAULT_COLLATERAL_AMOUNT, "unexpected circulation supply after funding");

        uint256 totalAmount = 0;
        IpcEnvelope[] memory msgs = new IpcEnvelope[](10);
        for (uint64 i = 0; i < 10; i++) {
            msgs[i] = TestUtils.newXnetCallMsg(
                IPCAddress({subnetId: subnetId, rawAddress: FvmAddressHelper.from(caller)}),
                IPCAddress({
                    subnetId: gatewayDiamond.getter().getNetworkName(),
                    rawAddress: FvmAddressHelper.from(recipient)
                }),
                amount,
                i
            );

            totalAmount += amount;
        }

        BottomUpCheckpoint memory checkpoint = BottomUpCheckpoint({
            subnetID: subnetId,
            blockHeight: gatewayDiamond.getter().bottomUpCheckPeriod(),
            blockHash: keccak256("block1"),
            nextConfigurationNumber: 1,
            msgs: BottomUpBatchHelper.makeCommitment(msgs),
            activity: ActivityHelper.newCompressedActivityRollup(1, 3, bytes32(uint256(0)))
        });

        vm.prank(caller);
        gatewayDiamond.checkpointer().commitCheckpoint(checkpoint);
        vm.prank(caller);
        gatewayDiamond.checkpointer().execBottomUpMsgBatch(msgs);

        (, subnetInfo) = gatewayDiamond.getter().getSubnet(subnetId);
        require(subnetInfo.circSupply == DEFAULT_COLLATERAL_AMOUNT - totalAmount, "unexpected circulating supply");
    }

    function testGatewayDiamond_listIncompleteCheckpoints() public {
        (, address[] memory addrs, uint256[] memory weights) = TestUtils.getFourValidators(vm);

        (bytes32 membershipRoot, ) = MerkleTreeHelper.createMerkleProofsForValidators(addrs, weights);

        BottomUpCheckpoint memory checkpoint1 = BottomUpCheckpoint({
            subnetID: gatewayDiamond.getter().getNetworkName(),
            blockHeight: gatewayDiamond.getter().bottomUpCheckPeriod(),
            blockHash: keccak256("block1"),
            nextConfigurationNumber: 1,
            msgs: BottomUpBatchHelper.makeCommitment(BottomUpBatchHelper.makeEmptyBatch()),
            activity: ActivityHelper.newCompressedActivityRollup(1, 3, bytes32(uint256(0)))
        });

        BottomUpCheckpoint memory checkpoint2 = BottomUpCheckpoint({
            subnetID: gatewayDiamond.getter().getNetworkName(),
            blockHeight: 2 * gatewayDiamond.getter().bottomUpCheckPeriod(),
            blockHash: keccak256("block2"),
            nextConfigurationNumber: 1,
            msgs: BottomUpBatchHelper.makeCommitment(BottomUpBatchHelper.makeEmptyBatch()),
            activity: ActivityHelper.newCompressedActivityRollup(1, 3, bytes32(uint256(0)))
        });

        // create a checkpoint
        vm.startPrank(FilAddress.SYSTEM_ACTOR);
        gatewayDiamond.checkpointer().createBottomUpCheckpoint(
            checkpoint1,
            membershipRoot,
            weights[0] + weights[1] + weights[2],
            BottomUpBatchHelper.makeEmptyBatch(),
            ActivityHelper.dummyActivityRollup()
        );
        gatewayDiamond.checkpointer().createBottomUpCheckpoint(
            checkpoint2,
            membershipRoot,
            weights[0] + weights[1] + weights[2],
            BottomUpBatchHelper.makeEmptyBatch(),
            ActivityHelper.dummyActivityRollup()
        );
        vm.stopPrank();

        uint256[] memory heights = gatewayDiamond.getter().getIncompleteCheckpointHeights();

        require(heights.length == 2, "unexpected heights");
        require(heights[0] == gatewayDiamond.getter().bottomUpCheckPeriod(), "heights[0] == period");
        require(heights[1] == 2 * gatewayDiamond.getter().bottomUpCheckPeriod(), "heights[1] == 2*period");

        QuorumInfo memory info = gatewayDiamond.getter().getCheckpointInfo(
            gatewayDiamond.getter().bottomUpCheckPeriod()
        );
        require(info.rootHash == membershipRoot, "info.rootHash == membershipRoot");
        require(
            info.threshold == gatewayDiamond.getter().getQuorumThreshold(weights[0] + weights[1] + weights[2]),
            "checkpoint 1 correct threshold"
        );

        info = gatewayDiamond.getter().getCheckpointInfo(2 * gatewayDiamond.getter().bottomUpCheckPeriod());
        require(info.rootHash == membershipRoot, "info.rootHash == membershipRoot");
        require(
            info.threshold == gatewayDiamond.getter().getQuorumThreshold(weights[0] + weights[1] + weights[2]),
            "checkpoint 2 correct threshold"
        );

        BottomUpCheckpoint[] memory incomplete = gatewayDiamond.getter().getIncompleteCheckpoints();
        require(incomplete.length == 2, "incomplete.length == 2");
        require(
            incomplete[0].blockHeight == gatewayDiamond.getter().bottomUpCheckPeriod(),
            "incomplete[0].blockHeight"
        );
        require(incomplete[0].blockHash == keccak256("block1"), "incomplete[0].blockHash");
        require(
            incomplete[1].blockHeight == 2 * gatewayDiamond.getter().bottomUpCheckPeriod(),
            "incomplete[1].blockHeight"
        );
        require(incomplete[1].blockHash == keccak256("block2"), "incomplete[1].blockHash");
    }

    function testGatewayDiamond_addCheckpointSignature_newCheckpoint() public {
        (uint256[] memory privKeys, address[] memory addrs, uint256[] memory weights) = TestUtils.getFourValidators(vm);

        (bytes32 membershipRoot, bytes32[][] memory membershipProofs) = MerkleTreeHelper
            .createMerkleProofsForValidators(addrs, weights);

        BottomUpCheckpoint memory checkpoint = BottomUpCheckpoint({
            subnetID: gatewayDiamond.getter().getNetworkName(),
            blockHeight: gatewayDiamond.getter().bottomUpCheckPeriod(),
            blockHash: keccak256("block"),
            nextConfigurationNumber: 1,
            msgs: BottomUpBatchHelper.makeCommitment(BottomUpBatchHelper.makeEmptyBatch()),
            activity: ActivityHelper.newCompressedActivityRollup(1, 3, bytes32(uint256(0)))
        });

        // create a checkpoint
        vm.startPrank(FilAddress.SYSTEM_ACTOR);
        gatewayDiamond.checkpointer().createBottomUpCheckpoint(
            checkpoint,
            membershipRoot,
            weights[0] + weights[1] + weights[2],
            BottomUpBatchHelper.makeEmptyBatch(),
            ActivityHelper.dummyActivityRollup()
        );
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
            gatewayDiamond.checkpointer().addCheckpointSignature(
                checkpoint.blockHeight,
                membershipProofs[i],
                weights[i],
                signature
            );
            vm.stopPrank();
        }

        require(
            gatewayDiamond.getter().getCheckpointCurrentWeight(checkpoint.blockHeight) == totalWeight(weights),
            "checkpoint weight was not updated"
        );

        (
            BottomUpCheckpoint memory ch,
            QuorumInfo memory info,
            address[] memory signatories,
            bytes[] memory signatures
        ) = gatewayDiamond.getter().getCheckpointSignatureBundle(gatewayDiamond.getter().bottomUpCheckPeriod());
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
            subnetID: gatewayDiamond.getter().getNetworkName(),
            blockHeight: gatewayDiamond.getter().bottomUpCheckPeriod(),
            blockHash: keccak256("block"),
            nextConfigurationNumber: 1,
            msgs: BottomUpBatchHelper.makeCommitment(BottomUpBatchHelper.makeEmptyBatch()),
            activity: ActivityHelper.newCompressedActivityRollup(1, 3, bytes32(uint256(0)))
        });

        // create a checkpoint
        vm.startPrank(FilAddress.SYSTEM_ACTOR);
        gatewayDiamond.checkpointer().createBottomUpCheckpoint(
            checkpoint,
            membershipRoot,
            weights[0] + weights[1] + weights[2],
            BottomUpBatchHelper.makeEmptyBatch(),
            ActivityHelper.dummyActivityRollup()
        );
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
            gatewayDiamond.checkpointer().addCheckpointSignature(
                checkpoint.blockHeight,
                membershipProofs[i],
                weights[i],
                signature
            );
            vm.stopPrank();
        }

        QuorumInfo memory info = gatewayDiamond.getter().getCheckpointInfo(1);
        require(!info.reached, "not reached");
        require(gatewayDiamond.getter().getIncompleteCheckpointHeights().length == 1, "unexpected size");

        info = gatewayDiamond.getter().getCheckpointInfo(1);

        (v, r, s) = vm.sign(privKeys[2], keccak256(abi.encode(checkpoint)));
        signature = abi.encodePacked(r, s, v);

        vm.startPrank(vm.addr(privKeys[2]));
        gatewayDiamond.checkpointer().addCheckpointSignature(
            checkpoint.blockHeight,
            membershipProofs[2],
            weights[2],
            signature
        );
        vm.stopPrank();

        info = gatewayDiamond.getter().getCheckpointInfo(checkpoint.blockHeight);
        require(info.reached, "not reached");
        require(gatewayDiamond.getter().getIncompleteCheckpointHeights().length == 0, "unexpected size");

        require(
            gatewayDiamond.getter().getCheckpointCurrentWeight(checkpoint.blockHeight) == totalWeight(weights),
            "checkpoint weight was not updated"
        );
        (v, r, s) = vm.sign(privKeys[3], keccak256(abi.encode(checkpoint)));
        signature = abi.encodePacked(r, s, v);

        vm.startPrank(vm.addr(privKeys[3]));
        gatewayDiamond.checkpointer().addCheckpointSignature(
            checkpoint.blockHeight,
            membershipProofs[3],
            weights[3],
            signature
        );
        vm.stopPrank();
    }

    function testGatewayDiamond_addCheckpointSignature_notAuthorized() public {
        (uint256[] memory privKeys, address[] memory addrs, uint256[] memory weights) = TestUtils.getFourValidators(vm);

        (bytes32 membershipRoot, bytes32[][] memory membershipProofs) = MerkleTreeHelper
            .createMerkleProofsForValidators(addrs, weights);

        BottomUpCheckpoint memory checkpoint = BottomUpCheckpoint({
            subnetID: gatewayDiamond.getter().getNetworkName(),
            blockHeight: gatewayDiamond.getter().bottomUpCheckPeriod(),
            blockHash: keccak256("block"),
            nextConfigurationNumber: 1,
            msgs: BottomUpBatchHelper.makeCommitment(BottomUpBatchHelper.makeEmptyBatch()),
            activity: ActivityHelper.newCompressedActivityRollup(1, 3, bytes32(uint256(0)))
        });

        // create a checkpoint
        vm.startPrank(FilAddress.SYSTEM_ACTOR);
        gatewayDiamond.checkpointer().createBottomUpCheckpoint(
            checkpoint,
            membershipRoot,
            10,
            BottomUpBatchHelper.makeEmptyBatch(),
            ActivityHelper.dummyActivityRollup()
        );
        vm.stopPrank();

        uint8 v;
        bytes32 r;
        bytes32 s;
        bytes memory signature;

        (v, r, s) = vm.sign(privKeys[0], keccak256(abi.encode(checkpoint)));
        signature = abi.encodePacked(r, s, v);

        uint256 h = gatewayDiamond.getter().bottomUpCheckPeriod();
        vm.startPrank(vm.addr(privKeys[1]));
        vm.expectRevert(abi.encodeWithSelector(NotAuthorized.selector, vm.addr(privKeys[0])));
        gatewayDiamond.checkpointer().addCheckpointSignature(h, membershipProofs[2], weights[2], signature);
        vm.stopPrank();
    }

    function testGatewayDiamond_addCheckpointSignature_invalidSignature_replayedSignature() public {
        (uint256[] memory privKeys, address[] memory addrs, uint256[] memory weights) = TestUtils.getFourValidators(vm);

        (bytes32 membershipRoot, bytes32[][] memory membershipProofs) = MerkleTreeHelper
            .createMerkleProofsForValidators(addrs, weights);

        BottomUpCheckpoint memory checkpoint = BottomUpCheckpoint({
            subnetID: gatewayDiamond.getter().getNetworkName(),
            blockHeight: gatewayDiamond.getter().bottomUpCheckPeriod(),
            blockHash: keccak256("block"),
            nextConfigurationNumber: 1,
            msgs: BottomUpBatchHelper.makeCommitment(BottomUpBatchHelper.makeEmptyBatch()),
            activity: ActivityHelper.newCompressedActivityRollup(1, 3, bytes32(uint256(0)))
        });

        // create a checkpoint
        vm.startPrank(FilAddress.SYSTEM_ACTOR);
        gatewayDiamond.checkpointer().createBottomUpCheckpoint(
            checkpoint,
            membershipRoot,
            10,
            BottomUpBatchHelper.makeEmptyBatch(),
            ActivityHelper.dummyActivityRollup()
        );
        vm.stopPrank();

        uint8 v;
        bytes32 r;
        bytes32 s;
        bytes memory signature;

        (v, r, s) = vm.sign(privKeys[0], keccak256(abi.encode(checkpoint)));
        signature = abi.encodePacked(r, s, v);

        uint256 h = gatewayDiamond.getter().bottomUpCheckPeriod();
        vm.startPrank(vm.addr(privKeys[0]));

        // send incorrect signature
        vm.expectRevert(InvalidSignature.selector);
        gatewayDiamond.checkpointer().addCheckpointSignature(h, membershipProofs[0], weights[0], new bytes(0));

        // send correct signature
        gatewayDiamond.checkpointer().addCheckpointSignature(h, membershipProofs[0], weights[0], signature);

        // replay the previous signature
        vm.expectRevert(SignatureReplay.selector);
        gatewayDiamond.checkpointer().addCheckpointSignature(h, membershipProofs[0], weights[0], signature);

        vm.stopPrank();
    }

    function testGatewayDiamond_addCheckpointSignature_incorrectCheckpoint() public {
        (uint256[] memory privKeys, address[] memory addrs, uint256[] memory weights) = TestUtils.getFourValidators(vm);

        (bytes32 membershipRoot, bytes32[][] memory membershipProofs) = MerkleTreeHelper
            .createMerkleProofsForValidators(addrs, weights);

        BottomUpCheckpoint memory checkpoint = BottomUpCheckpoint({
            subnetID: gatewayDiamond.getter().getNetworkName(),
            blockHeight: gatewayDiamond.getter().bottomUpCheckPeriod(),
            blockHash: keccak256("block"),
            nextConfigurationNumber: 1,
            msgs: BottomUpBatchHelper.makeCommitment(BottomUpBatchHelper.makeEmptyBatch()),
            activity: ActivityHelper.newCompressedActivityRollup(1, 3, bytes32(uint256(0)))
        });

        // create a checkpoint
        vm.startPrank(FilAddress.SYSTEM_ACTOR);
        gatewayDiamond.checkpointer().createBottomUpCheckpoint(
            checkpoint,
            membershipRoot,
            10,
            BottomUpBatchHelper.makeEmptyBatch(),
            ActivityHelper.dummyActivityRollup()
        );
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
        gatewayDiamond.checkpointer().addCheckpointSignature(0, membershipProofs[0], weights[0], signature);

        // send correct signature for incorrect height
        vm.expectRevert(CheckpointNotCreated.selector);
        gatewayDiamond.checkpointer().addCheckpointSignature(100, membershipProofs[0], weights[0], signature);

        vm.stopPrank();
    }

    function testGatewayDiamond_garbage_collect_bottomUpCheckpoints() public {
        (, address[] memory addrs, uint256[] memory weights) = TestUtils.getFourValidators(vm);

        (bytes32 membershipRoot, ) = MerkleTreeHelper.createMerkleProofsForValidators(addrs, weights);

        uint256 index = gatewayDiamond.getter().getCheckpointRetentionHeight();
        require(index == 1, "unexpected height");

        BottomUpCheckpoint memory checkpoint;

        // create a checkpoint
        uint64 n = 10;
        vm.startPrank(FilAddress.SYSTEM_ACTOR);
        for (uint64 i = 1; i <= n; i++) {
            checkpoint = BottomUpCheckpoint({
                subnetID: gatewayDiamond.getter().getNetworkName(),
                blockHeight: i * gatewayDiamond.getter().bottomUpCheckPeriod(),
                blockHash: keccak256("block"),
                nextConfigurationNumber: 1,
                msgs: BottomUpBatchHelper.makeCommitment(BottomUpBatchHelper.makeEmptyBatch()),
                activity: ActivityHelper.newCompressedActivityRollup(1, 3, bytes32(uint256(0)))
            });

            gatewayDiamond.checkpointer().createBottomUpCheckpoint(
                checkpoint,
                membershipRoot,
                10,
                BottomUpBatchHelper.makeEmptyBatch(),
                ActivityHelper.dummyActivityRollup()
            );
        }
        vm.stopPrank();

        index = gatewayDiamond.getter().getCheckpointRetentionHeight();
        require(index == 1, "retention height is not 1");

        uint256[] memory heights = gatewayDiamond.getter().getIncompleteCheckpointHeights();
        require(heights.length == n, "heights.len is not n");
    }

    function testGatewayDiamond_commitCheckpoint_Fails_WrongNumberMessages() public {
        address caller = address(saDiamond);
        vm.startPrank(caller);
        vm.deal(caller, DEFAULT_COLLATERAL_AMOUNT + DEFAULT_CROSS_MSG_FEE);
        registerSubnet(DEFAULT_COLLATERAL_AMOUNT, caller);
        vm.stopPrank();

        (SubnetID memory subnetId, , , , ) = getSubnet(address(caller));
        (bool exist, Subnet memory subnetInfo) = gatewayDiamond.getter().getSubnet(subnetId);
        require(exist, "subnet does not exist");
        require(subnetInfo.circSupply == 0, "unexpected initial circulation supply");

        gatewayDiamond.manager().fund{value: DEFAULT_COLLATERAL_AMOUNT}(
            subnetId,
            FvmAddressHelper.from(address(caller))
        );
        (, subnetInfo) = gatewayDiamond.getter().getSubnet(subnetId);
        require(subnetInfo.circSupply == DEFAULT_COLLATERAL_AMOUNT, "unexpected circulation supply after funding");
    }

    function newListOfMessages(uint64 size) internal view returns (IpcEnvelope[] memory msgs) {
        msgs = new IpcEnvelope[](size);
        for (uint64 i = 0; i < size; i++) {
            msgs[i] = TestUtils.newXnetCallMsg(
                IPCAddress({
                    subnetId: gatewayDiamond.getter().getNetworkName(),
                    rawAddress: FvmAddressHelper.from(address(this))
                }),
                IPCAddress({
                    subnetId: gatewayDiamond.getter().getNetworkName(),
                    rawAddress: FvmAddressHelper.from(address(this))
                }),
                0,
                i
                // method: this.callback.selector,
            );
        }
    }

    function callback() public view {}
}
