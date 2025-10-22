// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity ^0.8.23;

import {SafeCast} from "@openzeppelin/contracts/utils/math/SafeCast.sol";
import {ECDSA} from "@openzeppelin/contracts/utils/cryptography/ECDSA.sol";

import {CanonicalVote, Timestamp, Consensus, BlockID, Vote, LightHeader} from "tendermint-sol/proto/TendermintLight.sol";
import {Encoder} from "tendermint-sol/proto/Encoder.sol";
import {MerkleTree} from "tendermint-sol/utils/crypto/MerkleTree.sol";

import {TendermintHelper} from "tendermint-sol/proto/TendermintHelper.sol";
import {Bytes} from "tendermint-sol/utils/Bytes.sol";

import {ISubnetActor} from "../../interfaces/ISubnetActor.sol";
import {ValidatorInfo} from "../../structs/Subnet.sol";
import {BottomUpBatch} from "../../structs/BottomUpBatch.sol";
import {LibPower} from "../LibPower.sol";
import {LibSubnetActorStorage, SubnetActorStorage} from "../LibSubnetActorStorage.sol";
import {DuplicateValidatorSignaturesFound, SignatureAddressesNotSorted} from "../../errors/IPCErrors.sol";
import {CompressedActivityRollup} from "../../structs/Activity.sol";

/// Breakdown how the app hash is generated, it's keccak(abi.encode(AppHashBreakdown))
struct AppHashBreakdown {
    bytes stateRoot; // fvm state root
    BottomUpBatch.Commitment msgBatchCommitment;
    uint64 validatorNextConfigurationNumber;
    CompressedActivityRollup activityCommitment;
}

/// Validator sigature payload from cometbft pre-commit quorum certificate.
/// @dev This struct is used together with vote template for light client verification,
/// see contracts/contracts/subnet/SubnetActorCheckpointingFacet.sol#submitBottomUpCheckpoint method
/// for its usage.
struct ValidatorSignPayload {
    Timestamp.Data timestamp;
    bytes signature;
}

struct ValidatorCertificate {
    uint256 bitmap;
    ValidatorSignPayload[] signatures;
}

