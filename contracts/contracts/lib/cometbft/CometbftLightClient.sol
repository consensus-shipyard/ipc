// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity ^0.8.23;

import {ECDSA} from "@openzeppelin/contracts/utils/cryptography/ECDSA.sol";

import {CanonicalVote} from "tendermint-sol/proto/TendermintLight.sol";
import {Encoder} from "tendermint-sol/proto/Encoder.sol";

import {ISubnetActor} from "../../interfaces/ISubnetActor.sol";
import {DuplicateValidatorSignaturesFound, SignatureAddressesNotSorted} from "../../errors/IPCErrors.sol";

contract CometbftLightClient {
    error NotSameChain();

    bytes32 public chainIDHash;
    ISubnetActor public subnetActor;
    
    constructor(string memory _chainID, address _subnetActor) {
        chainIDHash = keccak256(bytes(_chainID));
        subnetActor = ISubnetActor(_subnetActor);
    }

    /// This method validates the quorum certificate of cometbft pre-commit votes.
    function verifyValidatorsQuorum(address[] calldata validators, bytes[] calldata signatures, CanonicalVote.Data[] calldata preCommitVotes) external view returns(bool) {
        checkSigners(validators);

        uint256 totalPower = 0;
        uint256 powerSoFar = 0;

        bytes32 _chainIDHash = chainIDHash;

        for (uint256 i = 0; i < preCommitVotes.length; i++) {
            if (_chainIDHash != keccak256(bytes(preCommitVotes[i].chain_id))) revert NotSameChain();

            // each validator's vote is different due to timestamp
            bytes memory message = Encoder.encodeDelim(CanonicalVote.encode(preCommitVotes[i]));

            if (!isValidSignature(message, signatures[i], validators[i])) return false;

            powerSoFar += subnetActor.getCurrentPower(validators[i]);
        }

        return powerSoFar >= (totalPower * 2 / 3);
    }

    function checkSigners(address[] calldata signatories) internal pure {
        for (uint256 i = 1; i < signatories.length; ) {
            if (signatories[i] < signatories[i - 1]) {
                revert SignatureAddressesNotSorted();
            }
            if (signatories[i] == signatories[i - 1]) {
                revert DuplicateValidatorSignaturesFound();
            }

            unchecked {
                i++;
            }
        }
    }

    function isValidSignature(bytes memory message, bytes calldata signature, address expectedSigner) internal pure returns(bool) {
        (address recovered, ECDSA.RecoverError ecdsaErr, ) = ECDSA.tryRecover({
            hash: keccak256(message),
            signature: signature
        });
        if (ecdsaErr != ECDSA.RecoverError.NoError) {
            return false;
        }
        if (recovered != expectedSigner) {
            return false;
        }

        return true;
    }
}