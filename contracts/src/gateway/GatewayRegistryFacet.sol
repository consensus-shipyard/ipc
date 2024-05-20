// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity ^0.8.23;

/// TODO: we might need this in the future, comment off first.

// import {SubnetID} from "../structs/Subnet.sol";
// import {InvalidXnetMessage} from "../errors/IPCErrors.sol";
// import {SubnetIDHelper} from "../lib/SubnetIDHelper.sol";

// /// @notice Handles the registration of subnet with the current gateway. The gateway will only route messages
// ///         for registered subnet only.
// library LibGatewayRegistry {
//     using SubnetIDHelper for SubnetID;

//     function enforceOnlyApproved(SubnetID calldata subnet) internal {
//         require(false, "todo");
//     }

//     /// @notice Application to be a subnet of the current network
//     function applyRegister(SubnetID calldata subnet) internal {
//         require(false, "todo");
//     }

//     /// @notice Revoke the registration of a subnet from the current gateway
//     function revokeRegister(SubnetID calldata subnet) internal {
//         require(false, "todo");
//     }

//     /// @notice Approve the registration of a subnet
//     function approveRegister(SubnetID calldata subnet) internal {
//         require(false, "todo");
//     }
// }

// contract GatewayRegistryFacet {
//     modifier onlyOwner {
//         require(false, "todo");

//         _;
//     }

//     /// @notice Application to be a subnet of the current network
//     function applyRegister(SubnetID calldata subnet) onlyOwner external {
//         LibGatewayRegistry.applyRegister(subnet);
//     }

//     /// @notice Revoke the registration of a subnet from the current gateway
//     function revokeRegister(SubnetID calldata subnet) onlyOwner external {
//         LibGatewayRegistry.revokeRegister(subnet);
//     }

//     /// @notice Approve the registration of a subnet
//     function approveRegister(SubnetID calldata subnet) onlyOwner external {
//         LibGatewayRegistry.approveRegister(subnet);
//     }
// }
