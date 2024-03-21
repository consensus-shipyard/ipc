// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity ^0.8.23;

import {IpcEnvelope} from "@ipc/src/structs/CrossNet.sol";
import {LinkedTokenStorage, LibLinkedTokenStorage, UnconfirmedTransfer} from "./LibLinkedTokenStorage.sol";
import {IERC20} from "openzeppelin-contracts/token/ERC20/IERC20.sol";
import {SubnetID} from "@ipc/src/structs/Subnet.sol";

library LibLinkedToken {

    function getLinkedGateway() internal view returns (address) {
        LinkedTokenStorage storage s = LibLinkedTokenStorage.appStorage();
        return s._gatewayAddr;
    }

    function getLinkedSubnet() internal view returns (SubnetID memory) {
        LinkedTokenStorage storage s = LibLinkedTokenStorage.appStorage();
        return s._linkedSubnet;
    }

    function getLinkedContract() internal view returns (address) {
        LinkedTokenStorage storage s = LibLinkedTokenStorage.appStorage();
        return s._linkedContract;
    }

    function setLinkedContract(address linkedContract) internal  {
        LinkedTokenStorage storage s = LibLinkedTokenStorage.appStorage();
        s._linkedContract = linkedContract;
    }



    function getUnderlyingToken() internal view returns (IERC20) {
        LinkedTokenStorage storage s = LibLinkedTokenStorage.appStorage();
        return s._underlying;
    }


    function addInflightMessages(bytes32 id, IpcEnvelope memory envelope) internal{
        LinkedTokenStorage storage s = LibLinkedTokenStorage.appStorage();
        s.inflightMsgs[id] = envelope;
    }

    function deleteInflightMessages(bytes32 id) internal{
        LinkedTokenStorage storage s = LibLinkedTokenStorage.appStorage();
        delete s.inflightMsgs[id];
    }

    function getInflightMessages(bytes32 id) internal returns (IpcEnvelope storage){
        LinkedTokenStorage storage s = LibLinkedTokenStorage.appStorage();
        return s.inflightMsgs[id];
    }


    function addUnconfirmedTransfer(bytes32 hash, address sender, uint256 value) internal  {
        LinkedTokenStorage storage s = LibLinkedTokenStorage.appStorage();
        s._unconfirmedTransfers[hash] = UnconfirmedTransfer(sender, value);
    }

    function deleteUnconfirmedTransfer(bytes32 id) internal {
        LinkedTokenStorage storage s = LibLinkedTokenStorage.appStorage();
        delete s._unconfirmedTransfers[id];
    }

    function getUnconfirmedTransfer(bytes32 id) internal view returns (address, uint256) {
        LinkedTokenStorage storage s = LibLinkedTokenStorage.appStorage();
        UnconfirmedTransfer storage details = s._unconfirmedTransfers[id];
        return (details.sender, details.value);
    }

}
