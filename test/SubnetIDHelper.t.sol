// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.7;

import "forge-std/Test.sol";
import "forge-std/console.sol";
import "openzeppelin-contracts/utils/Strings.sol";

import "../src/lib/SubnetIDHelper.sol";

contract SubnetIDHelperTest is Test {
    using Strings for *;
    using SubnetIDHelper for SubnetID;

    address ROOT_ADDRESS;
    address SUBNET_ONE_ADDRESS;
    address SUBNET_TWO_ADDRESS;

    bytes32 constant EMPTY_SUBNET_ID_HASH =
        0x2b88776ddf4d5290d360b934e1785b2f98fc538a5a4d0dc8dab162167e24841c;

    SubnetID EMPTY_SUBNET_ID = SubnetID(new address[](0));

    function setUp() public {
        ROOT_ADDRESS = makeAddr("root"); // 0x9f86b1918e5cf3a2150388024ff87df8c90d1d82
        SUBNET_ONE_ADDRESS = makeAddr("subnet_one"); // 0xb0c7ebf9ce6bfce01fba323a8b98054326032522
        SUBNET_TWO_ADDRESS = makeAddr("subnet_two"); // 0x374b3bb66c3a33e054e804d5ea825a8c2514816a
    }

    function test_GetParentSubnet_Fails_EmptySubnet() public {
        vm.expectRevert("error getting parent for subnet addr");

        EMPTY_SUBNET_ID.getParentSubnet();
    }

    function test_GetParentSubnet_Fails_NoParent() public {
        address[] memory route = new address[](1);
        route[0] = ROOT_ADDRESS;

        SubnetID memory emptySubnet = SubnetID(route);

        vm.expectRevert("error getting parent for subnet addr");

        emptySubnet.getParentSubnet();
    }

    function test_GetParentSubnet_Works_ParentRoot() public view {
        address[] memory route = new address[](2);
        route[0] = ROOT_ADDRESS;
        route[1] = SUBNET_ONE_ADDRESS;

        SubnetID memory subnetId = SubnetID(route);

        address[] memory expectedRoute = new address[](1);
        expectedRoute[0] = ROOT_ADDRESS;

        require(
            subnetId.getParentSubnet().toHash() ==
                SubnetID(expectedRoute).toHash()
        );
    }

    function test_GetParentSubnet_Works_ParentSubnetOne() public view {
        address[] memory route = new address[](3);
        route[0] = ROOT_ADDRESS;
        route[1] = SUBNET_ONE_ADDRESS;
        route[2] = SUBNET_TWO_ADDRESS;

        SubnetID memory subnetId = SubnetID(route);

        address[] memory expectedRoute = new address[](2);
        expectedRoute[0] = ROOT_ADDRESS;
        expectedRoute[1] = SUBNET_ONE_ADDRESS;

        require(
            subnetId.getParentSubnet().toHash() ==
                SubnetID(expectedRoute).toHash()
        );
    }

    function test_ToString_Works_NoRoutes() public view {
        require(EMPTY_SUBNET_ID.toString().equal("/root"));
    }

    function test_ToString_Works_SingleRoute() public view {
        address[] memory route = new address[](1);
        route[0] = ROOT_ADDRESS;

        require(
            SubnetID(route).toString().equal(
                "/root/0x9f86b1918e5cf3a2150388024ff87df8c90d1d82"
            )
        );
    }

    function test_ToString_Works_MultiRoute() public view {
        address[] memory route = new address[](3);
        route[0] = ROOT_ADDRESS;
        route[1] = SUBNET_ONE_ADDRESS;
        route[2] = SUBNET_TWO_ADDRESS;

        require(
            SubnetID(route).toString().equal(
                "/root/0x9f86b1918e5cf3a2150388024ff87df8c90d1d82/0xb0c7ebf9ce6bfce01fba323a8b98054326032522/0x374b3bb66c3a33e054e804d5ea825a8c2514816a"
            )
        );
    }

    function test_ToHash_Works_EmptySubnet() public view {
        require(EMPTY_SUBNET_ID.toHash() == EMPTY_SUBNET_ID_HASH);
    }

    function test_ToHash_Works_NonEmptySubnet() public view {
        address[] memory route = new address[](2);
        route[0] = ROOT_ADDRESS;
        route[1] = SUBNET_ONE_ADDRESS;

        SubnetID memory subnetId = SubnetID(route);

        bytes32 expectedSubnetIdHash = keccak256(abi.encode(subnetId));

        require(subnetId.toHash() == expectedSubnetIdHash);
    }

    function test_CreateSubnetId_Fails_EmptySubnet() public {
        vm.expectRevert("cannot set actor for empty subnet");

        EMPTY_SUBNET_ID.createSubnetId(SUBNET_ONE_ADDRESS);
    }

    function test_CreateSubnetId_Works() public view {
        address[] memory route = new address[](1);
        route[0] = ROOT_ADDRESS;

        SubnetID memory subnetId = SubnetID(route).createSubnetId(
            SUBNET_ONE_ADDRESS
        );

        address[] memory expectedRoute = new address[](2);
        expectedRoute[0] = ROOT_ADDRESS;
        expectedRoute[1] = SUBNET_ONE_ADDRESS;

        require(subnetId.toHash() == SubnetID(expectedRoute).toHash());
    }

    function test_GetActor_Works_EmptySubnet() public view {
        address emptyActor = EMPTY_SUBNET_ID.getActor();
        require(emptyActor == address(0));
    }

    function test_GetActor_Works_RootSubnet() public view {
        address[] memory route = new address[](1);
        route[0] = ROOT_ADDRESS;

        address emptyActor = SubnetID(route).getActor();
        require(emptyActor == address(0));
    }

    function test_GetActor_Works_EmptyActor() public view {
        address[] memory route = new address[](2);
        route[0] = ROOT_ADDRESS;
        route[1] = SUBNET_ONE_ADDRESS;

        address actor = SubnetID(route).getActor();
        require(actor == SUBNET_ONE_ADDRESS);
    }

    function test_IsRoot_Works_EmptySubnet() public view {
        require(EMPTY_SUBNET_ID.isRoot() == false);
    }

    function test_IsRoot_Works_ChildSubnet() public view {
        address[] memory route = new address[](2);
        route[0] = ROOT_ADDRESS;
        route[1] = SUBNET_ONE_ADDRESS;

        require(SubnetID(route).isRoot() == false);
    }   

    function test_IsRoot_Works_RootSubnet() public view {
        address[] memory route = new address[](1);
        route[0] = ROOT_ADDRESS;

        require(SubnetID(route).isRoot() == true);
    }

    function test_Down_Some_1() public pure {
        address[] memory subnetRoute1 = new address[](4);
        subnetRoute1[0] = address(100);
        subnetRoute1[1] = address(101);
        subnetRoute1[2] = address(102);
        subnetRoute1[3] = address(103);

        address[] memory subnetRoute2 = new address[](2);
        subnetRoute2[0] = address(100);
        subnetRoute2[1] = address(101);

        SubnetID memory subnetId1 = SubnetID(subnetRoute1);
        SubnetID memory subnetId2 = SubnetID(subnetRoute2);

        SubnetID memory subnetId = subnetId1.down(subnetId2);

        address[] memory expectedRoute = new address[](3);
        expectedRoute[0] = address(100);
        expectedRoute[1] = address(101);
        expectedRoute[2] = address(102);

        require(subnetId.toHash() == SubnetID(expectedRoute).toHash());
    }

    function test_Down_Some_2() public pure {
        address[] memory subnetRoute1 = new address[](4);
        subnetRoute1[0] = address(100);
        subnetRoute1[1] = address(101);
        subnetRoute1[2] = address(102);
        subnetRoute1[3] = address(103);

        address[] memory subnetRoute2 = new address[](3);
        subnetRoute2[0] = address(100);
        subnetRoute2[1] = address(101);
        subnetRoute2[2] = address(102);

        SubnetID memory subnetId1 = SubnetID(subnetRoute1);
        SubnetID memory subnetId2 = SubnetID(subnetRoute2);

        SubnetID memory subnetId = subnetId1.down(subnetId2);

        address[] memory expectedRoute = new address[](4);
        expectedRoute[0] = address(100);
        expectedRoute[1] = address(101);
        expectedRoute[2] = address(102);
        expectedRoute[3] = address(103);

        require(subnetId.toHash() == SubnetID(expectedRoute).toHash());
    }

    function test_Down_None_1() public pure {
        address[] memory subnetRoute1 = new address[](1);
        subnetRoute1[0] = address(100);

        address[] memory subnetRoute2 = new address[](2);
        subnetRoute2[0] = address(100);
        subnetRoute2[1] = address(101);

        SubnetID memory subnetId1 = SubnetID(subnetRoute1);
        SubnetID memory subnetId2 = SubnetID(subnetRoute2);

        SubnetID memory subnetId = subnetId1.down(subnetId2);

        require(subnetId.toHash() == EMPTY_SUBNET_ID_HASH);
    }

    function test_Down_None_2() public pure {
        address[] memory subnetRoute1 = new address[](2);
        subnetRoute1[0] = address(100);
        subnetRoute1[1] = address(101);

        address[] memory subnetRoute2 = new address[](2);
        subnetRoute2[0] = address(100);
        subnetRoute2[1] = address(101);

        SubnetID memory subnetId1 = SubnetID(subnetRoute1);
        SubnetID memory subnetId2 = SubnetID(subnetRoute2);

        SubnetID memory subnetId = subnetId1.down(subnetId2);

        require(subnetId.toHash() == EMPTY_SUBNET_ID_HASH);
    }

    function test_Down_None_3() public pure {
        address[] memory subnetRoute1 = new address[](3);
        subnetRoute1[0] = address(100);
        subnetRoute1[1] = address(101);
        subnetRoute1[2] = address(102);

        address[] memory subnetRoute2 = new address[](4);
        subnetRoute2[0] = address(100);
        subnetRoute2[1] = address(101);
        subnetRoute2[2] = address(102);
        subnetRoute2[3] = address(103);

        SubnetID memory subnetId1 = SubnetID(subnetRoute1);
        SubnetID memory subnetId2 = SubnetID(subnetRoute2);

        SubnetID memory subnetId = subnetId1.down(subnetId2);

        require(subnetId.toHash() == EMPTY_SUBNET_ID_HASH);
    }

    function test_Down_None_4() public pure {
        address[] memory subnetRoute1 = new address[](2);
        subnetRoute1[0] = address(101);
        subnetRoute1[1] = address(100);

        address[] memory subnetRoute2 = new address[](1);
        subnetRoute2[0] = address(100);

        SubnetID memory subnetId1 = SubnetID(subnetRoute1);
        SubnetID memory subnetId2 = SubnetID(subnetRoute2);

        SubnetID memory subnetId = subnetId1.down(subnetId2);

        require(subnetId.toHash() == EMPTY_SUBNET_ID_HASH);
    }
}
