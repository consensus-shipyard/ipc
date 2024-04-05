// SPDX-License-Identifier: MIT
pragma solidity 0.8.23;

import {FvmAddressHelper} from "@ipc/src/lib/FvmAddressHelper.sol";
import {FvmAddress} from "@ipc/src/structs/FvmAddress.sol";
import {IpcExchange} from "@ipc/sdk/IpcContract.sol";
import {IpcEnvelope, ResultMsg, CallMsg, OutcomeType, IpcMsgKind} from "@ipc/src/structs/CrossNet.sol";
import {IPCAddress, SubnetID} from "@ipc/src/structs/Subnet.sol";
import {CrossMsgHelper} from "@ipc/src/lib/CrossMsgHelper.sol";
import {SubnetIDHelper} from "@ipc/src/lib/SubnetIDHelper.sol";
import {DelegationManager} from "./DelegationManager.sol";
import {ISlasher} from "./ISlasher.sol";

error InvalidOriginContract();
error InvalidOriginSubnet();

contract IPCSlasherController is IpcExchange, ISlasher {
    using CrossMsgHelper for IpcEnvelope;
    using SubnetIDHelper for SubnetID;
    using FvmAddressHelper for FvmAddress;

    DelegationManager public delegation;

    struct UnconfirmedTransfer {
        address operator;
        uint256 proof;
    }

    // address public _linkedContract;
    // SubnetID public _linkedSubnet;

    mapping(bytes32 => UnconfirmedTransfer) public _unconfirmedTransfers;

    string private constant ERR_ZERO_ADDRESS = "zero address is not allowed";
    string private constant ERR_VALUE_MUST_BE_ZERO = "value must be zero";
    string private constant ERR_PROOF_CANNOT_BE_ZERO = "proof cannot be zero";

    error InvalidEnvelope(string reason);
    error TransferRejected(string reason);

    event LinkedTokenInitialized(
        address indexed underlying,
        SubnetID indexed linkedSubnet,
        address indexed linkedContract
    );

    event LinkedTokensSent(
        address indexed underlying,
        address indexed sender,
        address indexed recipient,
        bytes32 id,
        uint64 nonce,
        uint256 value
    );

    event OperatorSlashed(address indexed operator, uint256 proof);

    event LinkedSlashReceived(address indexed operator, uint256 proof);

    constructor(address gateway) IpcExchange(gateway) {}

    function slash(address operator, uint256 proof) public {
        delegation.slashOperator(operator);
        emit OperatorSlashed(operator, proof);
    }

    function setDelegationManager(DelegationManager _delegation) external override {
        delegation = _delegation;
    }

    // --------------------------------
    // GMP Entry points
    // --------------------------------

    function _handleIpcCall(
        IpcEnvelope memory envelope,
        CallMsg memory callMsg
    ) internal override returns (bytes memory) {
        _validateInitialized();
        _validateEnvelope(envelope);
        _requireSelector(callMsg.method, "receiveLinked(address,uint256)");

        (address operator, uint256 proof) = abi.decode(callMsg.params, (address, uint256));

        _receiveLinked(operator, proof);
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

    function _receiveLinked(address operator, uint256 proof) private {
        _validateTransfer(operator, proof);

        slash(operator, proof);

        // Emit an event for the token unlock and transfer
        emit LinkedSlashReceived(operator, proof);
    }

    // ----------------------------
    // GMP Validation helpers
    // ----------------------------

    function _validateInitialized() internal {
        // No need to initialize linked token since we are accepting from all subnets
        // require(_linkedContract != address(0), "linked token not initialized");
    }

    // Accept messages from all subnets
    // Made public for testing
    function _validateEnvelope(IpcEnvelope memory envelope) public {
        SubnetID memory subnetId = envelope.from.subnetId;
        // allow messages from all subnets
        // if (!subnetId.equals(_linkedSubnet)) {
        //     revert InvalidOriginSubnet();
        // }

        FvmAddress memory rawAddress = envelope.from.rawAddress;
        // allow messages from all raw addresses
        // if (!rawAddress.equal(FvmAddressHelper.from(_linkedContract))) {
        //     revert InvalidOriginContract();
        // }
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
        (address sender, uint256 value) = getUnconfirmedTransfer(id);
        delete _unconfirmedTransfers[id];
    }
}
