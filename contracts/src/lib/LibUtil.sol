// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity ^0.8.23;

import {NotSystemActor} from "../errors/IPCErrors.sol";
import {AccountHelper} from "../lib/AccountHelper.sol";

/// @notice A collection of util functions.
library LibUtil {
    using AccountHelper for address;

    function enforceSystemActorOnly() internal view {
        if (!msg.sender.isSystemActor()) {
            revert NotSystemActor();
        }
    }

    /// @notice Deduce the next expected bottom up checkpoint epoch given the target block number and checkpoint period
    /// @param blockNumber - the given block number
    /// @param checkPeriod - the checkpoint period
    function nextBottomUpCheckpointEpoch(uint256 blockNumber, uint256 checkPeriod) internal pure returns (uint256) {
        return ((uint64(blockNumber) / checkPeriod) + 1) * checkPeriod;
    }
}