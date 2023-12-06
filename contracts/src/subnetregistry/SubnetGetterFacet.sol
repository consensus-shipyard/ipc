// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity 0.8.19;
import {SubnetRegistryActorStorage} from "../lib/LibSubnetRegistryStorage.sol";
import {CannotFindSubnet} from "../errors/IPCErrors.sol";

contract SubnetGetterFacet {
    // slither-disable-next-line uninitialized-state
    SubnetRegistryActorStorage internal s;

    /// @notice Returns the address of the latest subnet actor deployed by a user
    function latestSubnetDeployed(address owner) external view returns (address subnet) {
        uint64 nonce = s.userNonces[owner];
        // need unchecked when nonce == 0 or else will underflow
        unchecked {
            nonce -= 1;
        }

        subnet = s.subnets[owner][nonce];
        if (subnet == address(0)) {
            revert CannotFindSubnet();
        }
    }

    /// @notice Returns the address of a subnet actor deployed for a specific nonce by a user
    function getSubnetDeployedByNonce(address owner, uint64 nonce) external view returns (address subnet) {
        subnet = s.subnets[owner][nonce];
        if (subnet == address(0)) {
            revert CannotFindSubnet();
        }
    }
}
