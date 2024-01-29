// SPDX-License-Identifier: MIT
pragma solidity 0.8.19;

import {ERC20} from "openzeppelin-contracts/token/ERC20/ERC20.sol";
import {Ownable} from "openzeppelin-contracts/access/Ownable.sol";

contract USDCMock is ERC20, Ownable {
    constructor(string memory name, string memory symbol) ERC20(name, symbol) Ownable(msg.sender) {}

    function mint(uint256 amount) public onlyOwner {
        _mint(msg.sender, amount);
    }
}
