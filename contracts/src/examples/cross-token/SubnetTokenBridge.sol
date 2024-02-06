// SPDX-License-Identifier: MIT
pragma solidity 0.8.19;

import {SubnetID} from "../../structs/Subnet.sol";

import "./ERC20TokenMessenger.sol";
import "forge-std/console.sol";

import {ERC20} from "openzeppelin-contracts/token/ERC20/ERC20.sol";

contract SubnetTokenBridge is ERC20TokenMessenger, ERC20 {
    address public parentSubnetUSDC;
    SubnetID public parentSubnet;

    SubnetID public networkName;
    GatewayMessengerFacet private immutable messenger;

    constructor(
        address _gateway,
        address _parentSubnetUSDC,
        SubnetID memory _parentSubnet
    ) ERC20TokenMessenger(_gateway) ERC20("USDCTestReplica", "USDCtR") {
        parentSubnetUSDC = _parentSubnetUSDC;
        parentSubnet = _parentSubnet;

        networkName = GatewayGetterFacet(address(_gateway)).getNetworkName();
        messenger = GatewayMessengerFacet(address(_gateway));
    }

    function _handleIpcCall(
        IpcEnvelope memory envelope,
        CallMsg memory callMsg
    ) internal override returns (bytes memory) {
        console.log("_handleIpcCall");
        console.logBytes(envelope.message);
        console.log(envelope.value);
        console.log(envelope.nonce);
        //CallMsg memory callMsg = abi.decode(envelope.message, (CallMsg));

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
    
    function x() public payable {
        console.log("HI");
        //_sendToken(address(this), parentSubnet, parentSubnetUSDC, receiver, amount);
    }

    function depositTokens(address receiver, uint256 amount) public payable returns (IpcEnvelope memory committed) {
        if (receiver == address(0)) {
            revert ZeroAddress();
        }
        if (msg.value != DEFAULT_CROSS_MSG_FEE) {
            revert NotEnoughFunds();
        }

        uint64 lastNonce = nonce;

        emit TokenSent({
            sourceContract: address(this),
            sender: msg.sender,
            destinationSubnet: parentSubnet,
            destinationContract: parentSubnetUSDC,
            receiver: receiver,
            nonce: lastNonce,
            value: amount
        });
        nonce++;

        CallMsg memory message = CallMsg({
            method: abi.encodePacked(bytes4(keccak256("transfer(address,uint256)"))),
            params: abi.encode(receiver, amount)
        });
        IpcEnvelope memory crossMsg = IpcEnvelope({
            kind: IpcMsgKind.Call,
            from: IPCAddress({subnetId: networkName, rawAddress: FvmAddressHelper.from(address(this))}),
            to: IPCAddress({subnetId: parentSubnet, rawAddress: FvmAddressHelper.from(parentSubnetUSDC)}),
            value: DEFAULT_CROSS_MSG_FEE,
            nonce: lastNonce,
            message: abi.encode(message)
        });

        return messenger.sendContractXnetMessage{value: DEFAULT_CROSS_MSG_FEE}(crossMsg);
    }


}
