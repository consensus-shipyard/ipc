// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity 0.8.19;

import "forge-std/Test.sol";
import "../../src/errors/IPCErrors.sol";
import {EMPTY_BYTES, METHOD_SEND} from "../../src/constants/Constants.sol";
import {IpcEnvelope, BottomUpMsgBatch, BottomUpCheckpoint, ParentFinality, CallMsg, IpcMsgKind} from "../../src/structs/CrossNet.sol";
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
import {TokenTransferAndMint} from "../../src/examples/cross-token/TokenTransferAndMint.sol";
import {USDCTest} from "../../src/examples/cross-token/USDCTest.sol";

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

        // TODO: commitParentFinality doesn not affect anything in this test.
        commitParentFinality(address(nativeSubnetGateway));

        executeTopDownMsgs(msgs, nativeSubnetName, address(nativeSubnetGateway));

        assertEq(recipient.balance, amount);
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

    function getNextEpoch(uint256 blockNumber, uint256 checkPeriod) internal pure returns (uint256) {
        return ((uint64(blockNumber) / checkPeriod) + 1) * checkPeriod;
    }

    function printActors() internal view {
        console.log("root actor: %s", rootSubnetName.getActor());
        console.log("root native subnet actor: %s", (address(rootNativeSubnetActor)));
        console.log("root token subnet actor: %s", (address(rootTokenSubnetActor)));
        console.log("root name: %s", rootSubnetName.toString());
        console.log("native subnet name: %s", nativeSubnetName.toString());
        console.log("token subnet name: %s", tokenSubnetName.toString());
        console.log("native subnet getActor(): %s", address(nativeSubnetName.getActor()));
    }

    function printEnvelope(IpcEnvelope memory envelope) internal {
        console.log("from %s:", envelope.from.subnetId.toString());
        console.log("to %s:", envelope.to.subnetId.toString());
    }

    // @dev This test verifies that USDC bridge connects correctly
    // a contract from native subnet with a contract in token subnet through the rootnet.
    function testMultiSubnet_Native_FundFromParentToChild_USDCBridge() public {
        IpcEnvelope[] memory msgs = new IpcEnvelope[](1);

        address caller = address(new MockIpcContractPayable());
        address recipient = address(new MockIpcContractPayable());
        uint256 transferAmount = 11 gwei;

        vm.deal(address(rootNativeSubnetActor), DEFAULT_COLLATERAL_AMOUNT);
        vm.deal(caller, transferAmount);

        console.log("---------------register native subnet---------------");

        vm.prank(address(rootNativeSubnetActor));
        registerSubnetGW(DEFAULT_COLLATERAL_AMOUNT, address(rootNativeSubnetActor), rootGateway);

        console.log("---------------fund native subnet---------------");

        IpcEnvelope memory expected = CrossMsgHelper.createFundMsg(
            nativeSubnetName,
            caller,
            FvmAddressHelper.from(caller),
            transferAmount
        );

        vm.prank(caller);
        vm.expectEmit(true, true, true, true, address(rootGateway));
        emit LibGateway.NewTopDownMessage(address(rootNativeSubnetActor), expected);
        rootGatewayManager.fund{value: transferAmount}(nativeSubnetName, FvmAddressHelper.from(address(caller)));

        console.log("--------------- transfer and mint (bottom-up)---------------");
        // here starts a flow from native subnet to token subnet.

        USDCTest testUSDC = new USDCTest();
        testUSDC.mint(transferAmount);
        address testUSDCOwner = testUSDC.owner();
        assertEq(transferAmount, testUSDC.balanceOf(testUSDCOwner));

        subnetTokenBridge = new SubnetTokenBridge(address(tokenSubnetGateway), address(testUSDC), rootSubnetName);

        rootTokenBridge = new TokenTransferAndMint(
            address(nativeSubnetGateway),
            address(testUSDC),
            tokenSubnetName,
            address(subnetTokenBridge)
        );

        subnetTokenBridge.setParentSubnetUSDC(address(rootTokenBridge));

        vm.deal(testUSDCOwner, DEFAULT_CROSS_MSG_FEE);
        vm.deal(address(rootTokenBridge), 1 ether);

        vm.prank(testUSDCOwner);
        testUSDC.approve(address(rootTokenBridge), transferAmount);

        console.log("mock usdc %s", address(testUSDC));
        console.log("mock usdc owner%s", address(testUSDCOwner));
        console.log("rootTokenBridge %s", address(rootTokenBridge));
        console.log("allowance: %d", testUSDC.allowance(address(testUSDCOwner), address(rootTokenBridge)));

        vm.prank(address(testUSDCOwner));
        rootTokenBridge.depositTokens{value: DEFAULT_CROSS_MSG_FEE}(testUSDCOwner, transferAmount);

        // after the two next calls the root gateway should store the message in its postbox.
        BottomUpCheckpoint memory checkpoint = callCreateBottomUpCheckpointFromChildSubnet(
            nativeSubnetName,
            address(nativeSubnetGateway)
        );

        callSubmitCheckpointFromParentSubnet(checkpoint, address(rootNativeSubnetActor));
        // check the message is in the postbox

        CallMsg memory payload = CallMsg({
            method: abi.encodePacked(bytes4(keccak256("transfer(address,uint256)"))),
            params: abi.encode(address(testUSDCOwner), transferAmount)
        });
        expected = IpcEnvelope({
            kind: IpcMsgKind.Call,
            from: IPCAddress({subnetId: nativeSubnetName, rawAddress: FvmAddressHelper.from(address(rootTokenBridge))}),
            to: IPCAddress({subnetId: tokenSubnetName, rawAddress: FvmAddressHelper.from(address(subnetTokenBridge))}),
            value: DEFAULT_CROSS_MSG_FEE,
            nonce: 0,
            message: abi.encode(payload)
        });

        bytes32 expectedCid = expected.toHash();
        IpcEnvelope memory postboxMsg = rootGatewayGetter.postbox(expectedCid);
        assertEq(expectedCid, postboxMsg.toHash(), "postboxMsg hash and cid not equal");

        console.log("--------------- execute (top-down) ---------------");

        // the message the root gateway's postbox is being executed in the token subnet's gateway
        msgs[0] = expected;
        commitParentFinality(address(tokenSubnetGateway));
        executeTopDownMsgs(msgs, tokenSubnetName, address(tokenSubnetGateway));

        //ensure that tokens are delivered on subnet
        assertEq(
            IERC20(subnetTokenBridge).balanceOf(address(testUSDCOwner)),
            transferAmount,
            "incorrect proxy token balance"
        );

        console.log("--------------- withdraw token (top-down)---------------");

        vm.deal(testUSDCOwner, DEFAULT_CROSS_MSG_FEE);
        vm.prank(address(testUSDCOwner));
        IpcEnvelope memory committed = subnetTokenBridge.withdrawTokens{value: DEFAULT_CROSS_MSG_FEE}(
            testUSDCOwner,
            transferAmount
        );

        /* 
            TODO replace the next two lines with the test utils so that the bottom up message to the rootTokenBridge contract is sent
        */
        vm.prank(address(nativeSubnetGateway));
        rootTokenBridge.handleIpcMessage(committed);

        //ensure that usdc tokens are returned on root net
        assertEq(transferAmount, testUSDC.balanceOf(testUSDCOwner));
        //ensure that the tokens are the subnet are minted and the token bridge and the usdc owner does not own any
        assertEq(0, subnetTokenBridge.balanceOf(testUSDCOwner));
        assertEq(0, subnetTokenBridge.balanceOf(address(subnetTokenBridge)));
    }
}
