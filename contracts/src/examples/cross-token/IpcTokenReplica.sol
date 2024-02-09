// SPDX-License-Identifier: MIT
pragma solidity 0.8.19;

import {SubnetID} from "../../structs/Subnet.sol";
import {ReentrancyGuard} from "../../lib/LibReentrancyGuard.sol";
import {IERC20} from "openzeppelin-contracts/token/ERC20/IERC20.sol";
import {ERC20} from "openzeppelin-contracts/token/ERC20/ERC20.sol";
import {IpcExchange} from "../../../sdk/IpcContract.sol";
import {FvmAddressHelper} from "../../lib/FvmAddressHelper.sol";
import {FvmAddress} from "../../structs/FvmAddress.sol";
import {SafeERC20} from "openzeppelin-contracts/token/ERC20/utils/SafeERC20.sol";
import {GatewayMessengerFacet} from "../../gateway/GatewayMessengerFacet.sol";
import {GatewayGetterFacet} from "../../gateway/GatewayGetterFacet.sol";
import {GatewayCannotBeZero, NotEnoughFunds} from "../../errors/IPCErrors.sol";
import {IpcExchange} from "../../../sdk/IpcContract.sol";
import {IpcEnvelope, ResultMsg, CallMsg, IpcMsgKind} from "../../structs/CrossNet.sol";
import {IPCAddress, SubnetID} from "../../structs/Subnet.sol";
import {CrossMsgHelper} from "../../../src/lib/CrossMsgHelper.sol";

error NoTransfer();
error ZeroAddress();

contract IpcTokenReplica is IpcExchange, ERC20, ReentrancyGuard {
    using FvmAddressHelper for FvmAddress;
    using CrossMsgHelper for IpcEnvelope;
    using SafeERC20 for IERC20;

    address public parentSubnetUSDC;
    SubnetID public parentSubnet;

    SubnetID public networkName;

    // Define a struct to hold the sender address and the value of unconfirmed transfers
    struct TransferDetails {
        address sender;
        uint256 value;
    }

    // Create the mapping of ipc envelope hash to TransferDetails
    mapping(bytes32 => TransferDetails) public unconfirmedTransfers;

    uint256 public constant DEFAULT_CROSS_MSG_FEE = 10 gwei;
    uint64 public nonce = 0;

    event TokenSent(
        address sourceContract,
        address sender,
        SubnetID destinationSubnet,
        address destinationContract,
        address receiver,
        uint64 nonce,
        uint256 value
    );

    constructor(
        address _gateway,
        address _parentSubnetUSDC,
        SubnetID memory _parentSubnet
    ) IpcExchange(_gateway) ERC20("USDCTestReplica", "USDCtR") {
        parentSubnetUSDC = _parentSubnetUSDC;
        parentSubnet = _parentSubnet;
        networkName = GatewayGetterFacet(address(_gateway)).getNetworkName();
    }

    function burnAndTransfer(address receiver, uint256 amount) external payable returns (IpcEnvelope memory committed) {
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
            method: abi.encodePacked(bytes4(keccak256("receiveAndUnlock(address,uint256)"))),
            params: abi.encode(receiver, amount)
        });
        IPCAddress memory destination = IPCAddress({subnetId: parentSubnet, rawAddress: FvmAddressHelper.from(parentSubnetUSDC)});
        committed= performIpcCall(destination, message, DEFAULT_CROSS_MSG_FEE);
        _burn(receiver, amount);

        //add receipt to unconfirmedTransfers
        unconfirmedTransfers[committed.toHash()] = TransferDetails(msg.sender, amount);
    }

    function getUnconfirmedTransfer(bytes32 hash) public view returns (address, uint256) {
        TransferDetails storage details = unconfirmedTransfers[hash];
        return (details.sender, details.value);
    }

    // Setter function to update the address of parentSubnetUSDC
    function setParentSubnetUSDC(address _newAddress) external onlyOwner {
        parentSubnetUSDC = _newAddress;
    }

    function getParentSubnet() external view returns (SubnetID memory) {
        return parentSubnet;
    }

    function _handleIpcCall(
        IpcEnvelope memory envelope,
        CallMsg memory callMsg
    ) internal override returns (bytes memory) {
        (address recipient, uint256 amount) = abi.decode(callMsg.params, (address, uint256));
        receiveAndMint(receiver, amount);
        return bytes("");
    }

    function receiveAndMint(address recipient, uint256 value) private {
        if (recipient == address(0)) {
            revert ZeroAddress();
        }
        _mint(recipient, value);
    }

    function _handleIpcResult(
        IpcEnvelope storage original,
        IpcEnvelope memory result,
        ResultMsg memory resultMsg
    ) internal override {}
}
