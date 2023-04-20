// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.7;

import "../structs/Subnet.sol";
import "openzeppelin-contracts/utils/Strings.sol";
/// @title Helper library for manipulating SubnetID struct
/// @author LimeChain team
library SubnetIDHelper {
    using Strings for address;

    function getParentSubnet(SubnetID memory subnet) public pure returns (SubnetID memory) {
        require(subnet.route.length > 1, "error getting parent for subnet addr");

        address[] memory route = new address[](subnet.route.length - 1);
        for (uint i = 0; i < route.length; ) {
            route[i] = subnet.route[i];
            unchecked {
                ++i;
            }
        }
        
        return SubnetID({
            route: route
        });
    }

    function toString(
        SubnetID calldata subnet
    ) public pure returns (string memory) {
        string memory route = "/root";
        for (uint i = 0; i < subnet.route.length; ) {
            route = string.concat(route, "/");
            route = string.concat(route, subnet.route[i].toHexString());
            unchecked {
                ++i;
            }
        }

        return route;
    }

    function toHash(SubnetID calldata subnet) public pure returns (bytes32) {
        return keccak256(abi.encode(subnet));
    }

    function createSubnetId(SubnetID calldata subnet, address actor)
        public
        pure
        returns (SubnetID memory newSubnet)
    {
        require(subnet.route.length >= 1, "cannot set actor for empty subnet");

        newSubnet.route = new address[](subnet.route.length + 1);
        for (uint i = 0; i < subnet.route.length; ) {
            newSubnet.route[i] = subnet.route[i];
            unchecked {
                ++i;
            }
        }

        newSubnet.route[newSubnet.route.length - 1] = actor;
    }

    function getActor(SubnetID calldata subnet) public pure returns (address) {
        if (subnet.route.length <= 1) return address(0);

        return subnet.route[subnet.route.length - 1];
    }

    function isRoot(SubnetID calldata subnet) public pure returns (bool) {
        return subnet.route.length == 1;
    }

    function equals(SubnetID calldata subnet1, SubnetID calldata subnet2)
        public
        pure
        returns (bool)
    {
       if (subnet1.route.length != subnet2.route.length) return false;
       
       return toHash(subnet1) == toHash(subnet2);
    }

    /// @notice Computes the common parent of the current subnet and the one given as argument
    function commonParent(SubnetID calldata subnet1, SubnetID calldata subnet2)
        public
        pure
        returns (SubnetID memory)
    {
        uint i = 0;
        while (
            i < subnet1.route.length &&
            i < subnet2.route.length &&
            subnet1.route[i] == subnet2.route[i]
        ) {
            unchecked {
                ++i;
            }
        }
        if (i == 0) return SubnetID({route: new address[](0)});

        address[] memory route = new address[](i);
        for (uint j = 0; j < i; ) {
            route[j] = subnet1.route[j];
            unchecked {
                ++j;
            }
        }

        return SubnetID({route: route});
    }

    /// @notice In the path determined by the current subnet id, it moves
    /// down in the path from the subnet id given as argument.
    /// the subnet2 needs to be a subset of the subnet1
    function down(
        SubnetID calldata subnet1,
        SubnetID calldata subnet2
    ) public pure returns (SubnetID memory) {
        if (subnet1.route.length <= subnet2.route.length) {
            return SubnetID({route: new address[](0)});
        }

        uint i = 0;
        while (
            i < subnet2.route.length &&
            subnet1.route[i] == subnet2.route[i]
        ) {
            unchecked {
                i++;
            }
        }

        if (i == 0) {
            return SubnetID({route: new address[](0)});
        }

        address[] memory route = new address[](i + 1);

        for (uint j = 0; j <= i; ) {
            route[j] = subnet1.route[j];
            unchecked {
                j++;
            }
        }
        
        return SubnetID({route: route});
    }
}
