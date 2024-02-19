// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity 0.8.23;

import {SafeERC20} from "openzeppelin-contracts/token/ERC20/utils/SafeERC20.sol";
import {IERC20} from "openzeppelin-contracts/token/ERC20/IERC20.sol";
import {FvmAddressHelper} from "../../lib/FvmAddressHelper.sol";
import {FvmAddress} from "../../structs/FvmAddress.sol";
import {IpcExchange} from "../../../sdk/IpcContract.sol";
import {IpcEnvelope, ResultMsg, CallMsg, OutcomeType, IpcMsgKind} from "../../structs/CrossNet.sol";
import {IPCAddress, SubnetID} from "../../structs/Subnet.sol";
import {CrossMsgHelper} from "../../../src/lib/CrossMsgHelper.sol";
import {SubnetIDHelper} from "../../lib/SubnetIDHelper.sol";

/**
 * @title LinkedToken
 * @notice Contract to handle token transfer from L1, lock them and mint on L2.
 */
abstract contract LinkedToken is IpcExchange {
    using CrossMsgHelper for IpcEnvelope;
    using SubnetIDHelper for SubnetID;
    using FvmAddressHelper for FvmAddress;

    IERC20 public immutable _underlying;
    address public immutable _destinationContract;
    SubnetID public _destinationSubnet;

    mapping(bytes32 => UnconfirmedTransfer) public _unconfirmedTransfers;

    string private constant ERR_ZERO_ADDRESS = "zero address is not allowed";
    string private constant ERR_VALUE_MUST_BE_ZERO = "value must be zero";
    string private constant ERR_AMOUNT_CANNOT_BE_ZERO = "amount cannot be zero";

    error InvalidEnvelope(string reason);
    error TransferRejected(string reason);

    struct UnconfirmedTransfer {
        address sender;
        uint256 value;
    }

    event LinkedTokenDeployed(
        address indexed underlying,
        SubnetID indexed destinationSubnet,
        address indexed destinationContract
    );

    event LinkedTokensSent(
        address indexed underlying,
        address indexed sender,
        address indexed recipient,
        bytes32 id,
        uint64 nonce,
        uint256 value
    );

    event LinkedTokenReceived(address indexed recipient, uint256 amount, bytes32 id);

    /**
     * @dev Constructor for IpcTokenController
     * @param _gateway Address of the gateway for cross-network communication
     * @param _tokenContractAddress Address of the source ERC20 token contract
     * @param _destinationSubnet SubnetID of the destination network
     * @param _destinationContract Address of the destination contract for minting
     */
    constructor(
        address gateway,
        address underlyingToken,
        SubnetID memory destinationSubnet,
        address destinationContract
    ) IpcExchange(gateway) {
        _underlying = underlyingToken;
        _destinationSubnet = destinationSubnet;
        _destinationContract = destinationContract;

        emit LinkedTokenDeployed({
            underlying: _underlying,
            destinationSubnet: _destinationSubnet,
            destinationContract: _destinationContract
        });
    }

    function _captureTokens(address holder, uint256 amount) internal virtual;

    function _releaseTokens(address beneficiary, uint256 amount) internal virtual;

    /**
     * @notice Transfers tokens from L1, locks them, and requests minting on L2.
     * @param receiver Address to receive the minted tokens on L2
     * @param amount Amount of tokens to be transferred and minted
     */
    function linkedTransfer(address receiver, uint256 amount) external {
        _linkedTransfer(receiver, amount);
    }

    function _linkedTransfer(
        address recipient,
        uint256 amount
    ) internal returns (IpcEnvelope memory committed) {
        // Validate that the transfer parameters are acceptable.
        _validateTransfer(recipient, amount);

        // Lock or burn, depending on concrete implementation.
        _captureTokens(msg.sender, amount);

        // Pack the message to send to the other side of the linked token.
        CallMsg memory message = CallMsg({
            method: abi.encodePacked(bytes4(keccak256("receiveLinked(address,uint256)"))),
            params: abi.encode(recipient, amount)
        });
        IPCAddress memory destination = IPCAddress({
            subnetId: _destinationSubnet,
            rawAddress: FvmAddressHelper.from(_destinationContract)
        });

        // Route through GMP.
        committed = performIpcCall(destination, message, 0);

        // Record the unconfirmed transfer.
        _addUnconfirmedTransfer(committed.toHash(), msg.sender, amount);

        emit LinkedTokensSent({
            underlying: _underlying,
            sender: msg.sender,
            recipient: recipient,
            nonce: committed.nonce,
            value: amount
        });
    }

    // TODO make internal
    function lockAndTransferWithReturn(
        address receiver,
        uint256 amount
    ) external payable returns (IpcEnvelope memory envelope) {
        // Transfer and lock tokens on L1 using the inherited sendToken function
        return _linkedTransfer(receiver, amount);
    }

    // ----------------------------
    // IPC GMP entrypoints.
    // ----------------------------

    function _handleIpcCall(
        IpcEnvelope memory envelope,
        CallMsg memory callMsg
    ) internal override returns (bytes memory) {
        _validateEnvelope(envelope);
        _requireSelector(callMsg.method, "receiveLinked(address,uint256)");
        (address receiver, uint256 amount) = abi.decode(callMsg.params, (address, uint256));

        _receiveLinked(receiver, amount);
        return bytes("");
    }

    function _handleIpcResult(
        IpcEnvelope storage original,
        IpcEnvelope memory result,
        ResultMsg memory resultMsg
    ) internal override validateEnvelope(result) {
        bytes32 id = resultMsg.id;
        OutcomeType outcome = resultMsg.outcome;
        if (outcome == OutcomeType.Ok) {
            _removeUnconfirmedTransfer(id);
            return;
        }
        if (outcome == OutcomeType.SystemErr || outcome == OutcomeType.ActorErr) {
            _refundAndRemoveUnconfirmed(id);
        }
    }

    function _receiveLinked(address recipient, uint256 amount) private {
        _validateTransfer(recipient, amount);

        // Transfer the specified amount of tokens to the recipient
        _underlying.safeTransfer(recipient, amount);

        // Emit an event for the token unlock and transfer
        emit LinkedTokenReceived(recipient, amount);
    }

    // ----------------------------
    // Validation helpers.
    // ----------------------------

    // Only accept messages from our linked token contract.
    function _validateEnvelope(IpcEnvelope memory envelope) internal {
        SubnetID memory subnetId = envelope.from.subnetId;
        if (!_subnetId.equals(_destinationSubnet)) {
            revert InvalidOriginSubnet();
        }

        FvmAddress memory rawAddress = envelope.from.rawAddress;
        if (!rawAddress.equal(FvmAddressHelper.from(_destinationContract))) {
            revert InvalidOriginContract();
        }
    }

    function _requireSelector(bytes memory method, string memory signature) internal {
        if (method.length < 4) {
            revert InvalidEnvelope("short selector");
        }
        bytes4 coerced;
        assembly {
            coerced := mload(add(data, 32))
        }
        if (coerced != bytes4(keccak256(signature))) {
            revert InvalidEnvelope("invalid selector");
        }
    }

    function _validateTransfer(address receiver, uint256 amount) internal {
        if (receiver == address(0)) {
            revert TransferRejected(ERR_ZERO_ADDRESS);
        }
        if (amount == 0) {
            revert TransferRejected(ERR_AMOUNT_CANNOT_BE_ZERO);
        }
    }

    // ----------------------------
    // Unconfirmed transfers
    // ----------------------------

    function unconfirmedTransfer(bytes32 id) public view returns (address, uint256) {
        UnconfirmedTransfer storage details = _unconfirmedTransfers[id];
        return (details.sender, details.value);
    }

    // Method for the contract owner to manually drop an entry from unconfirmedTransfers
    function removeUnconfirmedTransfer(bytes32 id) external onlyOwner {
        _removeUnconfirmedTransfer(id);
    }

    function _refundAndRemoveUnconfirmed(bytes32 id) private {
        (address sender, uint256 value) = getUnconfirmedTransfer(id);
        require(sender != address(0), "internal error: no unconfirmed transfer to refund");

        _removeUnconfirmedTransfer(id);
        _releaseTokens(sender, value);
    }

    function _addUnconfirmedTransfer(bytes32 hash, address sender, uint256 value) internal {
        _unconfirmedTransfers[hash] = UnconfirmedTransferDetails(sender, value);
    }

    function _removeUnconfirmedTransfer(bytes32 id) internal {
        delete _unconfirmedTransfers[id];
    }

}
