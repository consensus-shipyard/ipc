// SPDX-License-Identifier: MIT
pragma solidity 0.8.19;

import {SubnetID} from "../../structs/Subnet.sol";

import "./SubnetUSDCProxy.sol";
import "./ERC20TokenMessenger.sol";
import "forge-std/console.sol";

contract SubnetTokenBridge is ERC20TokenMessenger {
    SubnetUSDCProxy public proxyToken;
    address public parentSubnetUSDC;
    SubnetID public parentSubnet;

    constructor(
        address _gateway,
        address _parentSubnetUSDC,
        SubnetID memory _parentSubnet
    ) ERC20TokenMessenger(_gateway) {
        proxyToken = new SubnetUSDCProxy();
        parentSubnetUSDC = _parentSubnetUSDC;
        parentSubnet = _parentSubnet;
    }

    function getParentSubnet() public view returns (SubnetID memory) {
        return parentSubnet;
    }

    function getProxyTokenAddress() public view returns (address) {
        return address(proxyToken);
    }

    function _mint(address to, uint256 amount) internal {
        proxyToken.mint(to, amount);
    }

    /* TODO integrate with IpcReceiver */
    function onXNetMessageReceived(address _to, uint256 _amount) public /* parameters */ {
        console.log("onXNetMessageReceived");
        // Logic to handle IPC xnet message and mint tokens
        address to;
        uint256 amount;
        (to, amount) = extractParameters(/* parameters */ _to, _amount);
        _mint(to, amount);
    }

    /* TODO Change code below to parse parameters */
    function extractParameters(/* parameters */ address _to, uint256 _amount) internal view returns (address, uint256) {
        return (_to, _amount);
    }

    function depositTokens(address receiver, uint256 amount) public payable {
        _sendToken(getProxyTokenAddress(), parentSubnet, parentSubnetUSDC, receiver, amount);
    }
}
