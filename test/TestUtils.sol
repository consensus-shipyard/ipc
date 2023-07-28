// SPDX-License-Identifier: UNLICENSED
pragma solidity 0.8.19;

import "forge-std/Test.sol";

library TestUtils {
    function generateSelectors(Vm vm, string memory facetName) internal returns (bytes4[] memory facetSelectors) {
        string[] memory inputs = new string[](3);
        inputs[0] = "python3";
        inputs[1] = "scripts/python/get_selectors.py";
        inputs[2] = facetName;

        bytes memory res = vm.ffi(inputs);
        facetSelectors = abi.decode(res, (bytes4[]));
    }
}
