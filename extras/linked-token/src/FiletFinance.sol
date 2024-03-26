// SPDX-License-Identifier: MIT
pragma solidity 0.8.23;

import {ERC20} from "openzeppelin-contracts/token/ERC20/ERC20.sol";
import {Ownable} from "openzeppelin-contracts/access/Ownable.sol";

contract FiletFinance is ERC20, Ownable {
    constructor() ERC20("Filet Finance", "nFIL") Ownable(msg.sender) {}

    function mint(uint256 amount) public {
        _mint(msg.sender, amount);
    }

    function burn(uint256 amount) public {
        _burn(msg.sender, amount);
    }
}
