// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity ^0.8.23;

import {SafeCast} from "@openzeppelin/contracts/utils/math/SafeCast.sol";
import {ECDSA} from "@openzeppelin/contracts/utils/cryptography/ECDSA.sol";

import {CanonicalVote, Vote, CommitSig, Commit, SignedHeader, TENDERMINTLIGHT_PROTO_GLOBAL_ENUMS} from "tendermint-sol/proto/TendermintLight.sol";
import {Encoder} from "tendermint-sol/proto/Encoder.sol";
import {TendermintHelper} from "tendermint-sol/proto/TendermintHelper.sol";

import {ISubnetActor} from "../../interfaces/ISubnetActor.sol";
import {DuplicateValidatorSignaturesFound, SignatureAddressesNotSorted} from "../../errors/IPCErrors.sol";

contract CometbftLightClient {
    using TendermintHelper for SignedHeader.Data;
    using TendermintHelper for Vote.Data;

    error NotSameChain();
    error InvalidCommitHash(bytes32 expected, bytes32 actual);

    string public chainID;
    ISubnetActor public subnetActor;

    constructor(string memory _chainID, address _subnetActor) {
        chainID = _chainID;
        subnetActor = ISubnetActor(_subnetActor);
    }

    /// This method validates the quorum certificate of cometbft pre-commit votes.
    function verifyValidatorsQuorum(SignedHeader.Data calldata header) external view returns(bool) {
        checkCommitHash(header);

        uint256 totalPower = 0;
        uint256 powerSoFar = 0;

        CommitSig.Data calldata commitSig;
        string memory _chainID = chainID;

        for (uint256 i = 0; i < header.commit.signatures.length; i++) {
            commitSig = header.commit.signatures[i];
            // no need to verify absent or nil votes.
            if (commitSig.block_id_flag != TENDERMINTLIGHT_PROTO_GLOBAL_ENUMS.BlockIDFlag.BLOCK_ID_FLAG_COMMIT) {
                continue;
            }

            address validator = toAddress(commitSig.validator_address);
            bytes memory message = voteSignBytesDelim(header.commit, _chainID, i);

            if (!isValidSignature(message, commitSig.signature, validator)) return false;

            powerSoFar += subnetActor.getCurrentPower(validator);
        }

        return powerSoFar >= (totalPower * 2 / 3);
    }

    function checkCommitHash(SignedHeader.Data calldata header) internal pure {
        bytes32 expected = header.hash();
        bytes32 actual = toBytes32(header.commit.block_id.hash);
        if (actual != expected) revert InvalidCommitHash(expected, actual);
    }

    function isValidSignature(bytes memory message, bytes calldata signature, address validator) internal pure returns(bool) {
        (address recovered, ECDSA.RecoverError ecdsaErr, ) = ECDSA.tryRecover({
            hash: keccak256(message),
            signature: signature
        });
        if (ecdsaErr != ECDSA.RecoverError.NoError) {
            return false;
        }
        if (recovered != validator) {
            return false;
        }

        return true;
    }

    function toBytes32(bytes memory bz) internal pure returns (bytes32 ret) {
        require(bz.length == 32, "Bytes: toBytes32 invalid size");
        assembly {
            ret := mload(add(bz, 32))
        }
    }

    function toAddress(bytes memory b) public pure returns (address addr) {
        require(b.length == 20, "Invalid address length");
        assembly {
            addr := mload(add(b, 20))
        }
    }
    function voteSignBytes(
        Commit.Data calldata commit,
        string memory _chainID,
        uint256 idx
    ) internal pure returns (bytes memory) {
        Vote.Data memory vote;
        vote = toVote(commit, idx);

        return (CanonicalVote.encode(vote.toCanonicalVote(_chainID)));
    }

    function voteSignBytesDelim(
        Commit.Data calldata commit,
        string memory _chainID,
        uint256 idx
    ) internal pure returns (bytes memory) {
        return Encoder.encodeDelim(voteSignBytes(commit, _chainID, idx));
    }

    function toVote(Commit.Data calldata commit, uint256 valIdx) internal pure returns (Vote.Data memory) {
        CommitSig.Data memory commitSig = commit.signatures[valIdx];

        return
            Vote.Data({
                Type: TENDERMINTLIGHT_PROTO_GLOBAL_ENUMS.SignedMsgType.SIGNED_MSG_TYPE_PRECOMMIT,
                height: commit.height,
                round: commit.round,
                block_id: commit.block_id,
                timestamp: commitSig.timestamp,
                validator_address: commitSig.validator_address,
                validator_index: SafeCast.toInt32(int256(valIdx)),
                signature: commitSig.signature
            });
    }
}