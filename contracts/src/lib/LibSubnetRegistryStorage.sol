// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity 0.8.19;

struct SubnetRegistryActorStorage {
    // solhint-disable-next-line var-name-mixedcase
    address GATEWAY;
    /// The getter and manager facet shared by diamond
    // solhint-disable-next-line var-name-mixedcase
    address SUBNET_GETTER_FACET;
    // solhint-disable-next-line var-name-mixedcase
    address SUBNET_MANAGER_FACET;
    /// The subnet getter facet functions selectors
    bytes4[] subnetGetterSelectors;
    /// The subnet manager facet functions selectors
    bytes4[] subnetManagerSelectors;
    /// @notice Mapping that tracks the deployed subnet actors per user.
    /// Key is the hash of Subnet ID, values are addresses.
    /// mapping owner => nonce => subnet
    mapping(address => mapping(uint64 => address)) subnets;
    /// @notice Mapping that tracks the latest nonce of the deployed
    /// subnet for each user.
    /// owner => nonce
    mapping(address => uint64) userNonces;
}
