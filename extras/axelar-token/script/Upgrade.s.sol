// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "openzeppelin-contracts/proxy/transparent/TransparentUpgradeableProxy.sol";
import { Script, console2 as console } from "forge-std/Script.sol";
import "../src/IpcTokenHandler.sol";
import "../src/IpcTokenSender.sol";


contract Upgrade is Script {
    function setUp() public {}

    function upgradeTokenSenderProxy() public {

        string memory network = vm.envString("ORIGIN_NETWORK");
        uint256 privateKey = vm.envUint(string.concat(network, "__PRIVATE_KEY"));

        console.log("upgrading token sender on %s...", network);
        checkPathExists();
        string memory path = getPath();

        console.log("loading sender implementation address...");
        string memory readJson = vm.readFile(path);
        address newSenderAddrImplementation = vm.parseJsonAddress(readJson, ".src.token_sender_implementation");
        console.log("sender implementation address: %s", newSenderAddrImplementation);

        console.log("loading sender proxy address...");
        address senderAddr = vm.parseJsonAddress(readJson, ".src.token_sender");
        console.log("sender proxy address: %s", senderAddr);

        console.log("loading handler proxy address...");
        address handlerAddr = vm.parseJsonAddress(readJson, ".dest.token_handler");
        console.log("handler proxy address: %s", handlerAddr);

        address axelarIts= vm.envAddress(string.concat(network, "__AXELAR_ITS_ADDRESS"));
        string memory destinationChain= vm.envString(string.concat(network, "__AXELAR_CHAIN_NAME"));

        bytes memory initCall = abi.encodeCall(IpcTokenSender.reinitialize, (axelarIts, destinationChain, handlerAddr));

        vm.startBroadcast(privateKey);
        IpcTokenSender sender = IpcTokenSender(address(senderAddr));
        sender.upgradeToAndCall(newSenderAddrImplementation, initCall);
        vm.stopBroadcast();
        console.log("token sender proxy upgraded on %s: %s", network, address(sender));
    }


    function upgradeTokenHandlerProxy() public {

        string memory network = vm.envString("DEST_NETWORK");
        uint256 privateKey = vm.envUint(string.concat(network, "__PRIVATE_KEY"));

        console.log("upgrading token handler on %s...", network);
        checkPathExists();
        string memory path = getPath();

        console.log("loading handler implementation address...");
        string memory readJson = vm.readFile(path);
        address newHandlerAddrImplementation = vm.parseJsonAddress(readJson, ".dest.token_handler_implementation");
        console.log("handler implementation address: %s", newHandlerAddrImplementation);

        console.log("loading handler proxy address...");
        address handlerAddr = vm.parseJsonAddress(readJson, ".dest.token_handler");
        console.log("handler proxy address: %s", handlerAddr);



        address axelarIts= vm.envAddress(string.concat(network, "__AXELAR_ITS_ADDRESS"));
        address ipcGateway= vm.envAddress(string.concat(network, "__IPC_GATEWAY_ADDRESS"));

        bytes memory initCall = abi.encodeCall(IpcTokenHandler.reinitialize, (axelarIts, ipcGateway));

        vm.startBroadcast(privateKey);
        IpcTokenHandler handler = IpcTokenHandler(address(handlerAddr));
        handler.upgradeToAndCall(newHandlerAddrImplementation, initCall);
        vm.stopBroadcast();
        console.log("token handler proxy upgraded on %s: %s", network, address(handler));
    }

    function getPath() public returns (string memory path) {
            path = string.concat(vm.projectRoot(), "/out/addresses.json");
            if (!vm.exists(path)) {
                vm.writeJson("{\"dest\":{\"token_handler\":{}, \"token_handler_implementation\":{} },\"src\":{\"token_sender\":{}, \"token_sender_implementation\":{}}}", path);
            }
    }

    function checkPathExists() public {
        string memory path = string.concat(vm.projectRoot(), "/out/addresses.json");
        require(vm.exists(path), "no addresses.json; please run DeployTokenHandler on the destination chain");
    }




}

