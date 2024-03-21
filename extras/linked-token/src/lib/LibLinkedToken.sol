// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity ^0.8.23;

import {LinkedTokenStorage, LibLinkedTokenStorage, UnconfirmedTransfer} from "./LibLinkedTokenStorage.sol";

library LibLinkedToken {

    function getLinkedGateway() internal view returns (address) {
        LinkedTokenStorage storage s = LibLinkedTokenStorage.appStorage();
        return s._gatewayAddr;
    }


    function getUnconfirmedTransfer(bytes32 id) internal view returns (address, uint256) {
        LinkedTokenStorage storage s = LibLinkedTokenStorage.appStorage();
        UnconfirmedTransfer storage details = s._unconfirmedTransfers[id];
        return (details.sender, details.value);
    }

}
