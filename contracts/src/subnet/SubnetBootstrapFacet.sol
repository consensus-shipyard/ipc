// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity ^0.8.23;

import {SubnetNotBootstrapped, AlreadyInitialized} from "../errors/IPCErrors.sol";
import {IGenesisComponent} from "../interfaces/IGenesis.sol";

struct BootstrapTrackerStorage {
    /// @notice Tracks the number of component in the genesis
    IGenesisComponent[] components;
    /// @notice If the bootstrap tracker is initialized
    bool initialized;
}

/// @notice Tracks the facet bootstrap progress. Once all the facets that need to be bootstrapped are bootstrapped, the subnet
///         is then bootstrapped and `SubnetGenesis` will be finalized.
contract SubnetBootstrapFacet {
    function setup(IGenesisComponent[] calldata components) external {
        BootstrapTrackerStorage storage s = diamondStorage();

        if (s.initialized) {
            revert AlreadyInitialized();
        }

        uint256 length = components.length;
        for (uint256 i = 0; i < length; ) {
            s.components.push(components[i]);
        }
    }

    /// @notice Checks if the subnet is bootstrapped
    function bootstrapped() public view returns(bool) {
        BootstrapTrackerStorage storage s = diamondStorage();

        uint256 length = s.components.length;
        for (uint256 i = 0; i < length; ) {
            IGenesisComponent c = s.components[i];

            if (!c.bootstrapped()) {
                return false;
            }

            unchecked {
                i++;
            }
        }

        return true;
    }

    function genesis() external view returns(bytes memory) {
        if (!bootstrapped()) {
            revert SubnetNotBootstrapped();
        }

        // constructs the genesis bytes by scanning all the interfaces that supports `IGenesisComponent`
        revert("todo");
    }

    function diamondStorage() internal pure returns (BootstrapTrackerStorage storage ds) {
        bytes32 position = keccak256("ipc.subnet.genesis.storage");
        assembly {
            ds.slot := position
        }
    }
}
