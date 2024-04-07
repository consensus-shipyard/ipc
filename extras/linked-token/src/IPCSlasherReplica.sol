// SPDX-License-Identifier: MIT
pragma solidity 0.8.23;

import {FvmAddressHelper} from "@ipc/src/lib/FvmAddressHelper.sol";
import {FvmAddress} from "@ipc/src/structs/FvmAddress.sol";
import {IpcExchange} from "@ipc/sdk/IpcContract.sol";
import {IpcEnvelope, ResultMsg, CallMsg, OutcomeType, IpcMsgKind} from "@ipc/src/structs/CrossNet.sol";
import {IPCAddress, SubnetID} from "@ipc/src/structs/Subnet.sol";
import {CrossMsgHelper} from "@ipc/src/lib/CrossMsgHelper.sol";
import {SubnetIDHelper} from "@ipc/src/lib/SubnetIDHelper.sol";

error InvalidOriginContract();
error InvalidOriginSubnet();

contract IPCSlasherReplica is IpcExchange {
    using CrossMsgHelper for IpcEnvelope;
    using SubnetIDHelper for SubnetID;
    using FvmAddressHelper for FvmAddress;

    address public _linkedContract;
    SubnetID public _linkedSubnet;

    mapping(bytes32 => UnconfirmedTransfer) public _unconfirmedTransfers;

    string private constant ERR_ZERO_ADDRESS = "zero address is not allowed";
    string private constant ERR_VALUE_MUST_BE_ZERO = "value must be zero";
    string private constant ERR_PROOF_CANNOT_BE_ZERO = "proof cannot be zero";

    error InvalidEnvelope(string reason);
    error TransferRejected(string reason);

    struct UnconfirmedTransfer {
        address operator;
        uint256 proof;
    }

    event LinkedSlasherInitialized(SubnetID indexed linkedSubnet, address indexed linkedContract);

    event LinkedSlashSent(address indexed operator, address indexed sender, bytes32 id, uint64 nonce, uint256 proof);

    constructor(address gateway, SubnetID memory linkedSubnet) IpcExchange(gateway) {
        _linkedSubnet = linkedSubnet;
    }

    function setLinkedContract(address linkedContract) external onlyOwner {
        _linkedContract = linkedContract;
    }

    function linkedTransfer(address operator, uint256 proof) external returns (IpcEnvelope memory committed) {
        return _linkedTransfer(operator, proof);
    }

    function _linkedTransfer(address operator, uint256 proof) internal returns (IpcEnvelope memory committed) {
        _validateInitialized();
        _validateTransfer(operator, proof);
        CallMsg memory message = CallMsg({
            method: abi.encodePacked(bytes4(keccak256("receiveLinked(address,uint256)"))),
            params: abi.encode(operator, proof)
        });
        IPCAddress memory destination = IPCAddress({
            subnetId: _linkedSubnet,
            rawAddress: FvmAddressHelper.from(_linkedContract)
        });
        committed = performIpcCall(destination, message, 0);
        _addUnconfirmedTransfer(committed.toHash(), operator, proof);
        emit LinkedSlashSent({
            sender: msg.sender,
            operator: operator,
            id: committed.toHash(),
            nonce: committed.nonce,
            proof: proof
        });
    }

    function transferSlashWithReturn(address operator, uint256 proof) external returns (IpcEnvelope memory envelope) {
        return _linkedTransfer(operator, proof);
    }

    // ----------------------------
    // Validation helpers.
    // ----------------------------

    function _validateInitialized() internal {
        require(_linkedContract != address(0), "linked token not initialized");
    }

    // Only accept messages from our linked token contract.
    // Made public for testing
    function _validateEnvelope(IpcEnvelope memory envelope) public {
        SubnetID memory subnetId = envelope.from.subnetId;
        if (!subnetId.equals(_linkedSubnet)) {
            revert InvalidOriginSubnet();
        }

        FvmAddress memory rawAddress = envelope.from.rawAddress;
        if (!rawAddress.equal(FvmAddressHelper.from(_linkedContract))) {
            revert InvalidOriginContract();
        }
    }

    function _requireSelector(bytes memory method, bytes memory signature) internal {
        if (method.length < 4) {
            revert InvalidEnvelope("short selector");
        }
        bytes4 coerced;
        assembly {
            coerced := mload(add(method, 32))
        }
        if (coerced != bytes4(keccak256(signature))) {
            revert InvalidEnvelope("invalid selector");
        }
    }

    function _validateTransfer(address operator, uint256 proof) internal {
        if (operator == address(0)) {
            revert TransferRejected(ERR_ZERO_ADDRESS);
        }
        if (proof == 0) {
            revert TransferRejected(ERR_PROOF_CANNOT_BE_ZERO);
        }
    }

    // GMP Unconfirmed Transfers

    function getUnconfirmedTransfer(bytes32 id) public view returns (address, uint256) {
        UnconfirmedTransfer storage details = _unconfirmedTransfers[id];
        return (details.operator, details.proof);
    }

    // Method for the contract owner to manually drop an entry from unconfirmedTransfers
    function removeUnconfirmedTransfer(bytes32 id) external onlyOwner {
        _removeUnconfirmedTransfer(id, false);
    }

    function _addUnconfirmedTransfer(bytes32 hash, address operator, uint256 proof) internal {
        _unconfirmedTransfers[hash] = UnconfirmedTransfer(operator, proof);
    }

    function _removeUnconfirmedTransfer(bytes32 id, bool refund) internal {
        (address operator, uint256 proof) = getUnconfirmedTransfer(id);
        delete _unconfirmedTransfers[id];
    }

    // ----------------------------
    // IPC GMP entrypoints.
    // ----------------------------

    function _handleIpcCall(
        IpcEnvelope memory envelope,
        CallMsg memory callMsg
    ) internal override returns (bytes memory) {
        _validateInitialized();
        _validateEnvelope(envelope);
        _requireSelector(callMsg.method, "receiveLinked(address,uint256)");

        (address operator, uint256 proof) = abi.decode(callMsg.params, (address, uint256));

        return bytes("");
    }

    function _handleIpcResult(
        IpcEnvelope storage original,
        IpcEnvelope memory result,
        ResultMsg memory resultMsg
    ) internal override {
        _validateInitialized();
        _validateEnvelope(result);

        OutcomeType outcome = resultMsg.outcome;
        bool refund = outcome == OutcomeType.SystemErr || outcome == OutcomeType.ActorErr;

        _removeUnconfirmedTransfer({id: resultMsg.id, refund: refund});
    }
}
