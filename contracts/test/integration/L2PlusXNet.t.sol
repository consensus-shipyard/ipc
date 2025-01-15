// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity ^0.8.23;

import "forge-std/Test.sol";
import "../../contracts/errors/IPCErrors.sol";
import {IpcEnvelope, BottomUpMsgBatch, BottomUpCheckpoint, ParentFinality, IpcMsgKind, ResultMsg} from "../../contracts/structs/CrossNet.sol";
import {SubnetID, Subnet, IPCAddress, Validator, FvmAddress} from "../../contracts/structs/Subnet.sol";
import {SubnetIDHelper} from "../../contracts/lib/SubnetIDHelper.sol";
import {AssetHelper} from "../../contracts/lib/AssetHelper.sol";
import {Asset, AssetKind} from "../../contracts/structs/Subnet.sol";
import {FvmAddressHelper} from "../../contracts/lib/FvmAddressHelper.sol";
import {CrossMsgHelper} from "../../contracts/lib/CrossMsgHelper.sol";
import {GatewayDiamond} from "../../contracts/GatewayDiamond.sol";
import {SubnetActorDiamond} from "../../contracts/SubnetActorDiamond.sol";
import {SubnetActorManagerFacet} from "../../contracts/subnet/SubnetActorManagerFacet.sol";
import {SubnetActorCheckpointingFacet} from "../../contracts/subnet/SubnetActorCheckpointingFacet.sol";
import {GatewayGetterFacet} from "../../contracts/gateway/GatewayGetterFacet.sol";
import {LibGateway} from "../../contracts/lib/LibGateway.sol";
import {TopDownFinalityFacet} from "../../contracts/gateway/router/TopDownFinalityFacet.sol";
import {CheckpointingFacet} from "../../contracts/gateway/router/CheckpointingFacet.sol";
import {XnetMessagingFacet} from "../../contracts/gateway/router/XnetMessagingFacet.sol";
import {DiamondCutFacet} from "../../contracts/diamond/DiamondCutFacet.sol";
import {GatewayMessengerFacet} from "../../contracts/gateway/GatewayMessengerFacet.sol";
import {IntegrationTestBase, RootSubnetDefinition, TestSubnetDefinition} from "../IntegrationTestBase.sol";
import {TestUtils, MockIpcContract, MockIpcContractPayable, MockIpcContractResult} from "../helpers/TestUtils.sol";
import {FilAddress} from "fevmate/contracts/utils/FilAddress.sol";
import {MerkleTreeHelper} from "../helpers/MerkleTreeHelper.sol";
import {GatewayFacetsHelper} from "../helpers/GatewayFacetsHelper.sol";
import {SubnetActorFacetsHelper} from "../helpers/SubnetActorFacetsHelper.sol";

import {IERC20} from "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import {ERC20PresetFixedSupply} from "../helpers/ERC20PresetFixedSupply.sol";

import {ActivityHelper} from "../helpers/ActivityHelper.sol";
import {EnumerableSet} from "@openzeppelin/contracts/utils/structs/EnumerableSet.sol";
import {ISubnetActor} from "../../contracts/interfaces/ISubnetActor.sol";
import {IPCMsgType} from "../../contracts/enums/IPCMsgType.sol";
import {IIpcHandler} from "../../sdk/interfaces/IIpcHandler.sol";
import {IGateway} from "../../contracts/interfaces/IGateway.sol";

import "forge-std/console.sol";

struct SubnetsHierarchy {
    /// @dev The lookup key to l1 SubnetNode
    bytes32 l1NodeLookupKey;
    /// @dev The lookup keys to the children and beyond
    EnumerableSet.Bytes32Set subnetKeys;
    mapping(address => bytes32) actorToLookupKeys;
    mapping(bytes32 => SubnetNode) nodes;
    mapping(bytes32 => SubnetDefinition) definitions;
}

struct SubnetDefinition {
    address gatewayAddr;
    SubnetID id;
}

struct SubnetNode {
    /// @dev The list of child subnet actor address
    EnumerableSet.AddressSet childActors;
}

struct SubnetCreationParams {
    SubnetID parent;
    Asset supplySource;
}

