// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity 0.8.19;
import {SubnetRegistryActorStorage} from "../lib/LibSubnetRegistryStorage.sol";
import {CannotFindSubnet, FacetCannotBeZero} from "../errors/IPCErrors.sol";
import {LibDiamond} from "../lib/LibDiamond.sol";

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

    /// @notice Returns the last nonce used by the owner
    function getUserLastNonce(address user) external view returns (uint64 nonce) {
        nonce = s.userNonces[user];
    }

    /// @notice Returns the gateway
    function getGateway() external view returns (address) {
        return s.GATEWAY;
    }

    /// @notice Returns the address of the SUBNET_GETTER_FACET
    function getSubnetActorGetterFacet() external view returns (address) {
        return s.SUBNET_GETTER_FACET;
    }

    /// @notice Returns the address of the SUBNET_MANAGER_FACET
    function getSubnetActorManagerFacet() external view returns (address) {
        return s.SUBNET_MANAGER_FACET;
    }

    /// @notice Returns the subnet getter selectors
    function getSubnetActorGetterSelectors() external view returns (bytes4[] memory) {
        return s.subnetGetterSelectors;
    }

    /// @notice Returns the subnet manager selectors
    function getSubnetActorManagerSelectors() external view returns (bytes4[] memory) {
        return s.subnetManagerSelectors;
    }

    /// @notice Updates references to the subnet contract components
    /// Only callable by the contract owner
    function updateReferenceSubnetContract(
        address newGetterFacet,
        address newManagerFacet,
        bytes4[] calldata newSubnetGetterSelectors,
        bytes4[] calldata newSubnetManagerSelectors
    ) external {
        LibDiamond.enforceIsContractOwner();

        // Validate addresses are not zero
        if (newGetterFacet == address(0)) {
            revert FacetCannotBeZero();
        }
        if (newManagerFacet == address(0)) {
            revert FacetCannotBeZero();
        }

        // Update the storage variables
        s.SUBNET_GETTER_FACET = newGetterFacet;
        s.SUBNET_MANAGER_FACET = newManagerFacet;
        s.subnetGetterSelectors = newSubnetGetterSelectors;
        s.subnetManagerSelectors = newSubnetManagerSelectors;
    }
}
