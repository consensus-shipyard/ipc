// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity ^0.8.23;

import {SafeCast} from "@openzeppelin/contracts/utils/math/SafeCast.sol";
import {ECDSA} from "@openzeppelin/contracts/utils/cryptography/ECDSA.sol";

import {CanonicalVote, CanonicalBlockID, Timestamp, CanonicalPartSetHeader, Vote, CommitSig, Commit, SignedHeader, TENDERMINTLIGHT_PROTO_GLOBAL_ENUMS} from "tendermint-sol/proto/TendermintLight.sol";
import {Encoder} from "tendermint-sol/proto/Encoder.sol";
import {TendermintHelper} from "tendermint-sol/proto/TendermintHelper.sol";
import {Bytes} from "tendermint-sol/utils/Bytes.sol";

import {ISubnetActor} from "../../interfaces/ISubnetActor.sol";
import {ValidatorInfo} from "../../structs/Subnet.sol";
import {BottomUpBatch} from "../../structs/BottomUpBatch.sol";
import {LibPower} from "../LibPower.sol";
import {LibSubnetActorStorage, SubnetActorStorage} from "../LibSubnetActorStorage.sol";
import {DuplicateValidatorSignaturesFound, SignatureAddressesNotSorted} from "../../errors/IPCErrors.sol";

/// Breakdown how the app hash is generated
struct StateCommitmentBreakDown {
    bytes stateRoot; // fvm state root
    BottomUpBatch.Commitment msgBatchCommitment;
    uint64 validatorNextConfigurationNumber;
    bytes32 activityCommitment;
}

library CometbftLightClient {
    using TendermintHelper for SignedHeader.Data;
    using TendermintHelper for Vote.Data;

    error InvalidLength(string what, uint256 expected, uint256 actual);
    error NotSameChain();
    error NoQuorumFormed();
    error NoValidatorInQuoum();
    error CometbftSignerNotValidator(bytes20 expected, bytes20 incoming);
    error InvalidCommitHash(bytes32 expected, bytes32 actual);
    error InvalidSignature(bytes32 message, bytes signature, address validator, ECDSA.RecoverError err);
    error NotSigner(bytes32 message, bytes signature, address recovered, address expected);

    /// @notice Validates the quorum certificate of CometBFT pre-commit votes
    /// @dev Verifies that signatures meet BFT consensus requirements (>2/3 voting power)
    ///
    /// @param header The signed header containing the block header and commit with signatures
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
    function verifyValidatorsQuorum(SignedHeader.Data memory header) internal view {
        checkCommitHash(header);

        uint256 totalPower = LibPower.getTotalCurrentPower();
        if (totalPower == 0) {
            revert NoValidatorInQuoum();
        }

        uint256 powerSoFar = 0;

        CommitSig.Data memory commitSig;
        
        for (uint256 i = 0; i < header.commit.signatures.length; i++) {
            commitSig = header.commit.signatures[i];
            // no need to verify absent or nil votes.
            if (commitSig.block_id_flag != TENDERMINTLIGHT_PROTO_GLOBAL_ENUMS.BlockIDFlag.BLOCK_ID_FLAG_COMMIT) {
                continue;
            }

            (uint256 power, address validator) = ensureValidatorSubmission(i, commitSig.validator_address);

            bytes memory message = generateSignePayload(header.commit, LibSubnetActorStorage.appStorage().chainID, i);
            bytes32 messageHash = sha256(message);

            ensureValidSignature(messageHash, commitSig.signature, validator);

            powerSoFar += power;
        }

        if(powerSoFar <= (totalPower * 2 / 3)) {
            revert NoQuorumFormed();
        }
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
    function checkCommitHash(SignedHeader.Data memory header) internal pure {
        bytes32 expected = header.hash();
        bytes32 actual = toBytes32(header.commit.block_id.hash);
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

    /// @notice Validates that a commit signature comes from an expected validator
    /// @dev Ensures the CometBFT validator address matches the expected validator at the given index
    ///
    /// @param i The index of the validator in the active validator set
    /// @param incomingValidator The CometBFT validator address from the commit signature (20 bytes)
    ///
    /// @return power The voting power of the validated validator
    /// @return validator The Ethereum address of the validator
    ///
    /// Process:
    /// 1. Retrieves the validator at index i from the active validator set
    /// 2. Gets the validator's metadata containing their public key
    /// 3. Derives the expected CometBFT address from the public key
    /// 4. Compares the incoming validator address with the expected one
    /// 5. Returns the validator's power and address if valid
    function ensureValidatorSubmission(uint256 i, bytes memory incomingValidator) internal view returns (uint256 power, address validator) {
        validator = LibPower.getActiveValidatorAddressByIndex(uint16(i));
        ValidatorInfo memory info = LibPower.getActiveValidatorInfo(validator);

        bytes20 expectedCometbftAccountId = toCometBFTAddress(info.metadata);

        bytes20 incoming;
        assembly {
            // mload(add(b, 32)) loads the first 32 bytes of the actual data (after skipping the 32-byte length prefix).
            // first 12 bytes discarded due to bytes20.
            incoming := mload(add(incomingValidator, 32))
        }

        // cometbft account id is different than ethreum address, make sure the format matches
        if (incoming != expectedCometbftAccountId) {
            revert CometbftSignerNotValidator(expectedCometbftAccountId, incoming);
        }

        power = info.currentPower;
    }

    /// Generate the signed message payload to be validated
    function generateSignePayload(
        Commit.Data memory commit,
        string memory _chainID,
        uint256 idx
    ) internal pure returns (bytes memory) {
        CanonicalVote.Data memory vote = CanonicalVote.Data({
            Type: TENDERMINTLIGHT_PROTO_GLOBAL_ENUMS.SignedMsgType.SIGNED_MSG_TYPE_PRECOMMIT,
            height: commit.height,
            round: int64(commit.round),
            block_id: TendermintHelper.toCanonicalBlockID(commit.block_id),
            timestamp: commit.signatures[idx].timestamp,
            chain_id: _chainID
        });

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

    function toCompressedPubkey(bytes memory uncompressed) public pure returns (bytes memory) {
        if (uncompressed.length != 65) {
            revert InvalidLength("pubkey", 65, uncompressed.length);
        }

        // ignore prefix
        bytes memory compressed = new bytes(33);
        bytes1 prefix = uint8(uncompressed[64]) % 2 == 0 ? bytes1(0x02) : bytes1(0x03); // Y even?
        compressed[0] = prefix;

        for (uint256 i = 0; i < 32; i++) {
            compressed[i + 1] = uncompressed[i + 1]; // Copy X bytes
        }

        return compressed;
    }

    function toCometBFTAddress(bytes memory uncompressedPubkey) public pure returns (bytes20) {
        bytes memory compressed = toCompressedPubkey(uncompressedPubkey);
        return ripemd160(abi.encodePacked(sha256(compressed)));
    }
}
