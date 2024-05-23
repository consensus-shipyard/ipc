// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity ^0.8.23;

import {SubnetID} from "../structs/Subnet.sol";

contract GatewayChildFacet {
    function id() internal returns(SubnetID memory) {
        return LibGatewayChildQuery.id();
    }
}

library LibGatewayChildQuery {
    function diamondStorage() internal pure returns (SubnetInfo storage ds) {
        bytes32 position = keccak256("ipc.gateway.child.storage");
        assembly {
            ds.slot := position
        }
    }

    function id() internal returns(SubnetID memory) {
        return diamondStorage().id;
    }
}

// ============ Internal Usage Only ============

/// @notice Stores the child subnet information
struct SubnetInfo {
    /// @notice The id of the subnet
    SubnetID id;
}