library CometbftLightClient {
    using TendermintHelper for Vote.Data;
    using LibBitMap for uint256;

    error InvalidLength(string what, uint256 expected, uint256 actual);
    error NotSameHeight();
    error NoQuorumFormed();
    error NoValidatorInQuorum();
    error CometbftSignerNotValidator(bytes20 expected, bytes20 incoming);
    error InvalidCommitHash(bytes32 expected, bytes32 actual);
    error InvalidSignature(bytes32 message, bytes signature, address validator, ECDSA.RecoverError err);
    error NotSigner(bytes32 message, bytes signature, address recovered, address expected);
    error ValidatorsHashCannotBeEmpty();

    /// @notice Validates the quorum certificate of CometBFT pre-commit votes
    /// @dev Verifies that signatures meet BFT consensus requirements (>2/3 voting power)
    ///
    /// @param header The light client header containing the block header
    /// @param voteTemplate The canonical vote tempalted filled except for the timestamp and chain id. The chain id will be
    /// assigned in the contract while timestamp is take from the signatures. The rest of the fields in voteTemplate should be
    /// the same for all validators.
    /// @param certificate The validator certificate with each signature and signing timestamp
    ///
    /// CRITICAL: Validator signatures in the commit MUST be ordered exactly as validators
    /// are arranged in LibPower's active validator set. The signature at index i must
    /// correspond to the validator at index i in LibPower.getActiveValidatorAddressByIndex(i).
    /// Misaligned ordering will cause validation to fail.
    ///
    /// Process:
    /// 1. Validates commit hash matches the header
    /// 2. Gets total voting power from current validator set
    /// 3. Iterates through commit signatures by index:
    ///    - Skips absent/nil votes (only processes BLOCK_ID_FLAG_COMMIT)
    ///    - Validates signer at index i matches validator at index i in LibPower
    ///    - Constructs vote sign bytes for the specific validator
    ///    - Verifies ECDSA signature validity
    ///    - Accumulates voting power of valid signatures
    /// 4. Ensures accumulated power >= 2/3 of total power
    function verifyValidatorsQuorum(LightHeader.Data memory header, ValidatorCertificate memory certificate, CanonicalVote.Data memory voteTemplate) internal view {
        prepareParams(header, voteTemplate);

        // make sure the vote template block hash matches the header, so that
        // the validators are signing the same light client header hash.
        checkCommitHash(header, toBytes32(voteTemplate.block_id.hash));

        uint256 totalPower = LibPower.getTotalCurrentPower();
        if (totalPower == 0) revert NoValidatorInQuorum();

        uint256 validatorIndex = 0;
        uint256 powerSoFar = 0;
        uint8 totalValidators = uint8(LibPower.totalActiveValidators());

        for (uint8 bitCount = 0; bitCount < totalValidators; bitCount++) {
            if (!certificate.bitmap.isBitSet(bitCount)) continue;

            (uint256 power, address validator) = getValidatorInfo(validatorIndex);

            voteTemplate.timestamp = certificate.signatures[validatorIndex].timestamp;

            bytes32 messageHash = sha256(generateSignedPayload(voteTemplate));
            ensureValidSignature(messageHash, certificate.signatures[validatorIndex].signature, validator);

            powerSoFar += power;
            validatorIndex += 1;
        }

        if(powerSoFar <= (totalPower * 2 / 3)) {
            revert NoQuorumFormed();
        }
    }

    // Some preparation for the parameters, ensures the chain ids are expected and heights are the same
    function prepareParams(LightHeader.Data memory header, CanonicalVote.Data memory voteTemplate) internal view {
        string memory _chainID = LibSubnetActorStorage.appStorage().chainID;

        // slightly more gas efficient than string compare
        header.chain_id = _chainID;
        voteTemplate.chain_id = _chainID;

        if (header.height != voteTemplate.height) revert NotSameHeight();
    }

    /// @notice Verifies that the commit references the correct block
    /// @dev Ensures the block ID in the commit matches the header's hash
    ///
    /// @param header The signed header containing both the block header and commit
    ///
    /// This function validates that:
    /// - The commit is for the same block as the header
    /// - Prevents commits from being used with wrong blocks
    /// - The block_id.hash in the commit equals the computed header hash
    /// - This also makes sure the AppHash is not fabricated
    function checkCommitHash(LightHeader.Data memory header, bytes32 actual) internal pure {
        bytes32 expected = hashLightHeader(header);
        if (actual != expected) revert InvalidCommitHash(expected, actual);
    }

    function ensureValidSignature(bytes32 messageHash, bytes memory signature, address validator) internal pure {
        (address recovered, ECDSA.RecoverError ecdsaErr) = verify(messageHash, signature, validator);

        if (ecdsaErr != ECDSA.RecoverError.NoError) {
            revert InvalidSignature(messageHash, signature, validator, ecdsaErr);
        }
        if (recovered != validator) {
            revert NotSigner(messageHash, signature, recovered, validator);
        }
    }

    function toBytes32(bytes memory bz) internal pure returns (bytes32 ret) {
        if (bz.length != 32) {
            revert InvalidLength("bytes32", 32, bz.length);
        }
        assembly {
            ret := mload(add(bz, 32))
        }
    }

    /// @notice Get the validator address and power at position index i
    ///
    /// @param i The index of the validator in the active validator set
    ///
    /// @return power The voting power of the validated validator
    /// @return validator The Ethereum address of the validator
    function getValidatorInfo(uint256 i) internal view returns (uint256 power, address validator) {
        validator = LibPower.getActiveValidatorAddressByIndex(uint16(i));
        power = LibPower.getCurrentPower(validator);
    }

    /// Converts the CanonicalVote.Data into protobuf encoded data required by conmetbft signature verification.
    function generateSignedPayload(
        CanonicalVote.Data memory vote
    ) internal pure returns (bytes memory) {
        return Encoder.encodeDelim(CanonicalVote.encode(vote));
    }

    /**
     * @dev verifies the secp256k1 signature against the public key and message
     * Tendermint uses RFC6979 and BIP0062 standard, meaning there is no recovery bit ("v" argument) present in the signature.
     * The "v" argument is required by the ecrecover precompile (https://eips.ethereum.org/EIPS/eip-2098) and it can be either 0 or 1.
     *
     * To leverage the ecrecover precompile this method opportunisticly guess the "v" argument. At worst the precompile is called twice,
     * which still might be cheaper than running the verification in EVM bytecode (as solidity lib)
     *
     * See: tendermint/crypto/secp256k1/secp256k1_nocgo.go (Sign, Verify methods)
     */
    function verify(bytes32 messageHash, bytes memory signature, address signer) internal pure returns (address recovered, ECDSA.RecoverError err) {
        (recovered, err) = tryRecover(messageHash, signature, 27);
        if (err == ECDSA.RecoverError.NoError && recovered != signer) {
            (recovered, err) = tryRecover(messageHash, signature, 28);
        }
    }

    /**
     * @dev returns the address that signed the hash.
     * This function flavor forces the "v" parameter instead of trying to derive it from the signature
     *
     * Source: https://github.com/OpenZeppelin/openzeppelin-contracts/blob/master/contracts/utils/cryptography/ECDSA.sol#L56
     */
    function tryRecover(bytes32 hash, bytes memory signature, uint8 v) internal pure returns (address recovered, ECDSA.RecoverError err) {
        if (signature.length == 65 || signature.length == 64) {
            bytes32 r;
            bytes32 s;
            // ecrecover takes the signature parameters, and the only way to get them
            // currently is to use assembly.
            assembly {
                r := mload(add(signature, 0x20))
                s := mload(add(signature, 0x40))
            }

            (recovered, err, ) = ECDSA.tryRecover(hash, v, r, s);
            return (recovered, err);
        } else {
            return (address(0), ECDSA.RecoverError.InvalidSignatureLength);
        }
    }

    /// @dev This method hash LightHeader.Data into a bytes32 hash that is exactly how cometbft go client
    /// does it. This code is taken from tendermint-sol/contracts/proto/TendermintHelper.sol#hash method.
    /// The original method takes SignedHeader.Data as parameter, while this contract requires LightHeader.Data,
    /// immplementations are the same.
    function hashLightHeader(LightHeader.Data memory h) internal pure returns (bytes32) {
        if(h.validators_hash.length == 0) revert ValidatorsHashCannotBeEmpty();

        bytes memory hbz = Consensus.encode(h.version);
        bytes memory pbt = Timestamp.encode(h.time);
        bytes memory bzbi = BlockID.encode(h.last_block_id);

        bytes[14] memory all = [
            hbz,
            Encoder.cdcEncode(h.chain_id),
            Encoder.cdcEncode(h.height),
            pbt,
            bzbi,
            Encoder.cdcEncode(h.last_commit_hash),
            Encoder.cdcEncode(h.data_hash),
            Encoder.cdcEncode(h.validators_hash),
            Encoder.cdcEncode(h.next_validators_hash),
            Encoder.cdcEncode(h.consensus_hash),
            Encoder.cdcEncode(h.app_hash),
            Encoder.cdcEncode(h.last_results_hash),
            Encoder.cdcEncode(h.evidence_hash),
            Encoder.cdcEncode(h.proposer_address)
        ];

        return MerkleTree.merkleRootHash(all, 0, all.length);
    }

}

library LibBitMap {
    function isBitSet(uint256 bitmap, uint8 index) internal pure returns (bool) {
        // Shift 1 left by index positions and AND with bitmap
        return (bitmap & (uint256(1) << index)) != 0;
    }
}