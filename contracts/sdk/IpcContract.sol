// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity 0.8.19;

import {IpcEnvelope, ReceiptMsg, IpcMsg, IpcMsgKind} from "../src/structs/CrossNet.sol";
import {EMPTY_BYTES} from "../src/constants/Constants.sol";
import {IGateway} from "../src/interfaces/IGateway.sol";

// Interface that needs to be implemented by IPC-enabled contracts.
// This is really convenient to call it from other contracts.
interface IfaceIpcContract {
    /// @notice Entrypoint for cross-net messages in IPC-enabled contracts.
    // solhint-disable-next-line func-name-mixedcase
    function IpcEntrypoint(IpcEnvelope calldata crossMsg) external payable returns (bytes memory ret);
}

abstract contract IpcContract {
    // list of messages in-flight for which the contract
    // hasn't received an ACK.
    mapping(bytes32 => IpcEnvelope) public inFlightMsgs;
    // auxiliary mapping to track the height at which certain
    // in-flight messages were received.
    mapping(uint256 => bytes32[]) public inFlightMsgHeights;
    // last height where there messages were pruned
    uint256 private lastGcBlock;

    // The adderss of the gateway in the subnet.
    address public gatewayAddr;
    /// Number of blocks that the contract will wait for a receipt
    /// before garbage collecting the in-flight message.
    uint256 public timeoutBlocks;

    constructor(address gatewayAddr_, uint256 timeoutBlocks_) {
        gatewayAddr = gatewayAddr_;
        timeoutBlocks = timeoutBlocks_;
    }

    error CallerIsNotGateway();
    error IpcMsgCallFailed();
    error UnsupportedMsgKind();
    error UnrecognizedReceipt();

    /// @notice Entrypoint for IPC-enabled contracts. This function is always called by
    /// the gateway when a `Call` or `Receipt` cross-net messages is targeted to
    /// a specific address in the subnet.
    function IpcEntrypoint(IpcEnvelope calldata crossMsg) external returns (bytes memory) {
        // trigger the garbage collection of in-flight messages
        // in case any timeout has triggered already.
        // timeout == 0 means that no in-flight expiraton is set for receipts.
        if (timeoutBlocks != 0) {
            inFlightGC();
        }

        // only the gateway address is allowed to deliver cross-net messages.
        if (msg.sender != gatewayAddr) {
            revert CallerIsNotGateway();
        }

        // internal dispatch of the cross-net message to the right method.
        if (crossMsg.kind == IpcMsgKind.Call) {
            IpcMsg memory callMsg = abi.decode(crossMsg.message, (IpcMsg));

            (bool success, bytes memory ret) = address(this).delegatecall(
                abi.encodeWithSelector(callMsg.method, callMsg.params)
            );
            if (!success) {
                revert IpcMsgCallFailed();
            }

            return ret;
        } else if (crossMsg.kind == IpcMsgKind.Receipt) {
            ReceiptMsg memory ackMsg = abi.decode(crossMsg.message, (ReceiptMsg));

            // get the original message and chekc if the receipt was generated
            // by the target contract, and that we actually sent that cross-net message.
            // If this is not the case, revert, as someone may be trying to do something
            // sketchy.
            IpcEnvelope memory originalMsg = inFlightMsgs[ackMsg.id];
            if (
                originalMsg.message.length == 0 ||
                keccak256(abi.encode(crossMsg.from)) != keccak256(abi.encode(originalMsg.to))
            ) {
                revert UnrecognizedReceipt();
            }
            // process ACK message and remove from in-flight if it succeeds.
            if (AckEntrypoint(ackMsg)) {
                delete inFlightMsgs[ackMsg.id];
            }
        } else {
            revert UnsupportedMsgKind();
        }

        return EMPTY_BYTES;
    }

    /// @notice Function that needs to be called internally by the contract to send a
    /// cross-net message.
    /// TODO: Consider a more Solidity-friendly interface instead of having to pass
    /// the full envelope?
    function XnetCall(IpcEnvelope memory crossMsg) internal virtual {
        // Queue the cross-net message for propagation.
        IGateway(gatewayAddr).sendContractXnetMessage(crossMsg);

        // Add the message to the list of in-flights
        bytes32 msgID = keccak256(abi.encode(crossMsg));
        inFlightMsgs[msgID] = crossMsg;
        inFlightMsgHeights[block.number].push(msgID);
    }

    /// @notice Garbage collect in-flight messages for which the
    /// timeout has been reached.
    // TODO: We could probably abstract this logic into a helper
    // with the contract variables for readibility and convenience.
    function inFlightGC() internal {
        // if the timeout hasn't been reached, nothing to do
        if (block.number > lastGcBlock + timeoutBlocks) {
            // garbage collect from lastGcBlock to the latest height that is
            // garbage collectable.
            // e.g. lastGcBlock = 100; timeoutBlocks = 10; block.number = 120
            // Garbage collect: [100, ..., 110]
            for (uint256 i = lastGcBlock; i <= block.number - timeoutBlocks; ) {
                bytes32[] memory msgIds = inFlightMsgHeights[i];
                // remove the specific messages and the heigth trakcer
                for (uint256 j = 0; i < msgIds.length; ) {
                    delete inFlightMsgs[msgIds[j]];
                    unchecked {
                        ++j;
                    }
                }
                delete inFlightMsgHeights[i];
                unchecked {
                    ++i;
                }
            }
            // track last garbage collection height
            lastGcBlock = block.number;
        }
    }

    /// @notice Implements the logic of the contract when it receives a receipt.
    /// It should handle success and failures from calls to cross-net messages.
    /// When a failure receipt is received, the side-effects triggered through
    /// the cross-net call should be compensated.
    /* solhint-disable-next-line unused-vars */
    function AckEntrypoint(ReceiptMsg memory receipt) internal virtual returns (bool) {
        // This function is use-case specific and should be overwritten
        // explicitly by contract developers.
        // It should explicitly handle the compensation from a NAck
        // (ACKs may not need an explicit handler in many cases).
        return true;
    }
}
