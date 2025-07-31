// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity ^0.8.23;

// import {Test} from "forge-std/Test.sol";
// import {CanonicalVote, Timestamp, SignedHeader, CanonicalBlockID, CanonicalPartSetHeader, BlockID, Commit, PartSetHeader, CommitSig, LightHeader, Consensus, TENDERMINTLIGHT_PROTO_GLOBAL_ENUMS} from "tendermint-sol/proto/TendermintLight.sol";
// import {TendermintHelper} from "tendermint-sol/proto/TendermintHelper.sol";

// import {ValidatorInfo} from "../../contracts/structs/Subnet.sol";
// import {ISubnetActor} from "../../contracts/interfaces/ISubnetActor.sol";
// import {CometbftLightClient} from "../../contracts/lib/cometbft/CometbftLightClient.sol";

// contract CometbftLightClientMock is CometbftLightClient {
//     function verifyValidatorsQuorumExt(SignedHeader.Data calldata header, string memory chainID, ISubnetActor actor) external view {
//         verifyValidatorsQuorum(header, chainID, actor);
//     }
// }

// contract CometbftLightClientTest is Test {
//     string public constant CHAIN_ID = "62459684645140";

//     function sampleHeader() internal pure returns (SignedHeader.Data memory header) {
//         header = SignedHeader.Data({
//             header: LightHeader.Data({
//                 version: Consensus.Data({block: 11, app: 0}),
//                 chain_id: "62459684645140",
//                 height: 330,
//                 time: Timestamp.Data({Seconds: 1753364219, nanos: 868122203}),
//                 last_block_id: BlockID.Data({
//                     hash: hex"501A9DB0C8B861C3198205CEAEA52B2B85CAFCE3C86B09CD8B59D981CDF2576B",
//                     part_set_header: PartSetHeader.Data({
//                         total: 1,
//                         hash: hex"C1F02F6B3FA3678EE642EC5FFF05559776BBB434A7BA3BC8623E2D455BE1A242"
//                     })
//                 }),
//                 last_commit_hash: hex"C27A966FD9E22E306F5286B81382DB96D02B6FF4012D5FA8547CD41BA2171F92",
//                 data_hash: hex"E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855",
//                 validators_hash: hex"81A012FAA3453DC3F7F6DAACC34BC46DCA4EDC0A695A03188F43C40088C132BF",
//                 next_validators_hash: hex"81A012FAA3453DC3F7F6DAACC34BC46DCA4EDC0A695A03188F43C40088C132BF",
//                 consensus_hash: hex"895734B58A6CB41A56BFE448F135D54FA01DC948164EE7E409960F0D8958D42C",
//                 app_hash: hex"0171A0E40220862ED84A2A33B9DD2494F86D345FCFEBE190829A5EA1107D918CCF08973B48F5",
//                 last_results_hash: hex"E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855",
//                 evidence_hash: hex"E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855",
//                 proposer_address: hex"905B1C0098887EA9033946DE1EAB5427C97A82AD"
//             }),
//             commit: Commit.Data({
//                 height: 330,
//                 round: 0,
//                 block_id: BlockID.Data({
//                     hash: hex"24D7F4042FA6F0DE955DBB5CBB7D357335A563D009C6A8F0389A5F6864ADC241",
//                     part_set_header: PartSetHeader.Data({
//                         total: 1,
//                         hash: hex"47342D68A22653AFD21E2410414A41FD3DEA68FDD9362A4EC43F1774EF95E237"
//                     })
//                 }),
//                 signatures: new CommitSig.Data[](1)
//             })
//         });

//         header.commit.signatures[0] = CommitSig.Data({
//             block_id_flag: TENDERMINTLIGHT_PROTO_GLOBAL_ENUMS.BlockIDFlag.BLOCK_ID_FLAG_COMMIT,
//             validator_address: hex"905B1C0098887EA9033946DE1EAB5427C97A82AD",
//             timestamp: Timestamp.Data({Seconds: 1753364220, nanos: 928048657}),
//             signature: hex"5decc9a8daa8fb6e26bbd0b880d649268b7bb00ebc2b774c742723ced1ee7f134e64cd70ab1a5aa2e4a0cd92110afcfb49c084c406d0e7e4d77e48183d79273a"
//         });
//     }

//     function test_verifyValidatorsQuorumWorks() public {
//         CometbftLightClientMock client = new CometbftLightClientMock(CHAIN_ID, address(this));
//         client.verifyValidatorsQuorum(sampleHeader());
//     }

//     function getCurrentPower(address) external view returns (uint256) {
//         // bytes memory pubkey = hex"";
//         // console.logBytes20(toCometBFTAddress(pubkey));
//         return 100;
//     }

//     /// @notice Obtain the active validator address by its position index in the validator list array.
//     function getActiveValidatorAddressByIndex(uint256 index) external view returns (address) {
//         require(index == 0, "unknown index");
//         return 0x1A79385eAd0e873FE0C441C034636D3Edf7014cC;
//     }

//     function getValidator(address validatorAddress) external view returns (ValidatorInfo memory validator) {
//         require(validatorAddress == 0x1A79385eAd0e873FE0C441C034636D3Edf7014cC, "unknown address");
//         validator = ValidatorInfo({
//             currentPower: 100,
//             // not relevant in this test
//             nextPower: 100,
//             metadata: hex"047efe505fb55f56756514db73ff1e3a8d7fc08f7c5bbc3cbf10d646be71c2593766d6a8785f468ed6701c427d9b2a6a8d8a7d7146bc77a7e7a94c49bbcbd39f7f"
//         });
//     }

//     function getTotalCurrentPower() external view returns (uint256) {
//         return 100;
//     }
// }
