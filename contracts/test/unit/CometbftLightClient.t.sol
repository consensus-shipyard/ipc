// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity ^0.8.23;

import {Test} from "forge-std/Test.sol";
import {CanonicalVote, Timestamp, SignedHeader, CanonicalBlockID, CanonicalPartSetHeader, BlockID, Commit, PartSetHeader, CommitSig, LightHeader, Consensus, TENDERMINTLIGHT_PROTO_GLOBAL_ENUMS} from "tendermint-sol/proto/TendermintLight.sol";
import {TendermintHelper} from "tendermint-sol/proto/TendermintHelper.sol";

import {CometbftLightClient} from "../../contracts/lib/cometbft/CometbftLightClient.sol";

contract CometbftLightClientTest is Test {
    string public constant CHAIN_ID = "wormhole";

    function sampleHeader() internal pure returns (SignedHeader.Data memory header) {
        header = SignedHeader.Data({
            header: LightHeader.Data({
                version: Consensus.Data({block: 11, app: 0}),
                chain_id: "wormhole",
                height: 28,
                time: Timestamp.Data({Seconds: 1634765002, nanos: 453715295}),
                last_block_id: BlockID.Data({
                    hash: hex"c3d08f2b980d0c513ea5a7401a1e79a6ceafc6c709884918222f9c90320c364a",
                    part_set_header: PartSetHeader.Data({
                        total: 1,
                        hash: hex"9f81d62d43f90eeda259797549fe98224634e5e022ad4968e6c54cf7837a81d0"
                    })
                }),
                last_commit_hash: hex"0ae1e22c39a0f81cef5ea9c248455d9c34cd824fe05c2ded9215a2fbdfb5130c",
                data_hash: hex"e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
                validators_hash: hex"129e8ba93ab9f74557af3d1e180d844cdbd3bb46f48dd623ad0854af7967fef2",
                next_validators_hash: hex"129e8ba93ab9f74557af3d1e180d844cdbd3bb46f48dd623ad0854af7967fef2",
                consensus_hash: hex"048091bc7ddc283f77bfbf91d73c44da58c3df8a9cbc867405d8b7f3daada22f",
                app_hash: hex"6e47f0e9a0d3e607664664ed4ede8b6e19063c6335392ab59cc0400ef54240c3",
                last_results_hash: hex"e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
                evidence_hash: hex"e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
                proposer_address: hex"56e4989e5cca6c4d3b1cd56980f1ae5729453c75"
            }),
            commit: Commit.Data({
                height: 28,
                round: 0,
                block_id: BlockID.Data({
                    hash: hex"7dc230949771d4870f9203c8b4bb63cb77b9845dd15d078ecbd85c49173d8114",
                    part_set_header: PartSetHeader.Data({
                        total: 1,
                        hash: hex"3786e60cf4aa28dc0db490cd96ac38de88069a0effc5f05880b02b0d23fee128"
                    })
                }),
                signatures: new CommitSig.Data[](1)
            })
        });

        header.commit.signatures[0] = CommitSig.Data({
            block_id_flag: TENDERMINTLIGHT_PROTO_GLOBAL_ENUMS.BlockIDFlag.BLOCK_ID_FLAG_COMMIT,
            validator_address: hex"56e4989e5cca6c4d3b1cd56980f1ae5729453c75",
            timestamp: Timestamp.Data({Seconds: 1634765007, nanos: 501429636}),
            signature: hex"ced30df1dc85e4ae9f71ab57075eb6233670871455c6c75761f15696ffebe452452fccbedc3bac6a22f44e5e7f9b850be345e26a743a11038eda4fd77d15e503"
        });
    }

    function test_verifyValidatorsQuorumWorks() public {
        CometbftLightClient client = new CometbftLightClient(CHAIN_ID, address(this));
        client.verifyValidatorsQuorum(sampleHeader());
    }

    function getCurrentPower(address) external view returns (uint256) {
        return 100;
    }

    function getTotalCurrentPower() external view returns (uint256) {
        return 100;
    }
}
