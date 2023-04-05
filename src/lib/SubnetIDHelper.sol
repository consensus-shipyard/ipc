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
        for(uint i = 0; i < route.length; i++) {
            route[i] = subnet.route[i];
        }
        
        return SubnetID({
            route: route
        });
    }

    function toString(SubnetID memory subnet) public pure returns (string memory) {
        string memory route = "/root";
        for(uint i = 0; i < subnet.route.length; i++) {
            route = string.concat(route, "/");
            route = string.concat(route, subnet.route[i].toHexString());

        }

        return route;
    }

    function toHash(SubnetID memory subnet) public pure returns(bytes32) {
        return keccak256(abi.encode(subnet));
    }

    function createSubnetId(SubnetID memory subnet, address actor) public pure returns (SubnetID memory newSubnet) {
        require(subnet.route.length != 0, "cannot set actor for empty subnet");

        newSubnet.route = new address[](subnet.route.length + 1);
        for(uint i = 0; i < subnet.route.length; i++) {
            newSubnet.route[i] = subnet.route[i];
        }

        newSubnet.route[newSubnet.route.length - 1] = actor;
    }

    function getActor(SubnetID memory subnet) public pure returns (address) {
        if(subnet.route.length <= 1)
            return address(0);

        return subnet.route[subnet.route.length - 1];
    }

    function isRoot(SubnetID memory subnet) public pure returns (bool) {
        return subnet.route.length == 1;
    }

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
