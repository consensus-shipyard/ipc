// SPDX-License-Identifier: MIT
pragma solidity 0.8.23;

import "../src/Glif.sol";
import "../src/Stfil.sol";
import "../src/CollectifDao.sol";
import "../src/Repl.sol";
import "../src/SftProtocol.sol";
import "../src/FiletFinance.sol";
import "./ConfigManager.sol";

contract DeployLSTTokens is ConfigManager {
    function run() external override {
        vm.startBroadcast();

        Glif glif = new Glif();
        Stfil stfil = new Stfil();
        CollectifDao collectifDao = new CollectifDao();
        Repl repl = new Repl();
        SftProtocol sftProtocol = new SftProtocol();
        FiletFinance filetFinance = new FiletFinance();

        vm.stopBroadcast();

        writeConfig("Glif", vm.toString(address(glif)));
        writeConfig("Stfil", vm.toString(address(stfil)));
        writeConfig("CollectifDao", vm.toString(address(collectifDao)));
        writeConfig("Repl", vm.toString(address(repl)));
        writeConfig("SftProtocol", vm.toString(address(sftProtocol)));
        writeConfig("FiletFinance", vm.toString(address(filetFinance)));
    }
}
