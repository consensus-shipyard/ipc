// SPDX-License-Identifier: MIT
pragma solidity 0.8.19;

import {SubnetID} from "../../structs/Subnet.sol";

import "./ERC20TokenMessenger.sol";
import "forge-std/console.sol";

import {ERC20} from "openzeppelin-contracts/token/ERC20/ERC20.sol";

contract SubnetTokenBridge is ERC20TokenMessenger, ERC20 {
    address public parentSubnetUSDC;
    SubnetID public parentSubnet;

    constructor(
        address _gateway,
        address _parentSubnetUSDC,
        SubnetID memory _parentSubnet
    ) ERC20TokenMessenger(_gateway) ERC20("USDCTestReplica", "USDCtR") {
        parentSubnetUSDC = _parentSubnetUSDC;
        parentSubnet = _parentSubnet;
    }

    function _handleIpcCall(
        IpcEnvelope memory envelope,
        CallMsg memory callMsg
    ) internal override returns (bytes memory) {
        console.log("_handleIpcCall");
        console.logBytes(envelope.message);
        console.log(envelope.value);
        console.log(envelope.nonce);
        CallMsg memory callMsg = abi.decode(envelope.message, (CallMsg));

        (address receiver, uint256 amount) = abi.decode(callMsg.params, (address, uint256));
        console.log("INFO");
        console.log(receiver);
        console.log(amount);
        _mint(receiver, amount);

        return bytes("");
    }

    function getParentSubnet() public view returns (SubnetID memory) {
        return parentSubnet;
    }

    function depositTokens(address receiver, uint256 amount) public payable {
        _sendToken(address(this), parentSubnet, parentSubnetUSDC, receiver, amount);
    }
}
