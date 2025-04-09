// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity ^0.8.23;

import "forge-std/Test.sol";

import "../../contracts/errors/IPCErrors.sol";
import {IpcEnvelope, TopdownCheckpoint} from "../../contracts/structs/CrossNet.sol";
import {FvmAddress} from "../../contracts/structs/FvmAddress.sol";
import {SubnetID, Subnet, IPCAddress, Validator, PowerChange, PowerChangeRequest, PowerOperation} from "../../contracts/structs/Subnet.sol";
import {SubnetIDHelper} from "../../contracts/lib/SubnetIDHelper.sol";
import {FvmAddressHelper} from "../../contracts/lib/FvmAddressHelper.sol";
import {CrossMsgHelper} from "../../contracts/lib/CrossMsgHelper.sol";
import {FilAddress} from "fevmate/contracts/utils/FilAddress.sol";
import {GatewayDiamond} from "../../contracts/GatewayDiamond.sol";

import {TestUtils} from "../helpers/TestUtils.sol";
import {IntegrationTestBase, SubnetWithNativeTokenMock} from "../IntegrationTestBase.sol";
import {GatewayFacetsHelper} from "../helpers/GatewayFacetsHelper.sol";

contract GatewayTopdownVoting is Test, IntegrationTestBase, SubnetWithNativeTokenMock {
    using SubnetIDHelper for SubnetID;
    using CrossMsgHelper for IpcEnvelope;
    using GatewayFacetsHelper for GatewayDiamond;

    function setUp() public override {
        super.setUp();
    }

    function testTopdownVoting_works() public {
        Validator[] memory validators = createSubnet(3);

        uint64 blockHeight = 10;
        bytes32 blockHash = bytes32(uint256(100));
        
        (uint64 heightCommitted, ) = gatewayDiamond.topDownVoting().latestCommitted();
        require(heightCommitted == 0, "height should be 0");

        TopdownCheckpoint memory checkpoint = dummyCheckpoint(blockHeight, blockHash);
        bytes32 voteHash = keccak256(abi.encode(checkpoint));
        uint256 voteWeight = 0; 

        vm.prank(validators[0].addr);
        gatewayDiamond.topDownVoting().propose(checkpoint);
        require(gatewayDiamond.topDownVoting().hasVoted(validators[0].addr), "validator 1 should have voted");
        voteWeight += validators[0].weight;
        require(gatewayDiamond.topDownVoting().onGoingVoteInfo(voteHash).totalPower == voteWeight, "weight 0 incorrect");

        vm.prank(validators[1].addr);
        gatewayDiamond.topDownVoting().propose(checkpoint);
        require(gatewayDiamond.topDownVoting().hasVoted(validators[1].addr), "validator 2 should have voted");
        voteWeight += validators[1].weight;
        require(gatewayDiamond.topDownVoting().onGoingVoteInfo(voteHash).totalPower == voteWeight, "weight 1 incorrect");

        vm.prank(validators[2].addr);
        gatewayDiamond.topDownVoting().propose(checkpoint);
        voteWeight += validators[2].weight;
        require(gatewayDiamond.topDownVoting().onGoingVoteInfo(voteHash).totalPower == voteWeight, "weight 2 incorrect");

        vm.prank(FilAddress.SYSTEM_ACTOR);
        gatewayDiamond.topDownVoting().execute();

        (heightCommitted, ) = gatewayDiamond.topDownVoting().latestCommitted();
        require(heightCommitted == blockHeight, "height should be committed");

        for (uint256 i = 0; i < validators.length; i++) {
            require(!gatewayDiamond.topDownVoting().hasVoted(validators[i].addr), "validator should not have voted in new round");
        }

        (bytes32[] memory votes, uint256 totalPowerVoted) = gatewayDiamond.topDownVoting().onGoingVotes();
        require(votes.length == 0, "should have no ongoing votes");
        require(totalPowerVoted == 0, "should have 0 total power voted");
    }

    function testTopdownVoting_splitBrain() public {
        Validator[] memory validators = createSubnet(3);

        uint64 blockHeight = 10;
        bytes32 blockHash = bytes32(uint256(100));
        
        (uint64 heightCommitted, ) = gatewayDiamond.topDownVoting().latestCommitted();
        require(heightCommitted == 0, "height should be 0");

        TopdownCheckpoint memory checkpoint;

        for (uint256 i = 0; i < validators.length; i++) {
            checkpoint = dummyCheckpoint(blockHeight + uint64(i), blockHash);

            vm.prank(validators[i].addr);
            gatewayDiamond.topDownVoting().propose(checkpoint);
        }

        vm.prank(FilAddress.SYSTEM_ACTOR);
        gatewayDiamond.topDownVoting().execute();

        (heightCommitted, ) = gatewayDiamond.topDownVoting().latestCommitted();
        require(heightCommitted == 0, "height should still be 0");

        (bytes32[] memory votes, uint256 totalPowerVoted) = gatewayDiamond.topDownVoting().onGoingVotes();
        require(votes.length == 0, "should have no ongoing votes");
        require(totalPowerVoted == 0, "should have 0 total power voted");

        for (uint256 i = 0; i < validators.length; i++) {
            checkpoint = dummyCheckpoint(blockHeight + uint64(i), blockHash);
            bytes32 voteHash = keccak256(abi.encode(checkpoint));
            require(gatewayDiamond.topDownVoting().onGoingVoteInfo(voteHash).totalPower == 0, "should have no weight");

            require(!gatewayDiamond.topDownVoting().hasVoted(validators[i].addr), "validator should not have voted in new round");
        }
    }

    function testTopdownVoting_continuousVoting_works() public {
        Validator[] memory validators = createSubnet(3);

        uint64 blockHeight = 10;
        bytes32 blockHash = bytes32(uint256(100));
        
        (uint64 heightCommitted, ) = gatewayDiamond.topDownVoting().latestCommitted();
        require(heightCommitted == 0, "height should be 0");

        TopdownCheckpoint memory checkpoint = dummyCheckpoint(blockHeight, blockHash);

        for (uint256 i = 0; i < validators.length; i++) {
            vm.prank(validators[i].addr);
            gatewayDiamond.topDownVoting().propose(checkpoint);
        }

        vm.prank(FilAddress.SYSTEM_ACTOR);
        gatewayDiamond.topDownVoting().execute();

        (heightCommitted, ) = gatewayDiamond.topDownVoting().latestCommitted();
        require(heightCommitted == blockHeight, "height should be committed");

        blockHeight = 11;
        blockHash = bytes32(uint256(101));
        checkpoint = dummyCheckpoint(blockHeight, blockHash);

        for (uint256 i = 0; i < validators.length; i++) {
            vm.prank(validators[i].addr);
            gatewayDiamond.topDownVoting().propose(checkpoint);
        }
        
        vm.prank(FilAddress.SYSTEM_ACTOR);
        gatewayDiamond.topDownVoting().execute();

        (heightCommitted, ) = gatewayDiamond.topDownVoting().latestCommitted();
        require(heightCommitted == blockHeight, "height should be committed");
    }

    function testTopdownVoting_notGoingBackwards_works() public {
        Validator[] memory validators = createSubnet(3);

        uint64 blockHeight = 10;
        bytes32 blockHash = bytes32(uint256(100));
        
        (uint64 heightCommitted, ) = gatewayDiamond.topDownVoting().latestCommitted();
        require(heightCommitted == 0, "height should be 0");

        TopdownCheckpoint memory checkpoint = dummyCheckpoint(blockHeight, blockHash);

        for (uint256 i = 0; i < validators.length; i++) {
            vm.prank(validators[i].addr);
            gatewayDiamond.topDownVoting().propose(checkpoint);
        }

        vm.prank(FilAddress.SYSTEM_ACTOR);
        gatewayDiamond.topDownVoting().execute();

        (heightCommitted, ) = gatewayDiamond.topDownVoting().latestCommitted();
        require(heightCommitted == blockHeight, "height should be committed");

        blockHeight = 9;
        blockHash = bytes32(uint256(101));
        checkpoint = dummyCheckpoint(blockHeight, blockHash);


        vm.expectRevert(abi.encodeWithSelector(InvalidTopdownCheckpointHeight.selector, 9, 10));
        vm.prank(validators[0].addr);
        gatewayDiamond.topDownVoting().propose(checkpoint);
    }

    function testTopdownVoting_nonSequentialXnetMsgs_works() public {
        Validator[] memory validators = createSubnet(3);

        uint64 blockHeight = 10;
        bytes32 blockHash = bytes32(uint256(100));
        
        TopdownCheckpoint memory checkpoint = dummyCheckpoint(blockHeight, blockHash);
        checkpoint.xnetMsgs = newListOfMessages(10, 10);

        vm.expectRevert(abi.encodeWithSelector(InvalidTopdownMessageNonce.selector, 0, 10));
        vm.prank(validators[0].addr);
        gatewayDiamond.topDownVoting().propose(checkpoint);
    }

    function testTopdownVoting_nonSequentialConfigNumber_works() public {
        Validator[] memory validators = createSubnet(3);

        uint64 blockHeight = 10;
        bytes32 blockHash = bytes32(uint256(100));

        TopdownCheckpoint memory checkpoint = dummyCheckpoint(blockHeight, blockHash);

        PowerChangeRequest[] memory changes = new PowerChangeRequest[](2);
        changes[0] = PowerChangeRequest({
            configurationNumber: 10,
            change: PowerChange({validator: address(0), op: PowerOperation.SetPower, payload: abi.encode(uint256(0))})
        });
        changes[1] = PowerChangeRequest({
            configurationNumber: 11,
            change: PowerChange({validator: address(1), op: PowerOperation.SetPower, payload: abi.encode(uint256(0))})
        });
        checkpoint.powerChanges = changes;

        vm.expectRevert(abi.encodeWithSelector(InvalidTopdownConfigNumber.selector, 1, 10));
        vm.prank(validators[0].addr);
        gatewayDiamond.topDownVoting().propose(checkpoint);
    }

    function dummyCheckpoint(uint64 blockHeight, bytes32 blockHash) internal pure returns (TopdownCheckpoint memory cp) {
        cp = TopdownCheckpoint({
            height: blockHeight,
            blockHash: blockHash,
            xnetMsgs: new IpcEnvelope[](0),
            powerChanges: new PowerChangeRequest[](0)
        });
    }

    function newListOfMessages(uint64 size, uint64 startNonce) internal view returns (IpcEnvelope[] memory msgs) {
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
                i + startNonce
            );
        }
    }

    function createSubnet(uint256 numGenesisValidators) internal returns (Validator[] memory genesisValidators) {
        // run custom setup function
        address[] memory path = new address[](1);
        path[0] = ROOTNET_ADDRESS;
        SubnetID memory subnetId = SubnetID(ROOTNET_CHAINID, path);

        // create genesis validators
        uint256 startingPrivateKey = 100;
        genesisValidators = new Validator[](numGenesisValidators);

        for (uint256 i = 0; i < numGenesisValidators; i++) {
            (address validator, , bytes memory publicKey) = TestUtils.newValidator(startingPrivateKey + i);
            genesisValidators[i] = Validator({addr: validator, weight: 100, metadata: publicKey});
        
            vm.deal(genesisValidators[i].addr, 1 ether);
        }

        // now create the child subnet gateway
        gatewayDiamond = createGatewayDiamond(gatewayParams(subnetId, genesisValidators));

    }

    function callback() public view {}
}
