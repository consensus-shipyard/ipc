// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity ^0.8.23;

import {FvmAddress} from "../structs/FvmAddress.sol";
import {SubnetID, IPCAddress} from "../structs/Subnet.sol";
import {IpcEnvelope, IpcMsgKind, CallMsg} from "../structs/CrossNet.sol";
import {IGateway} from "../interfaces/IGateway.sol";
import {SubnetIDHelper} from "../lib/SubnetIDHelper.sol";
import {FvmAddressHelper} from "../lib/FvmAddressHelper.sol";
import {EMPTY_BYTES, METHOD_SEND} from "../../contracts/constants/Constants.sol";

interface ISubnetGetter {
    function ipcGatewayAddr() external view returns (address);
    function getParent() external view returns (SubnetID memory);
}

/// This is a simple example contract to invoke cross messages between subnets from different levels
contract CrossMessengeCaller {
    address public subnetActor;

    constructor(address _subnetActor) {
        subnetActor = _subnetActor;
    }

    function subnetId() public view returns (SubnetID memory id) {
        address actor = subnetActor;

        SubnetID memory parent = ISubnetGetter(actor).getParent();
        return SubnetIDHelper.createSubnetId(parent, actor);
    }

    /// @dev Invoke a cross net send fund message from the current subnet to the target subnet
    function invokeSendMessage(SubnetID calldata targetSubnet, address recipient, uint256 value) external {
        IPCAddress memory to = IPCAddress({subnetId: targetSubnet, rawAddress: FvmAddressHelper.from(recipient)});
        CallMsg memory message = CallMsg({method: abi.encodePacked(METHOD_SEND), params: EMPTY_BYTES});
        invokeCrossMessage(to, message, value);
    }

    function invokeCrossMessage(IPCAddress memory to, CallMsg memory callMsg, uint256 value) internal {
        IPCAddress memory from = IPCAddress({subnetId: subnetId(), rawAddress: FvmAddressHelper.from(address(this))});

        IpcEnvelope memory envelope = IpcEnvelope({
            kind: IpcMsgKind.Call,
            from: from,
            to: to,
            value: value,
            message: abi.encode(callMsg),
            nonce: 0
        });

        IGateway(ISubnetGetter(subnetActor).ipcGatewayAddr()).sendContractXnetMessage(envelope);
    }
}
