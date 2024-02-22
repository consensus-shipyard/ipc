// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity ^0.8.23;

import "forge-std/Test.sol";
import "../../src/errors/IPCErrors.sol";
import {EMPTY_BYTES, METHOD_SEND} from "../../src/constants/Constants.sol";
import {IpcEnvelope, BottomUpMsgBatch, BottomUpCheckpoint, ParentFinality, IpcMsgKind, OutcomeType} from "../../src/structs/CrossNet.sol";
import {FvmAddress} from "../../src/structs/FvmAddress.sol";
import {SubnetID, Subnet, IPCAddress, Validator} from "../../src/structs/Subnet.sol";
import {SubnetIDHelper} from "../../src/lib/SubnetIDHelper.sol";
import {FvmAddressHelper} from "../../src/lib/FvmAddressHelper.sol";
import {CrossMsgHelper} from "../../src/lib/CrossMsgHelper.sol";
import {GatewayDiamond, FEATURE_MULTILEVEL_CROSSMSG} from "../../src/GatewayDiamond.sol";
import {SubnetActorDiamond} from "../../src/SubnetActorDiamond.sol";
import {SubnetActorGetterFacet} from "../../src/subnet/SubnetActorGetterFacet.sol";
import {SubnetActorManagerFacet} from "../../src/subnet/SubnetActorManagerFacet.sol";
import {SubnetActorCheckpointingFacet} from "../../src/subnet/SubnetActorCheckpointingFacet.sol";
import {GatewayGetterFacet} from "../../src/gateway/GatewayGetterFacet.sol";
import {GatewayManagerFacet} from "../../src/gateway/GatewayManagerFacet.sol";
import {LibGateway} from "../../src/lib/LibGateway.sol";
import {TopDownFinalityFacet} from "../../src/gateway/router/TopDownFinalityFacet.sol";
import {CheckpointingFacet} from "../../src/gateway/router/CheckpointingFacet.sol";
import {XnetMessagingFacet} from "../../src/gateway/router/XnetMessagingFacet.sol";
import {DiamondCutFacet} from "../../src/diamond/DiamondCutFacet.sol";
import {GatewayMessengerFacet} from "../../src/gateway/GatewayMessengerFacet.sol";
import {DiamondLoupeFacet} from "../../src/diamond/DiamondLoupeFacet.sol";
import {DiamondCutFacet} from "../../src/diamond/DiamondCutFacet.sol";
import {IntegrationTestBase} from "../IntegrationTestBase.sol";
import {L2GatewayActorDiamond, L1GatewayActorDiamond} from "../IntegrationTestPresets.sol";
import {TestUtils, MockIpcContract, MockIpcContractPayable, MockIpcContractFallback} from "../helpers/TestUtils.sol";
import {FilAddress} from "fevmate/utils/FilAddress.sol";
import {MerkleTreeHelper} from "../helpers/MerkleTreeHelper.sol";

import {IERC20} from "openzeppelin-contracts/token/ERC20/IERC20.sol";
import {ERC20PresetFixedSupply} from "../helpers/ERC20PresetFixedSupply.sol";
import {IERC20Errors} from "openzeppelin-contracts/interfaces/draft-IERC6093.sol";

import "forge-std/console.sol";

