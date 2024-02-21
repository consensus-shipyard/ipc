// SPDX-License-Identifier: MIT
pragma solidity 0.8.23;

import {IntegrationTestBase} from "./IntegrationTestBase.sol";

import {SubnetIDHelper} from "../src/lib/SubnetIDHelper.sol";
import {IpcEnvelope, BottomUpMsgBatch, BottomUpCheckpoint, ParentFinality, IpcMsgKind, ResultMsg, CallMsg} from "../src/structs/CrossNet.sol";
import {SubnetID, Subnet, IPCAddress, Validator} from "../src/structs/Subnet.sol";
import {FvmAddressHelper} from "../src/lib/FvmAddressHelper.sol";
import {GatewayDiamond, FEATURE_MULTILEVEL_CROSSMSG} from "../src/GatewayDiamond.sol";
import {SubnetActorManagerFacet} from "../src/subnet/SubnetActorManagerFacet.sol";
import {SubnetActorCheckpointingFacet} from "../src/subnet/SubnetActorCheckpointingFacet.sol";
import {GatewayGetterFacet} from "../src/gateway/GatewayGetterFacet.sol";
import {GatewayManagerFacet} from "../src/gateway/GatewayManagerFacet.sol";
import {TopDownFinalityFacet} from "../src/gateway/router/TopDownFinalityFacet.sol";
import {CheckpointingFacet} from "../src/gateway/router/CheckpointingFacet.sol";
import {XnetMessagingFacet} from "../src/gateway/router/XnetMessagingFacet.sol";
import {CrossMsgHelper} from "../src/lib/CrossMsgHelper.sol";
import {SubnetActorDiamond} from "../src/SubnetActorDiamond.sol";

import {IERC20} from "openzeppelin-contracts/token/ERC20/IERC20.sol";
import {ERC20PresetFixedSupply} from "./helpers/ERC20PresetFixedSupply.sol";

import {TestUtils, MockIpcContract, MockIpcContractPayable, MockIpcContractFallback} from "./helpers/TestUtils.sol";

import {FilAddress} from "fevmate/utils/FilAddress.sol";

import {MerkleTreeHelper} from "./helpers/MerkleTreeHelper.sol";

import "forge-std/console.sol";

abstract contract MultiSubnetTestBase is IntegrationTestBase {
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
        token = new ERC20PresetFixedSupply("TestToken", "TEST", 1 ether, address(this));

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
        console.log("batch length %d", batch.msgs.length);
        require(batch.msgs.length > 0, "batch length incorrect");
        if (batch.msgs.length == 2) {
            printEnvelope(batch.msgs[0]);
            printEnvelope(batch.msgs[1]);
        }

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
        console.log("submitCheckpoint");
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

    //prints any IpcEnvelope for debugging
    function printEnvelope(IpcEnvelope memory envelope) public {
        console.log("\nPrint Envelope");
        console.log("from %s:", envelope.from.subnetId.toString());
        console.log("to %s:", envelope.to.subnetId.toString());
        console.log("Nonce");
        console.log(envelope.nonce);
        console.log("Value");
        console.log(envelope.value);
        console.log("Message");
        console.logBytes(envelope.message);
        console.log("Hash");
        console.logBytes32(envelope.toHash());
        if (envelope.kind == IpcMsgKind.Result) {
            ResultMsg memory result = abi.decode(envelope.message, (ResultMsg));
            console.log("Result id");
            console.logBytes32(result.id);
        } else if (envelope.kind == IpcMsgKind.Call) {
            CallMsg memory call = abi.decode(envelope.message, (CallMsg));
            console.log("Call Msg");
            console.logBytes(call.method);
            console.logBytes(call.params);
        }
    }
}
