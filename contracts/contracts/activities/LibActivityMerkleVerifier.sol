// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity ^0.8.23;

import {SubnetID} from "../structs/Subnet.sol";
import {InvalidProof} from "../errors/IPCErrors.sol";
import {ValidatorSummary} from "./Activity.sol";
import {MerkleProof} from "@openzeppelin/contracts/utils/cryptography/MerkleProof.sol";

/// Verifies the proof to the commitment in subnet activity summary
library LibActivityMerkleVerifier {
    function ensureValidProof(
        bytes32 commitment,
        ValidatorSummary calldata summary,
        bytes32[] calldata proof
    ) internal pure {
        // Constructing leaf: https://github.com/OpenZeppelin/merkle-tree#leaf-hash
        bytes32 leaf = keccak256(
            bytes.concat(
                keccak256(
                    abi.encode(summary.validator, summary.blocksCommitted, summary.metadata)
                )
            )
        );
        bool valid = MerkleProof.verify({proof: proof, root: commitment, leaf: leaf});
        if (!valid) {
            revert InvalidProof();
        }
    }
}
