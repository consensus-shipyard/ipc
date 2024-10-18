// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity ^0.8.23;

contract Greeter {
    string public greeting = "Hello, World!";

    function setGreetings(string memory _greeting) public {
        greeting = _greeting;
    }

    function getGreeting() public view returns (string memory) {
        return greeting;
    }
}