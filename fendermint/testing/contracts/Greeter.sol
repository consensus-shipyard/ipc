//SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

contract Greeter {
    string private greeting;

    event GreetingSet(string greeting);

    //This constructor assigns the initial greeting and emit GreetingSet event
    constructor(string memory _greeting) {
        greeting = _greeting;

        emit GreetingSet(_greeting);
    }

    //This function returns the current value stored in greeting variable
    function greet() public view returns (string memory) {
        //uint256 number = uint256(blockhash(block.number - 1));
        bytes32 myHash = blockhash(block.number - 1);

        return string(abi.encodePacked(myHash));
    }

    //This function sets the new greeting msg from the one passed down as parameter and emit event
    function setGreeting(string memory _greeting) public {
        greeting = _greeting;

        emit GreetingSet(_greeting);
    }
}
