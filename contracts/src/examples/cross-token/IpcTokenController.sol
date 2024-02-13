// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity 0.8.19;

import {SafeERC20} from "openzeppelin-contracts/token/ERC20/utils/SafeERC20.sol";
import {IERC20} from "openzeppelin-contracts/token/ERC20/IERC20.sol";
import {ReentrancyGuard} from "../../lib/LibReentrancyGuard.sol";
import {FvmAddressHelper} from "../../lib/FvmAddressHelper.sol";
import {FvmAddress} from "../../structs/FvmAddress.sol";
import {GatewayMessengerFacet} from "../../gateway/GatewayMessengerFacet.sol";
import {GatewayGetterFacet} from "../../gateway/GatewayGetterFacet.sol";
import {GatewayCannotBeZero, NotEnoughFunds} from "../../errors/IPCErrors.sol";
import {IpcExchange} from "../../../sdk/IpcContract.sol";
import {IpcEnvelope, ResultMsg, CallMsg, IpcMsgKind} from "../../structs/CrossNet.sol";
import {IPCAddress, SubnetID} from "../../structs/Subnet.sol";
import {CrossMsgHelper} from "../../../src/lib/CrossMsgHelper.sol";
import {SubnetIDHelper} from "../../lib/SubnetIDHelper.sol";
import {InvalidOriginContract, InvalidOriginSubnet} from "./IpcCrossTokenErrors.sol";

error NoTransfer();
error ZeroAddress();
error InvalidMessageSignature();
error InvalidMethod();

/**
 * @title IpcTokenController
 * @notice Contract to handle token transfer from L1, lock them and mint on L2.
 */
contract IpcTokenController is IpcExchange, ReentrancyGuard {
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
    mapping(bytes32 => TransferDetails) public unconfirmedTransfers;

    uint256 public constant DEFAULT_CROSS_MSG_FEE = 10 gwei;
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
        TransferDetails storage details = unconfirmedTransfers[hash];
        return (details.sender, details.value);
    }

    function _handleIpcCall(
        IpcEnvelope memory envelope,
        CallMsg memory callMsg
    ) internal override returns (bytes memory) {
        //only accept messages from replica contract
        verifyIpcEnvelope(envelope);

        bytes4 methodSignature = toBytes4(callMsg.method);
        if (methodSignature != bytes4(keccak256("receiveAndUnlock(address,uint256)"))) {
            revert InvalidMethod();
        }

        (address receiver, uint256 amount) = abi.decode(callMsg.params, (address, uint256));
        // Call receiveAndUnlock to process the unlocking and transfer of tokens
        receiveAndUnlock(receiver, amount);
        return bytes("");
    }

    function verifyIpcEnvelope(IpcEnvelope memory envelope) public {
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
            revert ZeroAddress();
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
            tokenContractAddress: tokenContractAddress,
            sender: msg.sender,
            destinationSubnet: destinationSubnet,
            destinationContract: destinationContract,
            receiver: receiver,
            nonce: lastNonce,
            value: amount
        });
        nonce++;

        uint256 startingBalance = IERC20(tokenContractAddress).balanceOf(address(this));
        IERC20(tokenContractAddress).safeTransferFrom({from: msg.sender, to: address(this), value: amount});
        uint256 endingBalance = IERC20(tokenContractAddress).balanceOf(address(this));

        if (endingBalance <= startingBalance) {
            revert NoTransfer();
        }

        CallMsg memory message = CallMsg({
            method: abi.encodePacked(bytes4(keccak256("receiveAndMint(address,uint256)"))),
            params: abi.encode(receiver, amount)
        });
        IPCAddress memory destination = IPCAddress({
            subnetId: destinationSubnet,
            rawAddress: FvmAddressHelper.from(destinationContract)
        });

        committed = performIpcCall(destination, message, DEFAULT_CROSS_MSG_FEE);

        //add receipt to unconfirmedTransfers
        unconfirmedTransfers[committed.toHash()] = TransferDetails(msg.sender, amount);
    }

    function _handleIpcResult(
        IpcEnvelope storage original,
        IpcEnvelope memory result,
        ResultMsg memory resultMsg
    ) internal override {}

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
