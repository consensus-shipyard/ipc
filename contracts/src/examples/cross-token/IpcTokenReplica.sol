// SPDX-License-Identifier: MIT
pragma solidity 0.8.23;

import {SubnetID} from "../../structs/Subnet.sol";
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
import {IpcEnvelope, ResultMsg, CallMsg, OutcomeType, IpcMsgKind} from "../../structs/CrossNet.sol";
import {IPCAddress, SubnetID} from "../../structs/Subnet.sol";
import {CrossMsgHelper} from "../../../src/lib/CrossMsgHelper.sol";
import {InvalidOriginContract, InvalidOriginSubnet} from "./IpcCrossTokenErrors.sol";
import {SubnetIDHelper} from "../../lib/SubnetIDHelper.sol";

error InvalidMessageSignature();
error InvalidMethod();
error TransferRejected(string reason);

string constant ERR_ZERO_ADDRESS = "Zero address is not allowed";
string constant ERR_VALUE_MUST_BE_ZERO = "Value must be zero";
string constant ERR_VALUE_CANNOT_BE_ZERO = "Value cannot be zero";

contract IpcTokenReplica is IpcExchange, ERC20 {
    using FvmAddressHelper for FvmAddress;
    using CrossMsgHelper for IpcEnvelope;
    using SafeERC20 for IERC20;
    using SubnetIDHelper for SubnetID;

    address public controller;
    SubnetID public controllerSubnet;

    SubnetID public networkName;

    // Define a struct to hold the sender address and the value of unconfirmed transfers
    struct TransferDetails {
        address sender;
        uint256 value;
    }

    // Create the mapping of ipc envelope hash to TransferDetails
    mapping(bytes32 => TransferDetails) private _unconfirmedTransfers;

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
        address _controller,
        SubnetID memory _controllerSubnet
    ) IpcExchange(_gateway) ERC20("USDCTestReplica", "USDCtR") {
        controller = _controller;
        controllerSubnet = _controllerSubnet;
        networkName = GatewayGetterFacet(address(_gateway)).getNetworkName();
    }

    function burnAndTransfer(address receiver, uint256 amount) external payable returns (IpcEnvelope memory committed) {
        if (msg.value != 0) {
            revert TransferRejected(ERR_VALUE_MUST_BE_ZERO);
        }
        if (controller == address(0)) {
            revert TransferRejected(ERR_ZERO_ADDRESS);
        }
        if (receiver == address(0)) {
            revert TransferRejected(ERR_ZERO_ADDRESS);
        }

        CallMsg memory message = CallMsg({
            method: abi.encodePacked(bytes4(keccak256("receiveAndUnlock(address,uint256)"))),
            params: abi.encode(receiver, amount)
        });
        IPCAddress memory destination = IPCAddress({
            subnetId: controllerSubnet,
            rawAddress: FvmAddressHelper.from(controller)
        });

        committed = performIpcCall(destination, message, 0);
        _burn(msg.sender, amount);

        addUnconfirmedTransfer(committed.toHash(), msg.sender, amount);

        emit TokenSent({
            sourceContract: address(this),
            sender: msg.sender,
            destinationSubnet: controllerSubnet,
            destinationContract: controller,
            receiver: receiver,
            nonce: committed.nonce,
            value: amount
        });
    }

    function getUnconfirmedTransfer(bytes32 hash) public view returns (address, uint256) {
        TransferDetails storage details = _unconfirmedTransfers[hash];
        return (details.sender, details.value);
    }

    // Setter function to update the address of controller
    function setController(address _newAddress) external onlyOwner {
        controller = _newAddress;
    }

    // Method for the contract owner to manually drop an entry from unconfirmedTransfers
    function manualRemoveUnconfirmedTransfer(bytes32 hash) external onlyOwner {
        removeUnconfirmedTransfer(hash);
    }

    function getControllerSubnet() external view returns (SubnetID memory) {
        return controllerSubnet;
    }

    function verifyIpcEnvelopeLogic(IpcEnvelope memory envelope) public view {
        SubnetID memory subnetId = envelope.from.subnetId;
        FvmAddress memory rawAddress = envelope.from.rawAddress;
        if (!subnetId.equals(controllerSubnet)) {
            revert InvalidOriginSubnet();
        }
        if (!rawAddress.equal(FvmAddressHelper.from(controller))) {
            revert InvalidOriginContract();
        }
    }

    function addUnconfirmedTransfer(bytes32 hash, address sender, uint256 value) internal {
        _unconfirmedTransfers[hash] = TransferDetails(sender, value);
    }

    function removeUnconfirmedTransfer(bytes32 hash) internal {
        delete _unconfirmedTransfers[hash];
    }

    modifier verifyIpcEnvelope(IpcEnvelope memory envelope) {
        verifyIpcEnvelopeLogic(envelope); // Call the function
        _; // Continue execution of the modified function
    }

    function _handleIpcCall(
        IpcEnvelope memory envelope,
        CallMsg memory callMsg
    ) internal override verifyIpcEnvelope(envelope) returns (bytes memory) {
        bytes4 methodSignature = toBytes4(callMsg.method);
        // Note: cannot use receiveAndMint.selector because the method is private.
        if (methodSignature != bytes4(keccak256("receiveAndMint(address,uint256)"))) {
            revert InvalidMethod();
        }

        (address receiver, uint256 amount) = abi.decode(callMsg.params, (address, uint256));
        receiveAndMint(receiver, amount);
        return bytes("");
    }

    // TODO: replace with abi.decode(data, (bytes4))?
    function toBytes4(bytes memory data) internal pure returns (bytes4 result) {
        if (data.length < 4) {
            revert InvalidMessageSignature();
        }

        // Assembly block to directly load the first 4 bytes
        assembly {
            result := mload(add(data, 32))
        }
    }

    function _refund(bytes32 id) private {
        (address sender, uint256 value) = getUnconfirmedTransfer(id);
        if (sender == address(0)) {
            revert TransferRejected(ERR_ZERO_ADDRESS);
        }
        if (value == 0) {
            revert TransferRejected(ERR_VALUE_CANNOT_BE_ZERO);
        }
        _mint(sender, value);
    }

    function receiveAndMint(address recipient, uint256 value) private {
        if (recipient == address(0)) {
            revert TransferRejected(ERR_ZERO_ADDRESS);
        }
        _mint(recipient, value);
    }

    function _handleIpcResult(
        IpcEnvelope storage original,
        IpcEnvelope memory result,
        ResultMsg memory resultMsg
    ) internal override verifyIpcEnvelope(original) {
        bytes32 id = resultMsg.id;
        OutcomeType outcome = resultMsg.outcome;
        if (outcome == OutcomeType.Ok) {
            removeUnconfirmedTransfer(id);
        } else {
            if (outcome == OutcomeType.SystemErr || outcome == OutcomeType.ActorErr) {
                _refund(id);
                removeUnconfirmedTransfer(id);
            }
        }
    }
}
