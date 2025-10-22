// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity ^0.8.23;

import "../../contracts/errors/IPCErrors.sol";
import {Test} from "forge-std/Test.sol";
import "forge-std/console.sol";
import {TestUtils} from "../helpers/TestUtils.sol";
import {SelectorLibrary} from "../helpers/SelectorLibrary.sol";
import {NumberContractFacetSeven} from "../helpers/contracts/NumberContractFacetSeven.sol";
import {NumberContractFacetEight} from "../helpers/contracts/NumberContractFacetEight.sol";
import {METHOD_SEND} from "../../contracts/constants/Constants.sol";
import {ConsensusType} from "../../contracts/enums/ConsensusType.sol";
import {IpcEnvelope} from "../../contracts/structs/CrossNet.sol";
import {FvmAddress} from "../../contracts/structs/FvmAddress.sol";
import {SubnetID, PermissionMode, IPCAddress, Subnet, Asset, ValidatorInfo, AssetKind, Membership, Validator, PowerOperation, PowerChangeRequest, PowerChange} from "../../contracts/structs/Subnet.sol";
import {IERC165} from "../../contracts/interfaces/IERC165.sol";
import {IGateway} from "../../contracts/interfaces/IGateway.sol";
import {IDiamond} from "../../contracts/interfaces/IDiamond.sol";
import {IDiamondCut} from "../../contracts/interfaces/IDiamondCut.sol";
import {IDiamondLoupe} from "../../contracts/interfaces/IDiamondLoupe.sol";
import {FvmAddressHelper} from "../../contracts/lib/FvmAddressHelper.sol";
import {MultisignatureChecker} from "../../contracts/lib/LibMultisignatureChecker.sol";
import {SubnetIDHelper} from "../../contracts/lib/SubnetIDHelper.sol";
import {GatewayDiamond} from "../../contracts/GatewayDiamond.sol";
import {SubnetActorDiamond, FunctionNotFound} from "../../contracts/SubnetActorDiamond.sol";
import {SubnetActorManagerFacet} from "../../contracts/subnet/SubnetActorManagerFacet.sol";
import {OwnershipFacet} from "../../contracts/OwnershipFacet.sol";
import {SubnetActorGetterFacet} from "../../contracts/subnet/SubnetActorGetterFacet.sol";
import {SubnetActorPauseFacet} from "../../contracts/subnet/SubnetActorPauseFacet.sol";
import {SubnetActorCheckpointFacetMock} from "../mocks/SubnetActorCheckpointFacetMock.sol";
import {SubnetActorRewardFacet} from "../../contracts/subnet/SubnetActorRewardFacet.sol";
import {DiamondCutFacet} from "../../contracts/diamond/DiamondCutFacet.sol";
import {FilAddress} from "fevmate/contracts/utils/FilAddress.sol";
import {LibPower} from "../../contracts/lib/LibPower.sol";
import {LibDiamond} from "../../contracts/lib/LibDiamond.sol";
import {Pausable} from "../../contracts/lib/LibPausable.sol";
import {AssetHelper} from "../../contracts/lib/AssetHelper.sol";

import {IntegrationTestBase} from "../IntegrationTestBase.sol";

import {SubnetActorFacetsHelper} from "../helpers/SubnetActorFacetsHelper.sol";
import {GatewayFacetsHelper} from "../helpers/GatewayFacetsHelper.sol";
import {ERC20PresetFixedSupply} from "../helpers/ERC20PresetFixedSupply.sol";
import {SubnetValidatorGater} from "../../contracts/examples/SubnetValidatorGater.sol";

import {FullActivityRollup, Consensus} from "../../contracts/structs/Activity.sol";
import {BottomUpBatch} from "../../contracts/structs/BottomUpBatch.sol";
import {ValidatorRewarderMap} from "../../contracts/examples/ValidatorRewarderMap.sol";
import {MintingValidatorRewarder} from "../../contracts/examples/MintingValidatorRewarder.sol";
import {MerkleTreeHelper} from "../helpers/MerkleTreeHelper.sol";
import {ActivityHelper} from "../helpers/ActivityHelper.sol";
import {BottomUpBatchHelper} from "../helpers/BottomUpBatchHelper.sol";

import {Timestamp, CanonicalBlockID, CanonicalPartSetHeader, SignedHeader, CanonicalVote, BlockID, Commit, PartSetHeader, CommitSig, LightHeader, Consensus as ConsensusData, TENDERMINTLIGHT_PROTO_GLOBAL_ENUMS} from "tendermint-sol/proto/TendermintLight.sol";

import {BottomUpCheckpoint} from "./util.sol";
import {ValidatorSignPayload, ValidatorCertificate, LibBitMap} from "../../contracts/lib/cometbft/CometbftLightClient.sol";

