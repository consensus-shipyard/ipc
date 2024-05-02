// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity ^0.8.23;

import {SubnetNotBootstrapped} from "../errors/IPCErrors.sol";
import {ReentrancyGuard} from "../lib/LibReentrancyGuard.sol";
import {Pausable} from "../lib/LibPausable.sol";
import {IGenesisComponent} from "../interfaces/IGenesis.sol";

contract SubnetGenesisFacet is ReentrancyGuard, Pausable {
    struct SubnetGenesisStorage {
        mapping(address => IGenesisComponent) components;
        address[] facets;
    }

    /// @notice Checks if the subnet is bootstrapped
    function bootstrapped() public view returns(bool) {
        revert("todo");
    }

    function genesis() external view returns(bytes memory) {
        if (!bootstrapped()) {
            revert SubnetNotBootstrapped();
        }

        // constructs the genesis bytes by scanning all the interfaces that supports `IGenesisComponent`
        revert("todo");
    }


    function diamondStorage() internal pure returns (SubnetActorStorage storage ds) {
        bytes32 position = keccak256("ipc.subnet.genesis.storage");
        assembly {
            ds.slot := position
        }
    }
}
