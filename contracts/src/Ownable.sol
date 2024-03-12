// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity ^0.8.23;

import {LibDiamond} from "./lib/LibDiamond.sol";

contract Ownable {
    function owner() public view returns (address) {
        return LibDiamond.contractOwner();
    }

    function transferOwnership(address newOwner) public {
        LibDiamond.transferOwnership(newOwner);
    }
}
