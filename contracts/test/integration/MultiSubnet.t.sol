// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity 0.8.19;

import "forge-std/Test.sol";
import "../../src/errors/IPCErrors.sol";
import {EMPTY_BYTES, METHOD_SEND} from "../../src/constants/Constants.sol";
import {IpcEnvelope, BottomUpMsgBatch, BottomUpCheckpoint, ParentFinality} from "../../src/structs/CrossNet.sol";
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

import {SubnetTokenBridge} from "../../src/examples/cross-token/SubnetTokenBridge.sol";
import {SubnetUSDCProxy} from "../../src/examples/cross-token/SubnetUSDCProxy.sol";
import {TokenTransferAndMint} from "../../src/examples/cross-token/TokenTransferAndMint.sol";
import {USDCMock} from "../../src/examples/cross-token/USDCMock.sol";

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

    SubnetTokenBridge subnetTokenBridge;
    TokenTransferAndMint rootTokenBridge;

    SubnetUSDCProxy subnetUSDCProxy;

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

    function testMultiSubnet_Native_SendCrossMessageFromChildToParent() public {
        address caller = address(new MockIpcContract());
        address recipient = address(new MockIpcContract());
        address funderAddress = vm.addr(123);
        uint256 amount = 3;

        vm.deal(address(rootNativeSubnetActor), DEFAULT_COLLATERAL_AMOUNT);
        vm.deal(caller, 3);
        vm.deal(funderAddress, 1 ether);

        vm.prank(address(rootNativeSubnetActor));
        registerSubnetGW(DEFAULT_COLLATERAL_AMOUNT, address(rootNativeSubnetActor), rootGateway);

        vm.prank(funderAddress);
        rootGatewayManager.fund{value: 100000}(nativeSubnetName, FvmAddressHelper.from(address(funderAddress)));

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

        callSubmitCheckpointFromParentSubnet(checkpoint, address(rootNativeSubnetActor));

        assertEq(recipient.balance, amount);
    }

    function testMultiSubnet_Token_CallFromChildToParent() public {
        address caller = address(new MockIpcContract());
        address recipient = address(new MockIpcContract());
        address funderAddress = vm.addr(123);
        uint256 amount = 3;

        vm.deal(address(rootTokenSubnetActor), DEFAULT_COLLATERAL_AMOUNT);
        vm.deal(address(token), DEFAULT_COLLATERAL_AMOUNT);
        vm.deal(funderAddress, 1 ether);
        vm.deal(caller, amount);

        // Fund an account in the subnet.
        token.transfer(caller, 100);
        vm.prank(caller);
        token.approve(address(rootGateway), 100);

        vm.prank(address(rootTokenSubnetActor));
        registerSubnetGW(DEFAULT_COLLATERAL_AMOUNT, address(rootTokenSubnetActor), rootGateway);

        vm.prank(caller);
        rootGatewayManager.fundWithToken(tokenSubnetName, FvmAddressHelper.from(address(funderAddress)), 15);

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

        callSubmitCheckpointFromParentSubnet(checkpoint, address(rootTokenSubnetActor));

        assertEq(token.balanceOf(recipient), amount);
    }

    function testMultiSubnet_Native_FundFromParentToChild() public {
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

        callSubmitCheckpointFromParentSubnet(checkpoint, address(rootNativeSubnetActor));

        assertEq(recipient.balance, amount);
    }

    function testMultiSubnet_Native_ReleaseFromChildToParent_DifferentFunderAndSenderInParent() public {
        address caller = address(new MockIpcContract());
        address recipient = address(new MockIpcContractPayable());
        address funderAddress = vm.addr(123);
        uint256 amount = 3;

        vm.deal(address(rootNativeSubnetActor), DEFAULT_COLLATERAL_AMOUNT);
        vm.deal(caller, 3);
        vm.deal(funderAddress, 1 ether);

        GatewayManagerFacet manager = GatewayManagerFacet(address(nativeSubnetGateway));

        vm.prank(address(rootNativeSubnetActor));
        registerSubnetGW(DEFAULT_COLLATERAL_AMOUNT, address(rootNativeSubnetActor), rootGateway);

        vm.prank(funderAddress);
        rootGatewayManager.fund{value: amount}(nativeSubnetName, FvmAddressHelper.from(address(caller)));

        vm.prank(caller);
        manager.release{value: amount}(FvmAddressHelper.from(address(recipient)));

        BottomUpCheckpoint memory checkpoint = callCreateBottomUpCheckpointFromChildSubnet(
            nativeSubnetName,
            address(nativeSubnetGateway)
        );

        callSubmitCheckpointFromParentSubnet(checkpoint, address(rootNativeSubnetActor));

        assertEq(recipient.balance, amount);
    }

    function testMultiSubnet_Native_NonPayable_ReleaseFromChildToParent() public {
        address caller = address(new MockIpcContract());
        address recipient = address(new MockIpcContractFallback());
        address funderAddress = vm.addr(123);
        uint256 amount = 3;

        vm.deal(address(rootNativeSubnetActor), DEFAULT_COLLATERAL_AMOUNT);
        vm.deal(caller, 3);
        vm.deal(funderAddress, 1 ether);

        vm.prank(address(rootNativeSubnetActor));
        registerSubnetGW(DEFAULT_COLLATERAL_AMOUNT, address(rootNativeSubnetActor), rootGateway);

        vm.prank(funderAddress);
        rootGatewayManager.fund{value: amount}(nativeSubnetName, FvmAddressHelper.from(address(funderAddress)));

        GatewayManagerFacet manager = GatewayManagerFacet(address(nativeSubnetGateway));
        vm.prank(funderAddress);
        manager.release{value: amount}(FvmAddressHelper.from(address(recipient)));

        BottomUpCheckpoint memory checkpoint = callCreateBottomUpCheckpointFromChildSubnet(
            nativeSubnetName,
            address(nativeSubnetGateway)
        );

        vm.expectRevert();
        callSubmitCheckpointFromParentSubnetRevert(checkpoint, address(rootNativeSubnetActor));
    }

    function testMultiSubnet_Erc20_ReleaseFromChildToParent_Failed() public {
        address caller = address(new MockIpcContract());
        address recipient = vm.addr(156);
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

        callSubmitCheckpointFromParentSubnet(checkpoint, address(rootTokenSubnetActor));

        assertEq(recipient.balance, 0);
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

        callSubmitCheckpointFromParentSubnet(checkpoint, address(rootTokenSubnetActor));

        assertEq(token.balanceOf(recipient), amount);
    }

    function testMultiSubnet_Erc20_FundFromParentToChild() public {
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

        // TODO: commitParentFinality doesn not affect anything in this test.
        commitParentFinality(address(tokenSubnetGateway));

        executeTopDownMsgs(msgs, tokenSubnetName, address(tokenSubnetGateway));

        assertEq(recipient.balance, amount);
    }

    function testMultiSubnet_Erc20NonPayable_FundFromParentToChild() public {
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

    function callSubmitCheckpointFromParentSubnet(BottomUpCheckpoint memory checkpoint, address subnetActor) internal {
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

    function callSubmitCheckpointFromParentSubnetRevert(
        BottomUpCheckpoint memory checkpoint,
        address subnetActor
    ) internal {
        vm.expectRevert();
        callSubmitCheckpointFromParentSubnet(checkpoint, subnetActor);
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


    function testMultiSubnet_Native_FundFromParentToChild_USDCBridge() public {
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



        uint256 transferAmount = 11 gwei;
        USDCMock mockUSDC = new USDCMock();
        subnetTokenBridge = new SubnetTokenBridge(address(nativeSubnetGateway), address(mockUSDC), rootSubnetName  );

        mockUSDC.mint(transferAmount);
        address myAddress = mockUSDC.me(); // todo learn how to get caller address from forge
        assertEq(transferAmount,  mockUSDC.balanceOf(myAddress));
        console.log(transferAmount);

        rootTokenBridge = new TokenTransferAndMint(
            address(rootGateway),
            address(mockUSDC),
            nativeSubnetName,
            address(subnetTokenBridge)
        );

        vm.deal(myAddress, DEFAULT_CROSS_MSG_FEE);
        mockUSDC.approve(address(rootTokenBridge), transferAmount);
        rootTokenBridge.transferAndMint{ value: DEFAULT_CROSS_MSG_FEE }( myAddress, transferAmount);
        //ensure that tokens are delivered on subnet
        address proxyUSDCToken = subnetTokenBridge.getProxyTokenAddress();
        assertEq(IERC20(proxyUSDCToken).balanceOf(myAddress), transferAmount, "incorrect proxy token balance");



        commitParentFinality(address(nativeSubnetGateway));
        executeTopDownMsgs(msgs, nativeSubnetName, address(nativeSubnetGateway));
    }
}