library LibSubnetsHierarchy {
    using EnumerableSet for EnumerableSet.AddressSet;
    using SubnetIDHelper for SubnetID;

    function initL1(SubnetsHierarchy storage self, SubnetDefinition memory l1) internal {
        bytes32 lookupId = l1.id.toHash();

        self.l1NodeLookupKey = lookupId;
        storeNewNode(self, lookupId, l1.id, l1.gatewayAddr);
    }

    function getSubnetGateway(SubnetsHierarchy storage self, SubnetID memory id) internal view returns (address) {
        bytes32 lookupId = id.toHash();
        return getSubnetGateway(self, lookupId);
    }

    function getSubnetGateway(SubnetsHierarchy storage self, bytes32 lookupId) internal view returns (address) {
        require(self.definitions[lookupId].gatewayAddr != address(0), "subnet not found");
        return self.definitions[lookupId].gatewayAddr;
    }

    function storeNewNode(
        SubnetsHierarchy storage self,
        bytes32 lookupId,
        SubnetID memory id,
        address gateway
    ) internal {
        self.definitions[lookupId].gatewayAddr = gateway;
        self.definitions[lookupId].id = id;
    }

    function linkNewChild(SubnetsHierarchy storage self, bytes32 lookupId, address subnetActor) internal {
        self.nodes[lookupId].childActors.add(subnetActor);
    }

    function registerNewSubnet(
        SubnetsHierarchy storage self,
        SubnetID memory parent,
        address subnetActor,
        address gateway
    ) internal returns (SubnetID memory) {
        // ensure parent exists
        getSubnetGateway(self, parent);

        SubnetID memory child = SubnetIDHelper.createSubnetId(parent, subnetActor);

        bytes32 lookupId = child.toHash();

        self.actorToLookupKeys[subnetActor] = lookupId;
        storeNewNode(self, lookupId, child, gateway);
        linkNewChild(self, lookupId, subnetActor);

        return child;
    }

    function getGatewayBySubnetActor(
        SubnetsHierarchy storage self,
        address subnetActor
    ) internal view returns (address) {
        bytes32 lookupId = self.actorToLookupKeys[subnetActor];
        return getSubnetGateway(self, lookupId);
    }
}