contract SubnetBottomUpCheckpointTest is Test, IntegrationTestBase {
    using SubnetIDHelper for SubnetID;
    using FilAddress for address;
    using FvmAddressHelper for FvmAddress;
    using SubnetActorFacetsHelper for SubnetActorDiamond;
    using GatewayFacetsHelper for GatewayDiamond;

    address gatewayAddress;

    function setUp() public override {
        super.setUp();

        gatewayAddress = address(gatewayDiamond);
    }

    function testSubnetActorDiamond_submitSignedHeader() public {
        SignedHeader.Data memory header = SignedHeader.Data({
            header: LightHeader.Data({
                version: ConsensusData.Data({block: 11, app: 0}),
                chain_id: "1671263715227509",
                height: 10,
                time: Timestamp.Data({Seconds: 1754924475, nanos: 753738680}),
                last_block_id: BlockID.Data({
                    hash: hex"cdf54989b2af7335f147497cce3462143805ace148e54e87a2478070da92c4ed",
                    part_set_header: PartSetHeader.Data({
                        total: 1,
                        hash: hex"76434337d10b011ab9d18dd8f0c9ccc58a7ccf069e0ace8c07772569489a489e"
                    })
                }),
                last_commit_hash: hex"8da7c3c5ccacf3277b63b0ecbc1897fd57e3f17372c3aeaa7eb366b69855d9a7",
                data_hash: hex"b8b9fe4ec01144702ae02277199c9f5de07e26614bb378fa434cb3410d847551",
                validators_hash: hex"6aa2b4fb8892eb46abe6d5b9b5e7e86a749d1fbd8e355e3a6b5f5426ef3e6790",
                next_validators_hash: hex"6aa2b4fb8892eb46abe6d5b9b5e7e86a749d1fbd8e355e3a6b5f5426ef3e6790",
                consensus_hash: hex"895734b58a6cb41a56bfe448f135d54fa01dc948164ee7e409960f0d8958d42c",
                app_hash: hex"fcbeb04f3c0175e06b8ef9d731476e88f2d37b98bca65b7e983356c92c9c53e9",
                last_results_hash: hex"7e23c5dbd335ecce8cad567dc6bf69373995bd718d63b562b46126a3d6574b95",
                evidence_hash: hex"e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
                proposer_address: hex"905b1c0098887ea9033946de1eab5427c97a82ad"
            }),
            commit: Commit.Data({
                height: 10,
                round: 0,
                block_id: BlockID.Data({
                    hash: hex"910035b9ff5ddd3f2434d198d718cbe1c97b3a484b6799b4106f072070a046ce",
                    part_set_header: PartSetHeader.Data({
                        total: 1,
                        hash: hex"648d2cb39485249faf92acb13db138eac1212579c433a8a2aa6e85e05d69f2bd"
                    })
                }),
                signatures: new CommitSig.Data[](1)
            })
        });

        LightHeader.Data memory lightHeader = header.header;
        CanonicalVote.Data memory voteTemplate;

        voteTemplate.Type = TENDERMINTLIGHT_PROTO_GLOBAL_ENUMS.SignedMsgType.SIGNED_MSG_TYPE_PRECOMMIT;
        voteTemplate.height = header.commit.height;
        voteTemplate.round = int64(header.commit.round);
        voteTemplate.block_id = CanonicalBlockID.Data({
            hash: header.commit.block_id.hash,
            part_set_header: CanonicalPartSetHeader.Data({
                total: header.commit.block_id.part_set_header.total,
                hash: header.commit.block_id.part_set_header.hash
            })
        });
        ValidatorCertificate memory certificate = ValidatorCertificate({
            bitmap: 1,
            signatures: new ValidatorSignPayload[](1)
        });

        certificate.signatures[0] = ValidatorSignPayload({
            timestamp: Timestamp.Data({Seconds: 1754924476, nanos: 811159237}),
            signature: hex"284f7f673bf73a515a8829dd29edc8671094e62d94db5cfa869bb62b4e8b6eff51c44f2662fb6fef1e37239d9a7d14707971feeddd1e9ba87c2ca5bafc1b6d9e"
        });

        address validator = address(0x1A79385eAd0e873FE0C441C034636D3Edf7014cC);
        bytes
            memory pubkey = hex"047efe505fb55f56756514db73ff1e3a8d7fc08f7c5bbc3cbf10d646be71c2593766d6a8785f468ed6701c427d9b2a6a8d8a7d7146bc77a7e7a94c49bbcbd39f7f";

        vm.deal(validator, 11 ether);
        vm.prank(validator);
        saDiamond.manager().join{value: 10 ether}(pubkey, 10 ether);
        saDiamond.checkpointer().submitBottomUpCheckpoint(abi.encode(lightHeader, certificate, voteTemplate));
    }

    function testSubnetActorDiamond_checkBitMap() public {
        // 10000011
        uint256 bitmap = 131;

        require(LibBitMap.isBitSet(bitmap, 0), "0");
        require(LibBitMap.isBitSet(bitmap, 1), "1");
        require(!LibBitMap.isBitSet(bitmap, 2), "2");
        require(!LibBitMap.isBitSet(bitmap, 3), "3");
        require(!LibBitMap.isBitSet(bitmap, 4), "4");
        require(!LibBitMap.isBitSet(bitmap, 5), "5");
        require(!LibBitMap.isBitSet(bitmap, 6), "6");
        require(LibBitMap.isBitSet(bitmap, 7), "7");
    }

    function callback() public view {
    }
}
