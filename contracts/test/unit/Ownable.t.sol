// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity ^0.8.23;

import "forge-std/Test.sol";

import "fevmate/utils/FilAddress.sol";
import {Ownable} from "../../src/Ownable.sol";
import {LibDiamond} from "../../src/lib/LibDiamond.sol";

contract OwnableInstance is Ownable {
    constructor(address owner) {
        LibDiamond.setContractOwner(owner);
    }
}

contract OwnableTest is Test {
    function testOwnableOk() public {
        address firstOwner = address(1);
        address secondOwner = address(2);
        vm.deal(firstOwner, 1 ether);
        vm.deal(secondOwner, 1 ether);

        OwnableInstance o = new OwnableInstance(firstOwner);
        require(o.owner() == firstOwner, "owner not correct");

        vm.prank(secondOwner);
        vm.expectRevert(LibDiamond.NotOwner.selector);
        o.transferOwnership(secondOwner);

        // owner can perform transfer
        vm.prank(firstOwner);
        o.transferOwnership(secondOwner);
        require(o.owner() == secondOwner, "second owner not correct");
    }
}
