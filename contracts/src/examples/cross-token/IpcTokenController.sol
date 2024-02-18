// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity 0.8.19;

import {SafeERC20} from "openzeppelin-contracts/token/ERC20/utils/SafeERC20.sol";
import {IERC20} from "openzeppelin-contracts/token/ERC20/IERC20.sol";
import {FvmAddressHelper} from "../../lib/FvmAddressHelper.sol";
import {FvmAddress} from "../../structs/FvmAddress.sol";
import {GatewayMessengerFacet} from "../../gateway/GatewayMessengerFacet.sol";
import {GatewayGetterFacet} from "../../gateway/GatewayGetterFacet.sol";
import {GatewayCannotBeZero, NotEnoughFunds} from "../../errors/IPCErrors.sol";
import {IpcExchange} from "../../../sdk/IpcContract.sol";
import {IpcEnvelope, ResultMsg, CallMsg, OutcomeType, IpcMsgKind} from "../../structs/CrossNet.sol";
import {IPCAddress, SubnetID} from "../../structs/Subnet.sol";
import {CrossMsgHelper} from "../../../src/lib/CrossMsgHelper.sol";
import {SubnetIDHelper} from "../../lib/SubnetIDHelper.sol";
import {InvalidOriginContract, InvalidOriginSubnet} from "./IpcCrossTokenErrors.sol";

error InvalidMessageSignature();
error InvalidMethod();
error TransferRejected(string reason);

string constant ERR_ZERO_ADDRESS = "Zero address is not allowed";
string constant ERR_VALUE_MUST_BE_ZERO = "Value must be zero";
string constant ERR_VALUE_CANNOT_BE_ZERO = "Value cannot be zero";


/**
 * @title IpcTokenController
 * @notice Contract to handle token transfer from L1, lock them and mint on L2.
 */
