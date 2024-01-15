// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity 0.8.19;

import "forge-std/Test.sol";
import {METHOD_SEND, EMPTY_BYTES} from "../constants/Constants.sol";
import {IpcEnvelope, ReceiptMsg, IpcMsg, IpcMsgKind} from "../structs/CrossNet.sol";
import {IPCMsgType} from "../enums/IPCMsgType.sol";
import {SubnetID, IPCAddress} from "../structs/Subnet.sol";
import {SubnetIDHelper} from "../lib/SubnetIDHelper.sol";
import {FvmAddressHelper} from "../lib/FvmAddressHelper.sol";
import {FvmAddress} from "../structs/FvmAddress.sol";
import {FilAddress} from "fevmate/utils/FilAddress.sol";
import {Address} from "openzeppelin-contracts/utils/Address.sol";
import {SupplySource} from "../structs/Subnet.sol";
import {SupplySourceHelper} from "./SupplySourceHelper.sol";

// Interface that needs to be implemented by IPC-enabled contracts.
// This is really convenient to call it from other contracts.
interface IpcContract {
    function IpcEntrypoint(IpcEnvelope calldata envelope) external payable returns (bytes memory);
}

/// @title Helper library for manipulating IpcEnvelope-related structs
library CrossMsgHelper {
    using SubnetIDHelper for SubnetID;
    using FilAddress for address;
    using FvmAddressHelper for FvmAddress;
    using SupplySourceHelper for SupplySource;

    error InvalidCrossMsgKind();
    error CannotExecuteEmptyEnvelope();

    function createTransferMsg(
        IPCAddress memory from,
        IPCAddress memory to,
        uint256 value,
        uint256 fee
    ) public pure returns (IpcEnvelope memory) {
        IpcMsg memory message = IpcMsg({value: value, method: METHOD_SEND, params: EMPTY_BYTES});
        return
            IpcEnvelope({
                kind: IpcMsgKind.Transfer,
                from: from,
                to: to,
                message: abi.encode(message),
                nonce: 0,
                fee: fee
            });
    }

    function createCallMsg(
        IPCAddress memory from,
        IPCAddress memory to,
        uint256 value,
        uint256 fee,
        bytes4 method,
        bytes memory params
    ) public pure returns (IpcEnvelope memory) {
        IpcMsg memory message = IpcMsg({value: value, method: method, params: params});
        return
            IpcEnvelope({kind: IpcMsgKind.Call, from: from, to: to, message: abi.encode(message), nonce: 0, fee: fee});
    }

    function createReleaseMsg(
        SubnetID calldata subnet,
        address signer,
        FvmAddress calldata to,
        uint256 value,
        uint256 fee
    ) public pure returns (IpcEnvelope memory) {
        return
            createTransferMsg(
                IPCAddress({subnetId: subnet, rawAddress: FvmAddressHelper.from(signer)}),
                IPCAddress({subnetId: subnet.getParentSubnet(), rawAddress: to}),
                value,
                fee
            );
    }

    function createFundMsg(
        SubnetID calldata subnet,
        address signer,
        FvmAddress calldata to,
        uint256 value,
        uint256 fee
    ) public pure returns (IpcEnvelope memory) {
        return
            createTransferMsg(
                IPCAddress({subnetId: subnet.getParentSubnet(), rawAddress: FvmAddressHelper.from(signer)}),
                IPCAddress({subnetId: subnet, rawAddress: to}),
                value,
                fee
            );
    }

    function applyType(IpcEnvelope calldata message, SubnetID calldata currentSubnet) public pure returns (IPCMsgType) {
        SubnetID memory toSubnet = message.to.subnetId;
        SubnetID memory fromSubnet = message.from.subnetId;
        SubnetID memory currentParentSubnet = currentSubnet.commonParent(toSubnet);
        SubnetID memory messageParentSubnet = fromSubnet.commonParent(toSubnet);

        if (currentParentSubnet.equals(messageParentSubnet)) {
            if (fromSubnet.route.length > messageParentSubnet.route.length) {
                return IPCMsgType.BottomUp;
            }
        }

        return IPCMsgType.TopDown;
    }

    function toHash(IpcEnvelope memory crossMsg) internal pure returns (bytes32) {
        return keccak256(abi.encode(crossMsg));
    }

    function toHash(IpcEnvelope[] memory crossMsgs) public pure returns (bytes32) {
        return keccak256(abi.encode(crossMsgs));
    }

    function isEmpty(IpcEnvelope memory crossMsg) internal pure returns (bool) {
        // envelopes need to necessarily include a message inside
        return crossMsg.message.length == 0;
    }

    function execute(IpcEnvelope calldata crossMsg, SupplySource memory supplySource) public returns (bytes memory) {
        if (isEmpty(crossMsg)) {
            revert CannotExecuteEmptyEnvelope();
        }
        if (crossMsg.kind == IpcMsgKind.Transfer || crossMsg.kind == IpcMsgKind.Call) {
            IpcMsg memory message = abi.decode(crossMsg.message, (IpcMsg));
            uint256 value = message.value;
            address recipient = crossMsg.to.rawAddress.extractEvmAddress().normalize();

            // if the message is of type transfer we can send it immediately
            if (crossMsg.kind == IpcMsgKind.Transfer) {
                supplySource.transfer({recipient: payable(recipient), value: value});
                return EMPTY_BYTES;
            } else {
                // send the envelope directly to the entrypoint
                // use supplySource so the tokens in the message are handled successfully
                // and by the right supply source
                return
                    supplySource.performCall(
                        payable(recipient),
                        abi.encodeCall(IpcContract.IpcEntrypoint, (crossMsg)),
                        value
                    );
            }
        } else if (crossMsg.kind == IpcMsgKind.Receipt) {
            address recipient = crossMsg.to.rawAddress.extractEvmAddress().normalize();
            // send the envelope directly to the entrypoint
            IpcContract ipcContract = IpcContract(recipient);
            return ipcContract.IpcEntrypoint(crossMsg);
        }

        return EMPTY_BYTES;
    }

    // This function requires deserializing the encoded message, if we are going
    // to access several fields of the message we are better-off using `getIpcMsg`
    // directly
    function getValue(IpcEnvelope calldata crossMsg) public pure returns (uint256) {
        // return 0 if empty, no need to decode anything
        if (isEmpty(crossMsg)) {
            return 0;
        }
        if (crossMsg.kind == IpcMsgKind.Transfer || crossMsg.kind == IpcMsgKind.Call) {
            IpcMsg memory message = abi.decode(crossMsg.message, (IpcMsg));
            return message.value;
        }
        // messages without value return 0
        return 0;
    }

    // get underlying IpcMsg from crossMsg
    function getIpcMsg(IpcEnvelope calldata crossMsg) public pure returns (IpcMsg memory ret) {
        if (isEmpty(crossMsg)) {
            return ret;
        }
        if (crossMsg.kind == IpcMsgKind.Call || crossMsg.kind == IpcMsgKind.Transfer) {
            IpcMsg memory message = abi.decode(crossMsg.message, (IpcMsg));
            return message;
        }

        // return empty IpcMsg otherwise
        return ret;
    }

    // set underlying IpcMsg from crossMsg.
    // This is a pure function, so the argument is not mutated
    function setIpcMsg(
        IpcEnvelope memory crossMsg,
        IpcMsg memory message
    ) public pure returns (IpcEnvelope memory ret) {
        if (crossMsg.kind == IpcMsgKind.Call || crossMsg.kind == IpcMsgKind.Transfer) {
            crossMsg.message = abi.encode(message);
            return crossMsg;
        }

        // Cannot set IPCMsg for the wrong kind
        revert InvalidCrossMsgKind();
    }

    // checks whether the cross messages are sorted in ascending order or not
    function isSorted(IpcEnvelope[] calldata crossMsgs) external pure returns (bool) {
        uint256 prevNonce;
        uint256 length = crossMsgs.length;
        for (uint256 i; i < length; ) {
            uint256 nonce = crossMsgs[i].nonce;

            if (prevNonce >= nonce) {
                // gas-opt: original check: i > 0
                if (i != 0) {
                    return false;
                }
            }

            prevNonce = nonce;
            unchecked {
                ++i;
            }
        }

        return true;
    }
}