contract L2PlusSubnetTest is Test, IntegrationTestBase, IIpcHandler {
    using SubnetIDHelper for SubnetID;
    using CrossMsgHelper for IpcEnvelope;
    using GatewayFacetsHelper for GatewayDiamond;
    using SubnetActorFacetsHelper for SubnetActorDiamond;
    using AssetHelper for Asset;
    using FvmAddressHelper for FvmAddress;

    using LibSubnetsHierarchy for SubnetsHierarchy;

    SubnetsHierarchy private subnets;
    bytes32 private new_topdown_message_topic;
    /// @dev The latest result message received from sending a cross network message
    bytes private latestResultMessage;

    function setUp() public override {
        // there seems no auto way to derive the abi in string, check this before running tests, make sure
        // it's updated with the implementation
        new_topdown_message_topic = keccak256(
            "NewTopDownMessage(address,(uint8,uint64,uint64,uint256,((uint64,address[]),(uint8,bytes)),((uint64,address[]),(uint8,bytes)),bytes),bytes32)"
        );

        // get some free money.
        vm.deal(address(this), 10 ether);
    }

    // ======= implements ipc handler =======

    /* solhint-disable-next-line unused-vars */
    function handleIpcMessage(IpcEnvelope calldata envelope) external payable returns (bytes memory ret) {
        if (envelope.kind == IpcMsgKind.Result) {
            latestResultMessage = envelope.message;
        }
        ret = bytes("");
    }

    receive() external payable {}

    // ======= internal util methods ========

    function executeTopdownMessages(IpcEnvelope[] memory msgs, GatewayDiamond gw) internal {
        uint256 mintedTokens;

        for (uint256 i; i < msgs.length; i++) {
            mintedTokens += msgs[i].value;
        }
        console.log("minted tokens in executed top-downs: %d", mintedTokens);

        // The implementation of the function in fendermint is in
        // https://github.com/consensus-shipyard/ipc/blob/main/fendermint/vm/interpreter/contracts/fvm/topdown.rs#L43

        // This emulates minting tokens.
        vm.deal(address(gw), mintedTokens);

        XnetMessagingFacet xnetMessenger = gw.xnetMessenger();

        vm.prank(FilAddress.SYSTEM_ACTOR);
        xnetMessenger.applyCrossMessages(msgs);
    }

    function createBottomUpCheckpoint(
        SubnetID memory subnet,
        GatewayDiamond gw
    ) internal returns (BottomUpCheckpoint memory checkpoint) {
        uint256 e = getNextEpoch(block.number, DEFAULT_CHECKPOINT_PERIOD);

        GatewayGetterFacet getter = gw.getter();
        CheckpointingFacet checkpointer = gw.checkpointer();

        BottomUpMsgBatch memory batch = getter.bottomUpMsgBatch(e);

        (, address[] memory addrs, uint256[] memory weights) = TestUtils.getFourValidators(vm);

        (bytes32 membershipRoot, ) = MerkleTreeHelper.createMerkleProofsForValidators(addrs, weights);

        checkpoint = BottomUpCheckpoint({
            subnetID: subnet,
            blockHeight: batch.blockHeight,
            blockHash: keccak256("block1"),
            nextConfigurationNumber: 0,
            msgs: batch.msgs,
            activity: ActivityHelper.newCompressedActivityRollup(1, 3, bytes32(uint256(0)))
        });

        vm.prank(FilAddress.SYSTEM_ACTOR);
        checkpointer.createBottomUpCheckpoint(
            checkpoint,
            membershipRoot,
            weights[0] + weights[1] + weights[2],
            ActivityHelper.dummyActivityRollup()
        );

        return checkpoint;
    }

    function prepareValidatorsSignatures(
        BottomUpCheckpoint memory checkpoint,
        SubnetActorDiamond sa
    ) internal returns (address[] memory, bytes[] memory) {
        (uint256[] memory parentKeys, address[] memory parentValidators, ) = TestUtils.getThreeValidators(vm);
        bytes[] memory parentPubKeys = new bytes[](3);
        bytes[] memory parentSignatures = new bytes[](3);

        SubnetActorManagerFacet manager = sa.manager();

        for (uint256 i = 0; i < 3; i++) {
            vm.deal(parentValidators[i], 10 gwei);
            parentPubKeys[i] = TestUtils.deriveValidatorPubKeyBytes(parentKeys[i]);
            vm.prank(parentValidators[i]);
            manager.join{value: 10}(parentPubKeys[i], 10);
        }

        bytes32 hash = keccak256(abi.encode(checkpoint));

        for (uint256 i = 0; i < 3; i++) {
            (uint8 v, bytes32 r, bytes32 s) = vm.sign(parentKeys[i], hash);
            parentSignatures[i] = abi.encodePacked(r, s, v);
        }

        return (parentValidators, parentSignatures);
    }

    function submitBottomUpCheckpoint(BottomUpCheckpoint memory checkpoint, SubnetActorDiamond sa) internal {
        (address[] memory parentValidators, bytes[] memory parentSignatures) = prepareValidatorsSignatures(
            checkpoint,
            sa
        );

        SubnetActorCheckpointingFacet checkpointer = sa.checkpointer();

        vm.deal(address(1), 1 ether);
        vm.prank(address(1));
        checkpointer.submitCheckpoint(checkpoint, parentValidators, parentSignatures);
    }

    function getNextEpoch(uint256 blockNumber, uint256 checkPeriod) internal pure returns (uint256) {
        return ((uint64(blockNumber) / checkPeriod) + 1) * checkPeriod;
    }

    function createNativeSubnet(
        address parentGatewayAddress,
        SubnetID memory parentNetworkName
    ) internal returns (TestSubnetDefinition memory) {
        SubnetActorDiamond subnetActor = createSubnetActor(
            defaultSubnetActorParamsWith(parentGatewayAddress, parentNetworkName)
        );

        return createSubnet(parentNetworkName.route, subnetActor);
    }

    function createTokenSubnet(
        address tokenAddress,
        address parentGatewayAddress,
        SubnetID memory parentNetworkName
    ) internal returns (TestSubnetDefinition memory) {
        SubnetActorDiamond subnetActor = createSubnetActor(
            defaultSubnetActorParamsWith(parentGatewayAddress, parentNetworkName, tokenAddress)
        );

        return createSubnet(parentNetworkName.route, subnetActor);
    }

    function createSubnet(
        address[] memory subnetPath,
        SubnetActorDiamond subnetActor
    ) internal returns (TestSubnetDefinition memory) {
        address[] memory newPath = new address[](subnetPath.length + 1);
        for (uint i = 0; i < subnetPath.length; i++) {
            newPath[i] = subnetPath[i];
        }

        newPath[subnetPath.length] = address(subnetActor);

        SubnetID memory subnetName = SubnetID({root: ROOTNET_CHAINID, route: newPath});
        GatewayDiamond subnetGateway = createGatewayDiamond(gatewayParams(subnetName));

        return
            TestSubnetDefinition({
                gateway: subnetGateway,
                gatewayAddr: address(subnetGateway),
                id: subnetName,
                subnetActor: subnetActor,
                subnetActorAddr: address(subnetActor),
                path: newPath
            });
    }

    function call(IpcEnvelope memory xnetMsg) internal {
        IPCMsgType msgType = xnetMsg.applyType(xnetMsg.from.subnetId);

        address gateway = subnets.getSubnetGateway(xnetMsg.from.subnetId);
        vm.prank(xnetMsg.from.rawAddress.extractEvmAddress());

        if (msgType == IPCMsgType.BottomUp) {
            IGateway(gateway).sendContractXnetMessage{value: xnetMsg.value}(xnetMsg);
        } else {
            IGateway(gateway).sendContractXnetMessage(xnetMsg);
        }
    }

    function fundContract(
        SubnetID memory originSubnet,
        SubnetID memory targetSubnet,
        address targetRecipient,
        uint256 amount
    ) internal {
        (bool sameChain, SubnetID memory nextHop) = targetSubnet.down(originSubnet);
        require(sameChain, "not in the same hierachy");

        Asset memory supplySource = ISubnetActor(nextHop.getActor()).supplySource();
        address gateway = subnets.getSubnetGateway(originSubnet);

        vm.prank(address(this));
        supplySource.makeAvailable(gateway, amount);

        // sadly fund/fundWithToken does not support multi hierachy call, need to use xnet msg
        IpcEnvelope memory crossMessage = TestUtils.newXnetCallMsg(
            IPCAddress({subnetId: originSubnet, rawAddress: FvmAddressHelper.from(address(this))}),
            IPCAddress({subnetId: targetSubnet, rawAddress: FvmAddressHelper.from(address(targetRecipient))}),
            amount,
            0 // the nonce, does not matter, should be handled by contract calls
        );

        call(crossMessage);
    }

    function initL1() internal returns (SubnetID memory) {
        SubnetID memory rootNetworkName = SubnetID({root: ROOTNET_CHAINID, route: new address[](0)});
        GatewayDiamond rootGateway = createGatewayDiamond(gatewayParams(rootNetworkName));

        SubnetDefinition memory l1Defi = SubnetDefinition({gatewayAddr: address(rootGateway), id: rootNetworkName});
        subnets.initL1(l1Defi);

        return l1Defi.id;
    }

    function newSubnets(SubnetCreationParams[] memory params) internal returns (SubnetID[] memory subnetIds) {
        uint256 length = params.length;

        subnetIds = new SubnetID[](length);

        for (uint256 i = 0; i < length; i++) {
            address parentGateway = subnets.getSubnetGateway(params[i].parent);

            TestSubnetDefinition memory t;
            if (params[i].supplySource.kind == AssetKind.Native) {
                t = createNativeSubnet(parentGateway, params[i].parent);
            } else {
                t = createTokenSubnet(params[i].supplySource.tokenAddress, parentGateway, params[i].parent);
            }

            // register child subnet to parent gateway
            vm.prank(t.subnetActorAddr);
            registerSubnetGW(0, t.subnetActorAddr, GatewayDiamond(payable(parentGateway)));

            subnetIds[i] = subnets.registerNewSubnet(params[i].parent, t.subnetActorAddr, t.gatewayAddr);
        }
    }

    function propagateUp(SubnetID memory from, SubnetID memory to) internal {
        require(from.commonParent(to).equals(to), "not related subnets");

        bool finished = false;
        while (!finished) {
            SubnetID memory parent = from.getParentSubnet();

            address gateway = subnets.getSubnetGateway(from);
            // this would normally submitted by releayer. It call the subnet actor on the L2 network.
            submitBottomUpCheckpoint(
                createBottomUpCheckpoint(from, GatewayDiamond(payable(gateway))),
                SubnetActorDiamond(payable(from.getActor()))
            );

            from = parent;
            finished = parent.equals(to);
        }
    }

    /// @dev Extracts all the NewTopdownMessage emitted events, down side is that all other events are consumed too.
    function propagateDown() internal {
        while (true) {
            Vm.Log[] memory entries = vm.getRecordedLogs();

            uint256 num = 0;
            for (uint256 i = 0; i < entries.length; i++) {
                if (entries[i].topics[0] == new_topdown_message_topic) {
                    IpcEnvelope[] memory msgs = new IpcEnvelope[](1);

                    (IpcEnvelope memory xnetMsg, ) = abi.decode(entries[i].data, (IpcEnvelope, bytes32));
                    msgs[0] = xnetMsg;

                    address gateway = subnets.getGatewayBySubnetActor(address(uint160(uint256(entries[i].topics[1]))));
                    executeTopdownMessages(msgs, GatewayDiamond(payable(gateway)));

                    num++;
                }
            }

            if (num == 0) {
                break;
            }
        }
    }

    /// @dev checks and ensures the latestResultMessage matches with expected bytes
    function checkResultMessageBytes(bytes memory expected, string memory errorMessage) internal view {
        ResultMsg memory message = abi.decode(latestResultMessage, (ResultMsg));
        require(keccak256(expected) == keccak256(message.ret), errorMessage);
    }

    //--------------------
    // Call flow tests.
    //---------------------

    // testing Native L1 => ERC20 L2 => ERC20 L3, this supply source is not allowed
    function test_N1E2E3_rejects() public {
        SubnetID memory l1SubnetID = initL1();

        address erc20_1 = address(new ERC20PresetFixedSupply("TestToken1", "TT", 21_000_000 ether, address(this)));
        address erc20_2 = address(new ERC20PresetFixedSupply("TestToken2", "TT", 21_000_000 ether, address(this)));

        // define L2s
        SubnetCreationParams[] memory l2Params = new SubnetCreationParams[](1);
        l2Params[0] = SubnetCreationParams({parent: l1SubnetID, supplySource: AssetHelper.erc20(erc20_1)});
        SubnetID[] memory l2SubnetIDs = newSubnets(l2Params);

        // define L3s
        SubnetCreationParams[] memory l3Params = new SubnetCreationParams[](1);
        l3Params[0] = SubnetCreationParams({parent: l2SubnetIDs[0], supplySource: AssetHelper.erc20(erc20_2)});
        SubnetID[] memory l3SubnetIDs = newSubnets(l3Params);

        // create the recipients
        MockIpcContractResult recipientContract = new MockIpcContractResult();

        // initial conditions
        require(address(recipientContract).balance == 0);

        // start recording events, if not called, propagate down will not work
        vm.recordLogs();

        uint256 originalBalance = IERC20(erc20_1).balanceOf(address(this));
        uint256 amount = 0.01 ether;

        // fund the L3 address should throw an error and triggers an result xnet message
        fundContract({
            originSubnet: l1SubnetID,
            targetSubnet: l3SubnetIDs[0],
            targetRecipient: address(recipientContract),
            amount: amount
        });
        propagateDown();

        // funds should now be locked
        require(IERC20(erc20_1).balanceOf(address(this)) == originalBalance - amount, "balance should have droppped");

        // relayer carries the bottom up checkpoint
        propagateUp(l2SubnetIDs[0], l1SubnetID);

        // post xnet message conditions
        uint256 postBalance = IERC20(erc20_1).balanceOf(address(this));
        require(postBalance == originalBalance, "token should be refunded");
        checkResultMessageBytes(
            abi.encodeWithSelector(InvalidXnetMessage.selector, InvalidXnetMessageReason.IncompatibleSupplySource),
            "result should be incompatible supply source"
        );
    }

    // testing Native L1 => ERC20 L2 => Native L3
    function test_N1E2N3_works() public {
        SubnetID memory l1SubnetID = initL1();

        address erc20 = address(new ERC20PresetFixedSupply("TestToken", "TT", 21_000_000 ether, address(this)));

        // define L2s
        SubnetCreationParams[] memory l2Params = new SubnetCreationParams[](1);
        l2Params[0] = SubnetCreationParams({parent: l1SubnetID, supplySource: AssetHelper.erc20(erc20)});
        SubnetID[] memory l2SubnetIDs = newSubnets(l2Params);

        // define L3s
        SubnetCreationParams[] memory l3Params = new SubnetCreationParams[](1);
        l3Params[0] = SubnetCreationParams({parent: l2SubnetIDs[0], supplySource: AssetHelper.native()});
        SubnetID[] memory l3SubnetIDs = newSubnets(l3Params);

        // create the recipients
        MockIpcContractResult recipientContract = new MockIpcContractResult();

        // initial conditions
        require(address(recipientContract).balance == 0);

        // start recording events, if not called, propagate down will not work
        vm.recordLogs();

        // fund the L3 address first
        fundContract({
            originSubnet: l1SubnetID,
            targetSubnet: l3SubnetIDs[0],
            targetRecipient: address(recipientContract),
            amount: 0.01 ether
        });

        propagateDown();

        // post xnet message conditions
        require(address(recipientContract).balance == 0.01 ether);
    }

    // testing Native L3 => ERC20 L2 => Native L1 => ERC20 L2' => Native L3'
    function test_N3E2N1E2N3_works() public {
        SubnetID memory l1SubnetID = initL1();

        address erc20 = address(new ERC20PresetFixedSupply("TestToken", "TT", 21_000_000 ether, address(this)));

        // define L2s
        SubnetCreationParams[] memory l2Params = new SubnetCreationParams[](2);
        l2Params[0] = SubnetCreationParams({parent: l1SubnetID, supplySource: AssetHelper.erc20(erc20)});
        l2Params[1] = SubnetCreationParams({parent: l1SubnetID, supplySource: AssetHelper.erc20(erc20)});
        SubnetID[] memory l2SubnetIDs = newSubnets(l2Params);

        // define L3s
        SubnetCreationParams[] memory l3Params = new SubnetCreationParams[](2);
        l3Params[0] = SubnetCreationParams({parent: l2SubnetIDs[0], supplySource: AssetHelper.native()});
        l3Params[1] = SubnetCreationParams({parent: l2SubnetIDs[1], supplySource: AssetHelper.native()});
        SubnetID[] memory l3SubnetIDs = newSubnets(l3Params);

        // create the sender and recipients
        MockIpcContractResult callerContract = new MockIpcContractResult();
        MockIpcContractResult recipientContract = new MockIpcContractResult();

        // start recording events, if not called, propagate down will not work
        vm.recordLogs();

        uint256 amount = 0.01 ether;

        // fund the L3 address first
        fundContract({
            originSubnet: l1SubnetID,
            targetSubnet: l3SubnetIDs[0],
            targetRecipient: address(callerContract),
            amount: amount
        });
        propagateDown();

        // initial conditions
        require(address(callerContract).balance == amount, "sender initial balance should be 0.01 ether");
        require(address(recipientContract).balance == 0, "recipient initial balance should be 0");

        // the funds now arrives at L3, trigger cross network call
        IpcEnvelope memory crossMessage = TestUtils.newXnetCallMsg(
            IPCAddress({subnetId: l3SubnetIDs[0], rawAddress: FvmAddressHelper.from(address(callerContract))}),
            IPCAddress({subnetId: l3SubnetIDs[1], rawAddress: FvmAddressHelper.from(address(recipientContract))}),
            amount,
            0 // the nonce, does not matter, should be handled by contract calls
        );

        call(crossMessage);

        propagateUp(l3SubnetIDs[0], l1SubnetID);
        propagateDown();

        // post xnet message conditions
        require(address(callerContract).balance == 0, "sender final balance should be 0");
        require(address(recipientContract).balance == amount, "recipient final balance should be 0.01 ether");
        require(
            address(subnets.getSubnetGateway(l3SubnetIDs[1])).balance == 0,
            "L3 gateway final balance should be 0 ether"
        );
        require(
            address(subnets.getSubnetGateway(l3SubnetIDs[0])).balance == 0,
            "L3 gateway final balance should be 0 ether"
        );
    }
}
