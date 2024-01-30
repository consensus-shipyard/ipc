// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity 0.8.19;

import {IpcExchange} from "../../../sdk/IpcContract.sol";
import {IpcEnvelope, ResultMsg, CallMsg, IpcMsgKind} from "../../structs/CrossNet.sol";
import {IPCAddress, SubnetID} from "../../structs/Subnet.sol";
import {FvmAddress} from "../../structs/FvmAddress.sol";
import {GatewayMessengerFacet} from "../../gateway/GatewayMessengerFacet.sol";
import {GatewayGetterFacet} from "../../gateway/GatewayGetterFacet.sol";
import {GatewayCannotBeZero, NotEnoughFunds} from "../../errors/IPCErrors.sol";
import {FvmAddressHelper} from "../../lib/FvmAddressHelper.sol";

import {IERC20} from "openzeppelin-contracts/token/ERC20/IERC20.sol";
import {SafeERC20} from "openzeppelin-contracts/token/ERC20/utils/SafeERC20.sol";
import {ReentrancyGuard} from "openzeppelin-contracts/security/ReentrancyGuard.sol";

error NoTransfer();
error ZeroAddress();

/**
 * @title TokenMessenger
 * @notice An example of a contract that allows users to send a token across subnets.
 */
abstract contract ERC20TokenMessenger is IpcExchange, ReentrancyGuard {
    using FvmAddressHelper for FvmAddress;
    using SafeERC20 for IERC20;

    uint64 public nonce = 0;
    // Gateway facet used to send messages
    GatewayMessengerFacet private immutable messenger;
    // Gateway facet used to get information about the subnet
    GatewayGetterFacet private immutable info;
    // Cross-net fee
    uint256 public constant DEFAULT_CROSS_MSG_FEE = 10 gwei;

    event TokenSent(
        address sourceContract,
        address sender,
        SubnetID destinationSubnet,
        address destinationContract,
        address receiver,
        uint64 nonce,
        uint256 value
    );

    function _handleIpcResult(IpcEnvelope storage original, IpcEnvelope memory result, ResultMsg memory resultMsg) internal override {
        console.log("_handleIpcResult");
    }

    function _handleIpcCall(
        IpcEnvelope memory envelope,
        CallMsg memory callMsg
    ) internal override returns (bytes memory) {
        console.log("handling ipc call");
        //console.log("envelope - ", envelope);
        //console.log("callMsg - ", callMsg);
        return bytes("");
    }


    constructor(address _gateway) IpcExchange(_gateway){
        if (_gateway == address(0)) {
            revert GatewayCannotBeZero();
        }
        messenger = GatewayMessengerFacet(address(_gateway));
        info = GatewayGetterFacet(address(_gateway));
    }

    function sendToken(
        address sourceContract,
        SubnetID memory destinationSubnet,
        address destinationContract,
        address receiver,
        uint256 amount
    ) external payable nonReentrant {
        _sendToken(sourceContract, destinationSubnet, destinationContract, receiver, amount);
    }

    function _sendToken(
        address sourceContract,
        SubnetID memory destinationSubnet,
        address destinationContract,
        address receiver,
        uint256 amount
    ) internal returns (IpcEnvelope memory committed) {
        if (destinationContract == address(0)) {
            revert ZeroAddress();
        }
        if (receiver == address(0)) {
            revert ZeroAddress();
        }
        if (msg.value != DEFAULT_CROSS_MSG_FEE) {
            revert NotEnoughFunds();
        }

        uint64 lastNonce = nonce;

        emit TokenSent({
            sourceContract: sourceContract,
            sender: msg.sender,
            destinationSubnet: destinationSubnet,
            destinationContract: destinationContract,
            receiver: receiver,
            nonce: lastNonce,
            value: amount
        });
        nonce++;

        uint256 startingBalance = IERC20(sourceContract).balanceOf(address(this));
        IERC20(sourceContract).safeTransferFrom({from: msg.sender, to: address(this), value: amount});
        uint256 endingBalance = IERC20(sourceContract).balanceOf(address(this));

        if (endingBalance <= startingBalance) {
            revert NoTransfer();
        }
        CallMsg memory message = CallMsg({
            method: abi.encodePacked(bytes4(keccak256("transfer(address,uint256)"))),
            params: abi.encode(receiver, amount)
        });
        IpcEnvelope memory crossMsg = IpcEnvelope({
            kind: IpcMsgKind.Call,
            from: IPCAddress({subnetId: info.getNetworkName(), rawAddress: FvmAddressHelper.from(sourceContract)}),
            to: IPCAddress({subnetId: destinationSubnet, rawAddress: FvmAddressHelper.from(destinationContract)}),
            value: DEFAULT_CROSS_MSG_FEE,
            nonce: lastNonce,
            message: abi.encode(message)
        });

        return messenger.sendContractXnetMessage{value: DEFAULT_CROSS_MSG_FEE}(crossMsg);
    }

    receive() external payable {}
}
