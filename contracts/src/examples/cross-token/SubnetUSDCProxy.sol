// SPDX-License-Identifier: MIT
pragma solidity 0.8.19;

import {ERC20} from "openzeppelin-contracts/token/ERC20/ERC20.sol";
import {Ownable} from "openzeppelin-contracts/access/Ownable.sol";

contract SubnetUSDCProxy is ERC20, Ownable {
    constructor() ERC20("SubnetUSDCProxy", "sUSDC") Ownable(msg.sender) {}

    function mint(address to, uint256 amount) public onlyOwner {
        _mint(to, amount);
    }
}