contract MultiSubnetTest is Test, IntegrationTestBase {
    using SubnetIDHelper for SubnetID;
    using CrossMsgHelper for IpcEnvelope;

    GatewayDiamond public rootGateway;
    GatewayGetterFacet public rootGatewayGetter;
    GatewayManagerFacet public rootGatewayManager;

    SubnetActorDiamond public rootNativeSubnetActor;
    SubnetActorDiamond public rootTokenSubnetActor;

    GatewayDiamond public tokenSubnetGateway;
    GatewayDiamond public nativeSubnetGateway;

    address[] public nativeSubnetPath;
    address[] public tokenSubnetPath;

    SubnetID rootSubnetName;
    SubnetID nativeSubnetName;
    SubnetID tokenSubnetName;

    IERC20 public token;

    function setUp() public override {
        token = new ERC20PresetFixedSupply("TestToken", "TEST", 1_000_000, address(this));

        rootSubnetName = SubnetID({root: ROOTNET_CHAINID, route: new address[](0)});
        require(rootSubnetName.isRoot(), "not root");

        rootGateway = createGatewayDiamond(gatewayParams(rootSubnetName));
        rootGatewayGetter = GatewayGetterFacet(address(rootGateway));
        rootGatewayManager = GatewayManagerFacet(address(rootGateway));

        rootNativeSubnetActor = createSubnetActor(subnetActorWithParams(address(rootGateway), rootSubnetName));

        rootTokenSubnetActor = createSubnetActor(
            subnetActorWithParams(address(rootGateway), rootSubnetName, address(token))
        );

        tokenSubnetPath = new address[](1);
        tokenSubnetPath[0] = address(rootTokenSubnetActor);
        tokenSubnetName = SubnetID({root: ROOTNET_CHAINID, route: tokenSubnetPath});
        tokenSubnetGateway = createGatewayDiamond(gatewayParams(tokenSubnetName));

        nativeSubnetPath = new address[](1);
        nativeSubnetPath[0] = address(rootNativeSubnetActor);
        nativeSubnetName = SubnetID({root: ROOTNET_CHAINID, route: nativeSubnetPath});
        nativeSubnetGateway = createGatewayDiamond(gatewayParams(nativeSubnetName));

        printActors();
    }

    //--------------------
    // Fund flow tests.
    //---------------------

    function testMultiSubnet_Native_FundingFromParentToChild() public {
        address caller = address(new MockIpcContract());
        address recipient = address(new MockIpcContractPayable());
        uint256 amount = 3;

        vm.deal(address(rootNativeSubnetActor), DEFAULT_COLLATERAL_AMOUNT);
        vm.deal(caller, amount);

        vm.prank(address(rootNativeSubnetActor));
        registerSubnetGW(DEFAULT_COLLATERAL_AMOUNT, address(rootNativeSubnetActor), rootGateway);

        IpcEnvelope memory expected = CrossMsgHelper.createFundMsg(
            nativeSubnetName,
            caller,
            FvmAddressHelper.from(recipient),
            amount
        );

        vm.prank(caller);
        vm.expectEmit(true, true, true, true, address(rootGateway));
        emit LibGateway.NewTopDownMessage(address(rootNativeSubnetActor), expected);
        rootGatewayManager.fund{value: amount}(nativeSubnetName, FvmAddressHelper.from(address(recipient)));

        IpcEnvelope[] memory msgs = new IpcEnvelope[](1);
        msgs[0] = expected;

        // TODO: commitParentFinality doesn't not affect anything in this test.
        commitParentFinality(address(nativeSubnetGateway));

        executeTopDownMsgs(msgs, nativeSubnetName, address(nativeSubnetGateway));

        assertEq(recipient.balance, amount);
    }

    // A bottom up receipt sending from parent to child. The original message is a
    // bottom up release message, but the execution worked in the parent, creating
    // a topdown result message from parent to child
    function testMultiSubnet_Native_OkResultFromParentToChild() public {
        address caller = address(new MockIpcContract());
        address recipient = address(new MockIpcContractPayable());
        uint256 amount = 3;

        vm.deal(address(rootNativeSubnetActor), DEFAULT_COLLATERAL_AMOUNT);
        vm.deal(caller, amount);
        vm.deal(recipient, amount);

        vm.prank(address(rootNativeSubnetActor));
        registerSubnetGW(DEFAULT_COLLATERAL_AMOUNT, address(rootNativeSubnetActor), rootGateway);

        IpcEnvelope memory crossMsg = CrossMsgHelper.createReleaseMsg(
            nativeSubnetName,
            recipient,
            FvmAddressHelper.from(caller),
            amount
        );

        IpcEnvelope memory resultMsg = CrossMsgHelper.createResultMsg(crossMsg, OutcomeType.Ok, new bytes(0));
        require(resultMsg.value == 0, "ok receipt should have 0 value");

        IpcEnvelope[] memory msgs = new IpcEnvelope[](1);
        msgs[0] = resultMsg;

        executeTopDownMsgs(msgs, nativeSubnetName, address(nativeSubnetGateway));

        // works with no state changes
        assertEq(recipient.balance, amount);
        assertEq(caller.balance, amount);
    }

    // A bottom up receipt sending from parent to child. The original message is a
    // bottom up release message, but the execution encounters system error in the parent,
    // creating a topdown result message from parent to child
    function testMultiSubnet_Native_SystemErrResultFromParentToChild() public {
        address caller = address(new MockIpcContract());
        address recipient = address(new MockIpcContractPayable());
        uint256 amount = 3;

        vm.deal(address(rootNativeSubnetActor), DEFAULT_COLLATERAL_AMOUNT);
        vm.deal(caller, amount);
        vm.deal(recipient, amount);

        vm.prank(address(rootNativeSubnetActor));
        registerSubnetGW(DEFAULT_COLLATERAL_AMOUNT, address(rootNativeSubnetActor), rootGateway);

        IpcEnvelope memory crossMsg = CrossMsgHelper.createReleaseMsg(
            nativeSubnetName,
            caller,
            FvmAddressHelper.from(recipient),
            amount
        );

        IpcEnvelope memory resultMsg = CrossMsgHelper.createResultMsg(crossMsg, OutcomeType.SystemErr, new bytes(0));

        IpcEnvelope[] memory msgs = new IpcEnvelope[](1);
        msgs[0] = resultMsg;

        executeTopDownMsgs(msgs, nativeSubnetName, address(nativeSubnetGateway));

        require(caller.balance == amount + amount, "refund not happening");
    }

    // A bottom up receipt sending from parent to child. The original message is a
    // bottom up release message, but the execution encounters system error in the parent,
    // creating a topdown result message from parent to child
    function testMultiSubnet_Native_ActorErrResultFromParentToChild() public {
        address caller = address(new MockIpcContract());
        address recipient = address(new MockIpcContractPayable());
        uint256 amount = 3;

        vm.deal(address(rootNativeSubnetActor), DEFAULT_COLLATERAL_AMOUNT);
        vm.deal(caller, amount);
        vm.deal(recipient, amount);

        vm.prank(address(rootNativeSubnetActor));
        registerSubnetGW(DEFAULT_COLLATERAL_AMOUNT, address(rootNativeSubnetActor), rootGateway);

        IpcEnvelope memory crossMsg = CrossMsgHelper.createReleaseMsg(
            nativeSubnetName,
            caller,
            FvmAddressHelper.from(recipient),
            amount
        );

        IpcEnvelope memory resultMsg = CrossMsgHelper.createResultMsg(crossMsg, OutcomeType.ActorErr, new bytes(0));

        IpcEnvelope[] memory msgs = new IpcEnvelope[](1);
        msgs[0] = resultMsg;

        executeTopDownMsgs(msgs, nativeSubnetName, address(nativeSubnetGateway));

        require(caller.balance == amount + amount, "refund not happening");
    }

    function testMultiSubnet_Native_NonPayable_FundingFromParentToChildFails() public {
        address caller = address(new MockIpcContract());
        address recipient = address(new MockIpcContractFallback());
        uint256 amount = 3;

        vm.deal(address(rootNativeSubnetActor), DEFAULT_COLLATERAL_AMOUNT);
        vm.deal(caller, amount);

        vm.prank(address(rootNativeSubnetActor));
        registerSubnetGW(DEFAULT_COLLATERAL_AMOUNT, address(rootNativeSubnetActor), rootGateway);

        IpcEnvelope memory expected = CrossMsgHelper.createFundMsg(
            nativeSubnetName,
            caller,
            FvmAddressHelper.from(recipient),
            amount
        );

        vm.prank(caller);
        vm.expectEmit(true, true, true, true, address(rootGateway));
        emit LibGateway.NewTopDownMessage(address(rootNativeSubnetActor), expected);
        rootGatewayManager.fund{value: amount}(nativeSubnetName, FvmAddressHelper.from(address(recipient)));

        IpcEnvelope[] memory msgs = new IpcEnvelope[](1);
        msgs[0] = expected;

        // TODO: commitParentFinality doesn't not affect anything in this test.
        commitParentFinality(address(nativeSubnetGateway));

        vm.expectRevert();
        executeTopDownMsgsRevert(msgs, tokenSubnetName, address(tokenSubnetGateway));
    }

    function testMultiSubnet_Erc20_FundingFromParentToChild() public {
        address caller = address(new MockIpcContract());
        address recipient = address(new MockIpcContractPayable());
        uint256 amount = 3;

        token.transfer(caller, 100);
        vm.prank(caller);
        token.approve(address(rootGateway), 100);

        vm.deal(address(rootTokenSubnetActor), DEFAULT_COLLATERAL_AMOUNT);
        vm.deal(caller, amount);

        vm.prank(address(rootTokenSubnetActor));
        registerSubnetGW(DEFAULT_COLLATERAL_AMOUNT, address(rootTokenSubnetActor), rootGateway);

        IpcEnvelope memory expected = CrossMsgHelper.createFundMsg(
            tokenSubnetName,
            caller,
            FvmAddressHelper.from(recipient),
            amount
        );

        vm.prank(caller);
        vm.expectEmit(true, true, true, true, address(rootGateway));
        emit LibGateway.NewTopDownMessage(address(rootTokenSubnetActor), expected);
        rootGatewayManager.fundWithToken(tokenSubnetName, FvmAddressHelper.from(address(recipient)), amount);

        IpcEnvelope[] memory msgs = new IpcEnvelope[](1);
        msgs[0] = expected;

        commitParentFinality(address(tokenSubnetGateway));

        executeTopDownMsgs(msgs, tokenSubnetName, address(tokenSubnetGateway));

        assertEq(recipient.balance, amount);
    }

    function testMultiSubnet_Erc20_ReleaseResultOkFromParentToChild() public {
        address caller = address(new MockIpcContract());
        address recipient = address(new MockIpcContractPayable());
        uint256 amount = 3;

        token.transfer(caller, 100);
        vm.prank(caller);
        token.approve(address(rootGateway), 100);

        vm.deal(address(rootTokenSubnetActor), DEFAULT_COLLATERAL_AMOUNT);
        vm.deal(caller, amount);

        vm.prank(address(rootTokenSubnetActor));
        registerSubnetGW(DEFAULT_COLLATERAL_AMOUNT, address(rootTokenSubnetActor), rootGateway);

        IpcEnvelope memory crossMsg = CrossMsgHelper.createReleaseMsg(
            tokenSubnetName,
            caller,
            FvmAddressHelper.from(recipient),
            amount
        );
        IpcEnvelope memory resultMsg = CrossMsgHelper.createResultMsg(crossMsg, OutcomeType.Ok, new bytes(0));

        IpcEnvelope[] memory msgs = new IpcEnvelope[](1);
        msgs[0] = resultMsg;

        executeTopDownMsgs(msgs, tokenSubnetName, address(tokenSubnetGateway));

        assertEq(caller.balance, amount);
    }

    function testMultiSubnet_Erc20_ReleaseActorErrFromParentToChild() public {
        address caller = address(new MockIpcContract());
        address recipient = address(new MockIpcContractPayable());
        uint256 amount = 3;

        token.transfer(caller, 100);
        vm.prank(caller);
        token.approve(address(rootGateway), 100);

        vm.deal(address(rootTokenSubnetActor), DEFAULT_COLLATERAL_AMOUNT);
        vm.deal(caller, amount);

        vm.prank(address(rootTokenSubnetActor));
        registerSubnetGW(DEFAULT_COLLATERAL_AMOUNT, address(rootTokenSubnetActor), rootGateway);

        IpcEnvelope memory crossMsg = CrossMsgHelper.createReleaseMsg(
            tokenSubnetName,
            caller,
            FvmAddressHelper.from(recipient),
            amount
        );
        IpcEnvelope memory resultMsg = CrossMsgHelper.createResultMsg(crossMsg, OutcomeType.ActorErr, new bytes(0));

        IpcEnvelope[] memory msgs = new IpcEnvelope[](1);
        msgs[0] = resultMsg;

        executeTopDownMsgs(msgs, tokenSubnetName, address(tokenSubnetGateway));
        require(caller.balance == amount + amount, "refund should have happened");
    }

    function testMultiSubnet_Erc20_ReleaseSystemErrFromParentToChild() public {
        address caller = address(new MockIpcContract());
        address recipient = address(new MockIpcContractPayable());
        uint256 amount = 3;

        token.transfer(caller, 100);
        vm.prank(caller);
        token.approve(address(rootGateway), 100);

        vm.deal(address(rootTokenSubnetActor), DEFAULT_COLLATERAL_AMOUNT);
        vm.deal(caller, amount);

        vm.prank(address(rootTokenSubnetActor));
        registerSubnetGW(DEFAULT_COLLATERAL_AMOUNT, address(rootTokenSubnetActor), rootGateway);

        IpcEnvelope memory crossMsg = CrossMsgHelper.createReleaseMsg(
            tokenSubnetName,
            caller,
            FvmAddressHelper.from(recipient),
            amount
        );
        IpcEnvelope memory resultMsg = CrossMsgHelper.createResultMsg(crossMsg, OutcomeType.SystemErr, new bytes(0));

        IpcEnvelope[] memory msgs = new IpcEnvelope[](1);
        msgs[0] = resultMsg;

        executeTopDownMsgs(msgs, tokenSubnetName, address(tokenSubnetGateway));
        require(caller.balance == amount + amount, "refund should have happened");
    }

    function testMultiSubnet_Erc20_NonPayable_FundingFromParentToChildFails() public {
        address caller = address(new MockIpcContract());
        address recipient = address(new MockIpcContractFallback());
        uint256 amount = 3;

        token.transfer(caller, 100);
        vm.prank(caller);
        token.approve(address(rootGateway), 100);

        vm.deal(address(rootTokenSubnetActor), DEFAULT_COLLATERAL_AMOUNT);
        vm.deal(caller, amount);

        vm.prank(address(rootTokenSubnetActor));
        registerSubnetGW(DEFAULT_COLLATERAL_AMOUNT, address(rootTokenSubnetActor), rootGateway);

        IpcEnvelope memory expected = CrossMsgHelper.createFundMsg(
            tokenSubnetName,
            caller,
            FvmAddressHelper.from(recipient),
            amount
        );

        vm.prank(caller);
        vm.expectEmit(true, true, true, true, address(rootGateway));
        emit LibGateway.NewTopDownMessage(address(rootTokenSubnetActor), expected);
        rootGatewayManager.fundWithToken(tokenSubnetName, FvmAddressHelper.from(address(recipient)), amount);

        IpcEnvelope[] memory msgs = new IpcEnvelope[](1);
        msgs[0] = expected;

        commitParentFinality(address(tokenSubnetGateway));

        vm.expectRevert();
        executeTopDownMsgsRevert(msgs, tokenSubnetName, address(tokenSubnetGateway));
    }

    //--------------------
    // Release flow tests.
    //---------------------

    function testMultiSubnet_Native_ReleaseFromChildToParent() public {
        address caller = address(new MockIpcContract());
        address recipient = address(new MockIpcContractPayable());
        uint256 amount = 3;

        vm.deal(address(rootNativeSubnetActor), DEFAULT_COLLATERAL_AMOUNT);
        vm.deal(caller, 1 ether);

        vm.prank(address(rootNativeSubnetActor));
        registerSubnetGW(DEFAULT_COLLATERAL_AMOUNT, address(rootNativeSubnetActor), rootGateway);

        vm.prank(caller);
        rootGatewayManager.fund{value: amount}(nativeSubnetName, FvmAddressHelper.from(address(caller)));

        GatewayManagerFacet manager = GatewayManagerFacet(address(nativeSubnetGateway));

        vm.prank(caller);
        manager.release{value: amount}(FvmAddressHelper.from(address(recipient)));

        BottomUpCheckpoint memory checkpoint = callCreateBottomUpCheckpointFromChildSubnet(
            nativeSubnetName,
            address(nativeSubnetGateway)
        );

        submitBottomUpCheckpoint(checkpoint, address(rootNativeSubnetActor));
        assertEq(recipient.balance, amount);
    }

    // The result message from the parent sending a fund message to the child.
    // The result message would be a bottom up cross message from the child to
    // the parent.
    function testMultiSubnet_Native_FundOkResultFromChildToParent() public {
        address caller = address(new MockIpcContract());
        address recipient = address(new MockIpcContractPayable());
        uint256 amount = 3;

        vm.deal(address(rootNativeSubnetActor), DEFAULT_COLLATERAL_AMOUNT);
        vm.deal(caller, 1 ether);

        vm.prank(address(rootNativeSubnetActor));
        registerSubnetGW(DEFAULT_COLLATERAL_AMOUNT, address(rootNativeSubnetActor), rootGateway);

        IpcEnvelope memory crossMsg = CrossMsgHelper.createFundMsg(
            nativeSubnetName,
            caller,
            FvmAddressHelper.from(recipient),
            amount
        );
        IpcEnvelope memory resultMsg = CrossMsgHelper.createResultMsg(crossMsg, OutcomeType.Ok, new bytes(0));
        IpcEnvelope[] memory crossMsgs = new IpcEnvelope[](1);
        crossMsgs[0] = resultMsg;

        GatewayManagerFacet manager = GatewayManagerFacet(address(nativeSubnetGateway));

        BottomUpCheckpoint memory checkpoint = callCreateBottomUpCheckpointFromChildSubnet(
            nativeSubnetName,
            address(nativeSubnetGateway),
            crossMsgs
        );

        submitBottomUpCheckpoint(checkpoint, address(rootNativeSubnetActor));

        // no change to caller's balance
        assertEq(caller.balance, 1 ether);
    }

    // The result message from the parent sending a fund message to the child.
    // The result message would be a bottom up cross message from the child to
    // the parent.
    function testMultiSubnet_Native_FundActorErrResultFromChildToParent() public {
        address caller = address(new MockIpcContract());
        address recipient = address(new MockIpcContractPayable());
        uint256 amount = 3;

        vm.deal(address(rootNativeSubnetActor), DEFAULT_COLLATERAL_AMOUNT);
        vm.deal(caller, 1 ether);

        vm.prank(address(rootNativeSubnetActor));
        registerSubnetGW(DEFAULT_COLLATERAL_AMOUNT, address(rootNativeSubnetActor), rootGateway);

        // fund first to provide circulation
        vm.prank(caller);
        rootGatewayManager.fund{value: amount}(nativeSubnetName, FvmAddressHelper.from(address(caller)));
        assertEq(caller.balance, 1 ether - amount);

        IpcEnvelope memory crossMsg = CrossMsgHelper.createFundMsg(
            nativeSubnetName,
            caller,
            FvmAddressHelper.from(recipient),
            amount
        );
        IpcEnvelope memory resultMsg = CrossMsgHelper.createResultMsg(crossMsg, OutcomeType.ActorErr, new bytes(0));
        IpcEnvelope[] memory crossMsgs = new IpcEnvelope[](1);
        crossMsgs[0] = resultMsg;

        GatewayManagerFacet manager = GatewayManagerFacet(address(nativeSubnetGateway));

        BottomUpCheckpoint memory checkpoint = callCreateBottomUpCheckpointFromChildSubnet(
            nativeSubnetName,
            address(nativeSubnetGateway),
            crossMsgs
        );

        submitBottomUpCheckpoint(checkpoint, address(rootNativeSubnetActor));

        // caller's balance should receive the refund
        assertEq(caller.balance, 1 ether);
    }

    // The result message from the parent sending a fund message to the child.
    // The result message would be a bottom up cross message from the child to
    // the parent.
    function testMultiSubnet_Native_FundSystemErrResultFromChildToParent() public {
        address caller = address(new MockIpcContract());
        address recipient = address(new MockIpcContractPayable());
        uint256 amount = 3;

        vm.deal(address(rootNativeSubnetActor), DEFAULT_COLLATERAL_AMOUNT);
        vm.deal(caller, 1 ether);

        vm.prank(address(rootNativeSubnetActor));
        registerSubnetGW(DEFAULT_COLLATERAL_AMOUNT, address(rootNativeSubnetActor), rootGateway);

        // fund first to provide circulation
        vm.prank(caller);
        rootGatewayManager.fund{value: amount}(nativeSubnetName, FvmAddressHelper.from(address(caller)));
        assertEq(caller.balance, 1 ether - amount);

        // now the fund propagated to the child and execution is Ok
        // assuming the Result message is created in the child message batch
        // and relayer pushed to the parent

        IpcEnvelope memory crossMsg = CrossMsgHelper.createFundMsg(
            nativeSubnetName,
            caller,
            FvmAddressHelper.from(recipient),
            amount
        );
        IpcEnvelope memory resultMsg = CrossMsgHelper.createResultMsg(crossMsg, OutcomeType.SystemErr, new bytes(0));
        IpcEnvelope[] memory crossMsgs = new IpcEnvelope[](1);
        crossMsgs[0] = resultMsg;

        GatewayManagerFacet manager = GatewayManagerFacet(address(nativeSubnetGateway));

        BottomUpCheckpoint memory checkpoint = callCreateBottomUpCheckpointFromChildSubnet(
            nativeSubnetName,
            address(nativeSubnetGateway),
            crossMsgs
        );

        submitBottomUpCheckpoint(checkpoint, address(rootNativeSubnetActor));

        // caller's balance should receive the refund
        assertEq(caller.balance, 1 ether);
    }

    function testMultiSubnet_Native_NonPayable_ReleaseFromChildToParentFails() public {
        address caller = address(new MockIpcContract());
        address recipient = address(new MockIpcContractFallback());
        uint256 amount = 3;

        vm.deal(address(rootNativeSubnetActor), DEFAULT_COLLATERAL_AMOUNT);
        vm.deal(caller, 6);

        vm.prank(address(rootNativeSubnetActor));
        registerSubnetGW(DEFAULT_COLLATERAL_AMOUNT, address(rootNativeSubnetActor), rootGateway);

        vm.prank(caller);
        rootGatewayManager.fund{value: amount}(nativeSubnetName, FvmAddressHelper.from(address(caller)));

        GatewayManagerFacet manager = GatewayManagerFacet(address(nativeSubnetGateway));
        vm.prank(caller);
        manager.release{value: amount}(FvmAddressHelper.from(address(recipient)));

        BottomUpCheckpoint memory checkpoint = callCreateBottomUpCheckpointFromChildSubnet(
            nativeSubnetName,
            address(nativeSubnetGateway)
        );

        vm.expectRevert();
        submitBottomUpCheckpointRevert(checkpoint, address(rootNativeSubnetActor));
    }

    function testMultiSubnet_Native_ReleaseFromChildToParent_DifferentFunderAndSenderInParent() public {
        address caller = address(new MockIpcContract());
        address recipient = address(new MockIpcContractPayable());
        uint256 amount = 3;

        vm.deal(address(rootNativeSubnetActor), DEFAULT_COLLATERAL_AMOUNT);
        vm.deal(caller, 6);

        vm.prank(address(rootNativeSubnetActor));
        registerSubnetGW(DEFAULT_COLLATERAL_AMOUNT, address(rootNativeSubnetActor), rootGateway);

        vm.prank(caller);
        rootGatewayManager.fund{value: amount}(nativeSubnetName, FvmAddressHelper.from(address(caller)));

        GatewayManagerFacet manager = GatewayManagerFacet(address(nativeSubnetGateway));
        vm.prank(caller);
        manager.release{value: amount}(FvmAddressHelper.from(address(recipient)));

        BottomUpCheckpoint memory checkpoint = callCreateBottomUpCheckpointFromChildSubnet(
            nativeSubnetName,
            address(nativeSubnetGateway)
        );

        submitBottomUpCheckpoint(checkpoint, address(rootNativeSubnetActor));

        assertEq(recipient.balance, amount);
    }

    function testMultiSubnet_Erc20_FundResultOkFromChildToParent() public {
        address caller = address(new MockIpcContract());
        address recipient = address(new MockIpcContractPayable());
        uint256 amount = 3;

        token.transfer(caller, amount);
        vm.prank(caller);
        token.approve(address(rootGateway), amount);

        vm.deal(address(rootTokenSubnetActor), DEFAULT_COLLATERAL_AMOUNT);
        vm.deal(caller, 1 ether);

        vm.prank(address(rootTokenSubnetActor));
        registerSubnetGW(DEFAULT_COLLATERAL_AMOUNT, address(rootTokenSubnetActor), rootGateway);

        vm.prank(caller);
        rootGatewayManager.fundWithToken(tokenSubnetName, FvmAddressHelper.from(address(caller)), amount);
        assertEq(token.balanceOf(caller), 0);

        // now the fund propagated to the child and execution is Ok
        // assuming the Result message is created in the child message batch
        // and relayer pushed to the parent

        // simulating the checkpoint pushed from relayer
        IpcEnvelope memory crossMsg = CrossMsgHelper.createFundMsg(
            tokenSubnetName,
            caller,
            FvmAddressHelper.from(caller),
            amount
        );
        IpcEnvelope memory resultMsg = CrossMsgHelper.createResultMsg(crossMsg, OutcomeType.Ok, new bytes(0));
        IpcEnvelope[] memory crossMsgs = new IpcEnvelope[](1);
        crossMsgs[0] = resultMsg;

        BottomUpCheckpoint memory checkpoint = callCreateBottomUpCheckpointFromChildSubnet(
            tokenSubnetName,
            address(tokenSubnetGateway),
            crossMsgs
        );

        submitBottomUpCheckpoint(checkpoint, address(rootTokenSubnetActor));

        // fund works, so fund is still locked, balance is 0
        assertEq(token.balanceOf(caller), 0);
    }

    function testMultiSubnet_Erc20_FundResultSystemErrFromChildToParent() public {
        address caller = address(new MockIpcContract());
        address recipient = address(new MockIpcContractPayable());
        uint256 amount = 3;

        token.transfer(caller, amount);
        vm.prank(caller);
        token.approve(address(rootGateway), amount);

        vm.deal(address(rootTokenSubnetActor), DEFAULT_COLLATERAL_AMOUNT);
        vm.deal(caller, 1 ether);

        vm.prank(address(rootTokenSubnetActor));
        registerSubnetGW(DEFAULT_COLLATERAL_AMOUNT, address(rootTokenSubnetActor), rootGateway);

        vm.prank(caller);
        rootGatewayManager.fundWithToken(tokenSubnetName, FvmAddressHelper.from(address(caller)), amount);
        assertEq(token.balanceOf(caller), 0);

        // now the fund propagated to the child and execution is Ok
        // assuming the Result message is created in the child message batch
        // and relayer pushed to the parent

        // simulating the checkpoint pushed from relayer
        IpcEnvelope memory crossMsg = CrossMsgHelper.createFundMsg(
            tokenSubnetName,
            caller,
            FvmAddressHelper.from(caller),
            amount
        );
        IpcEnvelope memory resultMsg = CrossMsgHelper.createResultMsg(crossMsg, OutcomeType.SystemErr, new bytes(0));
        IpcEnvelope[] memory crossMsgs = new IpcEnvelope[](1);
        crossMsgs[0] = resultMsg;

        BottomUpCheckpoint memory checkpoint = callCreateBottomUpCheckpointFromChildSubnet(
            tokenSubnetName,
            address(tokenSubnetGateway),
            crossMsgs
        );

        submitBottomUpCheckpoint(checkpoint, address(rootTokenSubnetActor));

        // fund rejected, so fund should be unlocked
        assertEq(token.balanceOf(caller), amount);
    }

    function testMultiSubnet_Erc20_FundResultActorErrFromChildToParent() public {
        address caller = address(new MockIpcContract());
        address recipient = address(new MockIpcContractPayable());
        uint256 amount = 3;

        token.transfer(caller, amount);
        vm.prank(caller);
        token.approve(address(rootGateway), amount);

        vm.deal(address(rootTokenSubnetActor), DEFAULT_COLLATERAL_AMOUNT);
        vm.deal(caller, 1 ether);

        vm.prank(address(rootTokenSubnetActor));
        registerSubnetGW(DEFAULT_COLLATERAL_AMOUNT, address(rootTokenSubnetActor), rootGateway);

        vm.prank(caller);
        rootGatewayManager.fundWithToken(tokenSubnetName, FvmAddressHelper.from(address(caller)), amount);
        assertEq(token.balanceOf(caller), 0);

        // now the fund propagated to the child and execution is Ok
        // assuming the Result message is created in the child message batch
        // and relayer pushed to the parent

        // simulating the checkpoint pushed from relayer
        IpcEnvelope memory crossMsg = CrossMsgHelper.createFundMsg(
            tokenSubnetName,
            caller,
            FvmAddressHelper.from(caller),
            amount
        );
        IpcEnvelope memory resultMsg = CrossMsgHelper.createResultMsg(crossMsg, OutcomeType.ActorErr, new bytes(0));
        IpcEnvelope[] memory crossMsgs = new IpcEnvelope[](1);
        crossMsgs[0] = resultMsg;

        BottomUpCheckpoint memory checkpoint = callCreateBottomUpCheckpointFromChildSubnet(
            tokenSubnetName,
            address(tokenSubnetGateway),
            crossMsgs
        );

        submitBottomUpCheckpoint(checkpoint, address(rootTokenSubnetActor));

        // fund rejected, so fund should be unlocked
        assertEq(token.balanceOf(caller), amount);
    }

    function testMultiSubnet_Erc20_ReleaseFromChildToParent() public {
        address caller = address(new MockIpcContract());
        address recipient = address(new MockIpcContractPayable());
        uint256 amount = 3;

        token.transfer(caller, amount);
        vm.prank(caller);
        token.approve(address(rootGateway), amount);

        vm.deal(address(rootTokenSubnetActor), DEFAULT_COLLATERAL_AMOUNT);
        vm.deal(caller, 1 ether);

        vm.prank(address(rootTokenSubnetActor));
        registerSubnetGW(DEFAULT_COLLATERAL_AMOUNT, address(rootTokenSubnetActor), rootGateway);

        vm.prank(caller);
        rootGatewayManager.fundWithToken(tokenSubnetName, FvmAddressHelper.from(address(caller)), amount);

        GatewayManagerFacet manager = GatewayManagerFacet(address(tokenSubnetGateway));
        vm.prank(caller);
        manager.release{value: amount}(FvmAddressHelper.from(address(recipient)));

        BottomUpCheckpoint memory checkpoint = callCreateBottomUpCheckpointFromChildSubnet(
            tokenSubnetName,
            address(tokenSubnetGateway)
        );

        submitBottomUpCheckpoint(checkpoint, address(rootTokenSubnetActor));

        assertEq(token.balanceOf(recipient), amount);
    }

    function testMultiSubnet_Erc20_Transfer_NonPayable_ReleaseFromChildToParent() public {
        address caller = address(new MockIpcContract());
        address recipient = address(new MockIpcContractFallback());
        uint256 amount = 3;

        token.transfer(caller, amount);
        vm.prank(caller);
        token.approve(address(rootGateway), amount);

        vm.deal(address(rootTokenSubnetActor), DEFAULT_COLLATERAL_AMOUNT);
        vm.deal(caller, 1 ether);

        vm.prank(address(rootTokenSubnetActor));
        registerSubnetGW(DEFAULT_COLLATERAL_AMOUNT, address(rootTokenSubnetActor), rootGateway);

        vm.prank(caller);
        rootGatewayManager.fundWithToken(tokenSubnetName, FvmAddressHelper.from(address(caller)), amount);

        GatewayManagerFacet manager = GatewayManagerFacet(address(tokenSubnetGateway));
        vm.prank(caller);
        manager.release{value: amount}(FvmAddressHelper.from(address(recipient)));

        BottomUpCheckpoint memory checkpoint = callCreateBottomUpCheckpointFromChildSubnet(
            tokenSubnetName,
            address(tokenSubnetGateway)
        );

        submitBottomUpCheckpoint(checkpoint, address(rootTokenSubnetActor));
        assertEq(token.balanceOf(recipient), amount);
        assertEq(recipient.balance, 0);
    }

    //--------------------
    // Call flow tests.
    //---------------------

    function testMultiSubnet_Native_SendCrossMessageFromChildToParent() public {
        address caller = address(new MockIpcContract());
        address recipient = address(new MockIpcContract());
        uint256 amount = 3;

        vm.deal(address(rootNativeSubnetActor), DEFAULT_COLLATERAL_AMOUNT);
        vm.deal(caller, 1 ether);

        vm.prank(address(rootNativeSubnetActor));
        registerSubnetGW(DEFAULT_COLLATERAL_AMOUNT, address(rootNativeSubnetActor), rootGateway);

        vm.prank(caller);
        rootGatewayManager.fund{value: 100000}(nativeSubnetName, FvmAddressHelper.from(address(caller)));

        GatewayMessengerFacet messenger = GatewayMessengerFacet(address(nativeSubnetGateway));
        vm.prank(address(caller));
        messenger.sendContractXnetMessage{value: amount}(
            TestUtils.newXnetCallMsg(
                IPCAddress({subnetId: nativeSubnetName, rawAddress: FvmAddressHelper.from(caller)}),
                IPCAddress({subnetId: rootSubnetName, rawAddress: FvmAddressHelper.from(recipient)}),
                amount,
                0
            )
        );

        BottomUpCheckpoint memory checkpoint = callCreateBottomUpCheckpointFromChildSubnet(
            nativeSubnetName,
            address(nativeSubnetGateway)
        );

        submitBottomUpCheckpoint(checkpoint, address(rootNativeSubnetActor));

        assertEq(recipient.balance, amount);
    }

    function testMultiSubnet_Native_SendCrossMessageFromParentToChild() public {
        address caller = address(new MockIpcContract());
        address recipient = address(new MockIpcContract());
        uint256 amount = 3;

        vm.deal(address(rootNativeSubnetActor), DEFAULT_COLLATERAL_AMOUNT);
        vm.deal(caller, 1 ether);

        vm.prank(address(rootNativeSubnetActor));
        registerSubnetGW(DEFAULT_COLLATERAL_AMOUNT, address(rootNativeSubnetActor), rootGateway);

        vm.prank(caller);
        rootGatewayManager.fund{value: 100000}(nativeSubnetName, FvmAddressHelper.from(address(caller)));

        IpcEnvelope memory xnetCallMsg = TestUtils.newXnetCallMsg(
            IPCAddress({subnetId: rootSubnetName, rawAddress: FvmAddressHelper.from(caller)}),
            IPCAddress({subnetId: nativeSubnetName, rawAddress: FvmAddressHelper.from(recipient)}),
            amount,
            0
        );

        IpcEnvelope memory committedEvent = IpcEnvelope({
            kind: IpcMsgKind.Call,
            from: IPCAddress({subnetId: rootSubnetName, rawAddress: FvmAddressHelper.from(caller)}),
            to: xnetCallMsg.to,
            value: xnetCallMsg.value,
            message: xnetCallMsg.message,
            nonce: 1
        });

        GatewayMessengerFacet rootGatewayMessenger = GatewayMessengerFacet(address(rootGateway));
        vm.prank(address(caller));
        vm.expectEmit(true, true, true, true, address(rootGateway));
        emit LibGateway.NewTopDownMessage({subnet: address(rootNativeSubnetActor), message: committedEvent});
        rootGatewayMessenger.sendContractXnetMessage{value: amount}(xnetCallMsg);

        IpcEnvelope[] memory msgs = new IpcEnvelope[](1);
        msgs[0] = xnetCallMsg;

        commitParentFinality(address(nativeSubnetGateway));
        executeTopDownMsgs(msgs, nativeSubnetName, address(nativeSubnetGateway));

        assertEq(address(recipient).balance, amount);
    }

    function testMultiSubnet_Token_CallFromChildToParent() public {
        address caller = address(new MockIpcContract());
        address recipient = address(new MockIpcContract());
        uint256 amount = 3;

        vm.deal(address(rootTokenSubnetActor), DEFAULT_COLLATERAL_AMOUNT);
        vm.deal(address(token), DEFAULT_COLLATERAL_AMOUNT);
        vm.deal(caller, 1 ether);

        // Fund an account in the subnet.
        token.transfer(caller, 100);
        vm.prank(caller);
        token.approve(address(rootGateway), 100);

        vm.prank(address(rootTokenSubnetActor));
        registerSubnetGW(DEFAULT_COLLATERAL_AMOUNT, address(rootTokenSubnetActor), rootGateway);

        vm.prank(caller);
        rootGatewayManager.fundWithToken(tokenSubnetName, FvmAddressHelper.from(address(caller)), 15);

        IPCAddress memory from = IPCAddress({subnetId: tokenSubnetName, rawAddress: FvmAddressHelper.from(caller)});
        IPCAddress memory to = IPCAddress({subnetId: rootSubnetName, rawAddress: FvmAddressHelper.from(recipient)});
        bytes4 method = bytes4(0x11223344);
        bytes memory params = bytes("hello");
        IpcEnvelope memory envelope = CrossMsgHelper.createCallMsg(from, to, amount, method, params);

        GatewayMessengerFacet messenger = GatewayMessengerFacet(address(tokenSubnetGateway));
        vm.prank(address(caller));
        messenger.sendContractXnetMessage{value: amount}(envelope);

        BottomUpCheckpoint memory checkpoint = callCreateBottomUpCheckpointFromChildSubnet(
            tokenSubnetName,
            address(tokenSubnetGateway)
        );

        submitBottomUpCheckpoint(checkpoint, address(rootTokenSubnetActor));

        assertEq(token.balanceOf(recipient), amount);
    }

    function testMultiSubnet_Erc20_SendCrossMessageFromParentToChild() public {
        address caller = address(new MockIpcContract());
        address recipient = address(new MockIpcContract());
        uint256 amount = 3;

        token.transfer(caller, 100);
        vm.prank(caller);
        token.approve(address(rootGateway), 100);

        vm.deal(address(rootTokenSubnetActor), DEFAULT_COLLATERAL_AMOUNT);
        vm.deal(caller, 3);

        vm.prank(address(rootTokenSubnetActor));
        registerSubnetGW(DEFAULT_COLLATERAL_AMOUNT, address(rootTokenSubnetActor), rootGateway);

        vm.prank(caller);
        rootGatewayManager.fundWithToken(tokenSubnetName, FvmAddressHelper.from(address(caller)), 15);

        IpcEnvelope memory xnetCallMsg = TestUtils.newXnetCallMsg(
            IPCAddress({subnetId: rootSubnetName, rawAddress: FvmAddressHelper.from(caller)}),
            IPCAddress({subnetId: tokenSubnetName, rawAddress: FvmAddressHelper.from(recipient)}),
            amount,
            0
        );

        IpcEnvelope memory committedEvent = IpcEnvelope({
            kind: IpcMsgKind.Call,
            from: IPCAddress({subnetId: rootSubnetName, rawAddress: FvmAddressHelper.from(caller)}),
            to: xnetCallMsg.to,
            value: xnetCallMsg.value,
            message: xnetCallMsg.message,
            nonce: 1
        });

        GatewayMessengerFacet rootGatewayMessenger = GatewayMessengerFacet(address(rootGateway));
        vm.prank(address(caller));
        vm.expectEmit(true, true, true, true, address(rootGateway));
        emit LibGateway.NewTopDownMessage({subnet: address(rootTokenSubnetActor), message: committedEvent});
        rootGatewayMessenger.sendContractXnetMessage{value: amount}(xnetCallMsg);

        IpcEnvelope[] memory msgs = new IpcEnvelope[](1);
        msgs[0] = xnetCallMsg;

        commitParentFinality(address(tokenSubnetGateway));
        executeTopDownMsgs(msgs, tokenSubnetName, address(tokenSubnetGateway));

        assertEq(address(recipient).balance, amount);
    }

    function commitParentFinality(address gateway) internal {
        vm.roll(10);
        ParentFinality memory finality = ParentFinality({height: block.number, blockHash: bytes32(0)});

        TopDownFinalityFacet gwTopDownFinalityFacet = TopDownFinalityFacet(address(gateway));

        vm.prank(FilAddress.SYSTEM_ACTOR);
        gwTopDownFinalityFacet.commitParentFinality(finality);
    }

    function executeTopDownMsgs(IpcEnvelope[] memory msgs, SubnetID memory subnet, address gateway) internal {
        XnetMessagingFacet xnet = XnetMessagingFacet(address(gateway));

        uint256 minted_tokens;

        for (uint256 i; i < msgs.length; ) {
            minted_tokens += msgs[i].value;
            unchecked {
                ++i;
            }
        }
        console.log("minted tokens in executed top-downs: %d", minted_tokens);

        // The implementation of the function in fendermint is in
        // https://github.com/consensus-shipyard/ipc/blob/main/fendermint/vm/interpreter/src/fvm/topdown.rs#L43

        // This emulates minting tokens.
        vm.deal(address(gateway), minted_tokens);

        // TODO: how to emulate increase of circulation supply?

        vm.prank(FilAddress.SYSTEM_ACTOR);
        xnet.applyCrossMessages(msgs);
    }

    function executeTopDownMsgsRevert(IpcEnvelope[] memory msgs, SubnetID memory subnet, address gateway) internal {
        vm.expectRevert();
        executeTopDownMsgs(msgs, subnet, gateway);
    }

    function callCreateBottomUpCheckpointFromChildSubnet(
        SubnetID memory subnet,
        address gateway
    ) internal returns (BottomUpCheckpoint memory checkpoint) {
        uint256 e = getNextEpoch(block.number, DEFAULT_CHECKPOINT_PERIOD);

        GatewayGetterFacet getter = GatewayGetterFacet(address(gateway));
        CheckpointingFacet checkpointer = CheckpointingFacet(address(gateway));

        BottomUpMsgBatch memory batch = getter.bottomUpMsgBatch(e);
        require(batch.msgs.length == 1, "batch length incorrect");

        (, address[] memory addrs, uint256[] memory weights) = TestUtils.getFourValidators(vm);

        (bytes32 membershipRoot, ) = MerkleTreeHelper.createMerkleProofsForValidators(addrs, weights);

        checkpoint = BottomUpCheckpoint({
            subnetID: subnet,
            blockHeight: batch.blockHeight,
            blockHash: keccak256("block1"),
            nextConfigurationNumber: 0,
            msgs: batch.msgs
        });

        vm.startPrank(FilAddress.SYSTEM_ACTOR);
        checkpointer.createBottomUpCheckpoint(checkpoint, membershipRoot, weights[0] + weights[1] + weights[2]);
        vm.stopPrank();

        return checkpoint;
    }

    function callCreateBottomUpCheckpointFromChildSubnet(
        SubnetID memory subnet,
        address gateway,
        IpcEnvelope[] memory msgs
    ) internal returns (BottomUpCheckpoint memory checkpoint) {
        uint256 e = getNextEpoch(block.number, DEFAULT_CHECKPOINT_PERIOD);

        GatewayGetterFacet getter = GatewayGetterFacet(address(gateway));
        CheckpointingFacet checkpointer = CheckpointingFacet(address(gateway));

        (, address[] memory addrs, uint256[] memory weights) = TestUtils.getFourValidators(vm);

        (bytes32 membershipRoot, ) = MerkleTreeHelper.createMerkleProofsForValidators(addrs, weights);

        checkpoint = BottomUpCheckpoint({
            subnetID: subnet,
            blockHeight: e,
            blockHash: keccak256("block1"),
            nextConfigurationNumber: 0,
            msgs: msgs
        });

        vm.startPrank(FilAddress.SYSTEM_ACTOR);
        checkpointer.createBottomUpCheckpoint(checkpoint, membershipRoot, weights[0] + weights[1] + weights[2]);
        vm.stopPrank();

        return checkpoint;
    }

    function submitBottomUpCheckpoint(BottomUpCheckpoint memory checkpoint, address subnetActor) internal {
        (uint256[] memory parentKeys, address[] memory parentValidators, ) = TestUtils.getThreeValidators(vm);
        bytes[] memory parentPubKeys = new bytes[](3);
        bytes[] memory parentSignatures = new bytes[](3);

        SubnetActorManagerFacet manager = SubnetActorManagerFacet(subnetActor);

        for (uint256 i = 0; i < 3; i++) {
            vm.deal(parentValidators[i], 10 gwei);
            parentPubKeys[i] = TestUtils.deriveValidatorPubKeyBytes(parentKeys[i]);
            vm.prank(parentValidators[i]);
            manager.join{value: 10}(parentPubKeys[i]);
        }

        bytes32 hash = keccak256(abi.encode(checkpoint));

        for (uint256 i = 0; i < 3; i++) {
            (uint8 v, bytes32 r, bytes32 s) = vm.sign(parentKeys[i], hash);
            parentSignatures[i] = abi.encodePacked(r, s, v);
        }

        SubnetActorCheckpointingFacet checkpointer = SubnetActorCheckpointingFacet(subnetActor);

        vm.startPrank(subnetActor);
        checkpointer.submitCheckpoint(checkpoint, parentValidators, parentSignatures);
        vm.stopPrank();
    }

    function submitBottomUpCheckpointRevert(BottomUpCheckpoint memory checkpoint, address subnetActor) internal {
        vm.expectRevert();
        submitBottomUpCheckpoint(checkpoint, subnetActor);
    }

    function getNextEpoch(uint256 blockNumber, uint256 checkPeriod) internal pure returns (uint256) {
        return ((uint64(blockNumber) / checkPeriod) + 1) * checkPeriod;
    }

    function printActors() internal view {
        console.log("root gateway: %s", address(rootGateway));
        console.log("root actor: %s", rootSubnetName.getActor());
        console.log("root native subnet actor: %s", (address(rootNativeSubnetActor)));
        console.log("root token subnet actor: %s", (address(rootTokenSubnetActor)));
        console.log("root name: %s", rootSubnetName.toString());
        console.log("native subnet name: %s", nativeSubnetName.toString());
        console.log("token subnet name: %s", tokenSubnetName.toString());
        console.log("native subnet getActor(): %s", address(nativeSubnetName.getActor()));
        console.log("native subnet gateway(): %s", address(nativeSubnetGateway));
    }

    function printEnvelope(IpcEnvelope memory envelope) internal {
        console.log("from %s:", envelope.from.subnetId.toString());
        console.log("to %s:", envelope.to.subnetId.toString());
    }
}
