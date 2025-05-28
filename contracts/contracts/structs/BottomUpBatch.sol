// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity ^0.8.23;

import {SubnetID} from "../structs/Subnet.sol";
import {IpcEnvelope} from "../structs/CrossNet.sol";

event BottomUpBatchRecorded(uint64 checkpointHeight, IpcEnvelope[] msgs);

library BottomUpBatch {
    type MerkleHash is bytes32;

    /// @notice Represents a commitment to a batch of bottom-up messages sent from a child subnet.
    struct Commitment {
        /// @notice The total number of committed messages.
        uint64 totalNumMsgs;
        /// @notice The messages hash root commitment, so that we don't have to transmit them in full.
        MerkleHash msgsRoot;
    }

    /// @notice Represents a Merkle proof of inclusion for a specific bottom-up message in a batch.
    /// Used to verify that the provided message is part of a previously committed batch.
    struct Inclusion {
        IpcEnvelope msg;
        MerkleHash[] proof;
    }
}