contract IpcTokenController is IpcExchange {
    using SafeERC20 for IERC20;
    using CrossMsgHelper for IpcEnvelope;
    using SubnetIDHelper for SubnetID;
    using FvmAddressHelper for FvmAddress;

    address private tokenContractAddress;
    SubnetID private destinationSubnet;
    address private destinationContract;
    SubnetID public networkName;

    // Define a struct to hold the sender address and the value of unconfirmed transfers
    struct TransferDetails {
        address sender;
        uint256 value;
    }

    // Create the mapping of ipc envelope hash to TransferDetails
    mapping(bytes32 => TransferDetails) public _unconfirmedTransfers;

    uint64 public nonce = 0;

    event TokenSent(
        address tokenContractAddress,
        address sender,
        SubnetID destinationSubnet,
        address destinationContract,
        address receiver,
        uint64 nonce,
        uint256 value
    );

    event TokensUnlocked(address indexed receiver, uint256 amount);

    /**
     * @dev Constructor for IpcTokenController
     * @param _gateway Address of the gateway for cross-network communication
     * @param _tokenContractAddress Address of the source ERC20 token contract
     * @param _destinationSubnet SubnetID of the destination network
     * @param _destinationContract Address of the destination contract for minting
     */
    constructor(
        address _gateway,
        address _tokenContractAddress,
        SubnetID memory _destinationSubnet,
        address _destinationContract
    ) IpcExchange(_gateway) {
        tokenContractAddress = _tokenContractAddress;
        destinationSubnet = _destinationSubnet;
        destinationContract = _destinationContract;

        networkName = GatewayGetterFacet(address(_gateway)).getNetworkName();
    }

    /**
     * @notice Transfers tokens from L1, locks them, and requests minting on L2.
     * @param receiver Address to receive the minted tokens on L2
     * @param amount Amount of tokens to be transferred and minted
     */
    function lockAndTransfer(address receiver, uint256 amount) external payable {
        // Transfer and lock tokens on L1 using the inherited sendToken function
        _sendToken(tokenContractAddress, destinationSubnet, destinationContract, receiver, amount);
    }

    function lockAndTransferWithReturn(
        address receiver,
        uint256 amount
    ) external payable returns (IpcEnvelope memory envelope) {
        // Transfer and lock tokens on L1 using the inherited sendToken function
        return _sendToken(tokenContractAddress, destinationSubnet, destinationContract, receiver, amount);
    }

    function getUnconfirmedTransfer(bytes32 hash) public view returns (address, uint256) {
        TransferDetails storage details = _unconfirmedTransfers[hash];
        return (details.sender, details.value);
    }

    // Method for the contract owner to manually drop an entry from unconfirmedTransfers
    function manualRemoveUnconfirmedTransfer(bytes32 hash) external onlyOwner {
        removeUnconfirmedTransfer(hash);
    }

    function addUnconfirmedTransfer(bytes32 hash, address sender, uint256 value) internal {
        _unconfirmedTransfers[hash] = TransferDetails(sender, value);
    }

    function removeUnconfirmedTransfer(bytes32 hash) internal {
        delete _unconfirmedTransfers[hash];
    }

    function _handleIpcCall(
        IpcEnvelope memory envelope,
        CallMsg memory callMsg
    ) internal override verifyIpcEnvelope(envelope) returns (bytes memory) {

        bytes4 methodSignature = toBytes4(callMsg.method);
        if (methodSignature != bytes4(keccak256("receiveAndUnlock(address,uint256)"))) {
            revert InvalidMethod();
        }

        (address receiver, uint256 amount) = abi.decode(callMsg.params, (address, uint256));
        // Call receiveAndUnlock to process the unlocking and transfer of tokens
        receiveAndUnlock(receiver, amount);
        return bytes("");
    }

    modifier verifyIpcEnvelope(IpcEnvelope memory envelope) {
        verifyIpcEnvelopeLogic(envelope); // Call the function
        _; // Continue execution of the modified function
    }

    //only accept messages from replica contract
    function verifyIpcEnvelopeLogic(IpcEnvelope memory envelope) public {
        SubnetID memory subnetId = envelope.from.subnetId;
        FvmAddress memory rawAddress = envelope.from.rawAddress;
        if (!subnetId.equals(destinationSubnet)) {
            revert InvalidOriginSubnet();
        }
        if (!rawAddress.equal(FvmAddressHelper.from(destinationContract))) {
            revert InvalidOriginContract();
        }
    }

    function receiveAndUnlock(address receiver, uint256 amount) private {
        if (receiver == address(0)) {
            revert TransferRejected(ERR_ZERO_ADDRESS);
        }

        // Transfer the specified amount of tokens to the receiver
        IERC20(tokenContractAddress).safeTransfer(receiver, amount);

        // Emit an event for the token unlock and transfer
        emit TokensUnlocked(receiver, amount);
    }

    function _sendToken(
        address tokenContractAddress,
        SubnetID memory destinationSubnet,
        address destinationContract,
        address receiver,
        uint256 amount
    ) internal returns (IpcEnvelope memory committed) {
        if (msg.value != 0) {
            revert TransferRejected(ERR_VALUE_MUST_BE_ZERO);
        }
        if (destinationContract == address(0)) {
            revert TransferRejected(ERR_ZERO_ADDRESS);
        }
        if (receiver == address(0)) {
            revert TransferRejected(ERR_ZERO_ADDRESS);
        }

        IERC20(tokenContractAddress).safeTransferFrom({from: msg.sender, to: address(this), value: amount});

        CallMsg memory message = CallMsg({
            method: abi.encodePacked(bytes4(keccak256("receiveAndMint(address,uint256)"))),
            params: abi.encode(receiver, amount)
        });
        IPCAddress memory destination = IPCAddress({
            subnetId: destinationSubnet,
            rawAddress: FvmAddressHelper.from(destinationContract)
        });

        committed = performIpcCall(destination, message, 0);

        addUnconfirmedTransfer(committed.toHash(), msg.sender, amount);

        emit TokenSent({
            tokenContractAddress: tokenContractAddress,
            sender: msg.sender,
            destinationSubnet: destinationSubnet,
            destinationContract: destinationContract,
            receiver: receiver,
            nonce: committed.nonce,
            value: amount
        });
    }

    function _refund(bytes32 id) private {
        (address sender, uint256 value) = getUnconfirmedTransfer(id);
        if (sender == address(0)) {
            revert TransferRejected(ERR_ZERO_ADDRESS);
        }
        if( value == 0){
            revert TransferRejected(ERR_VALUE_CANNOT_BE_ZERO);
        }

        IERC20(tokenContractAddress).safeTransfer(sender, value);
    }


    function _handleIpcResult(
        IpcEnvelope storage original,
        IpcEnvelope memory result,
        ResultMsg memory resultMsg
    ) internal override verifyIpcEnvelope(original) {

        bytes32 id = resultMsg.id;
        OutcomeType outcome = resultMsg.outcome;
        if(outcome == OutcomeType.Ok){
            removeUnconfirmedTransfer(id);
        }else{
            if( outcome == OutcomeType.SystemErr || outcome == OutcomeType.ActorErr ){
                _refund(id);
                removeUnconfirmedTransfer(id);
            }
        }
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
}