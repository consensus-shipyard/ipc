// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity 0.8.19;

import "forge-std/Test.sol";
import "openzeppelin-contracts/utils/Strings.sol";
import "../../src/lib/SubnetIDHelper.sol";

import {SupplySource, SupplyKind} from "../../src/structs/Subnet.sol";
import {SupplySourceHelper} from "../../src/lib/SupplySourceHelper.sol";

import {SupplySourceHelperMock} from "../mocks/SupplySourceHelperMock.sol";

import {IERC20} from "openzeppelin-contracts/token/ERC20/IERC20.sol";

import {ERC20PresetFixedSupply} from "../helpers/ERC20PresetFixedSupply.sol";

contract SupplySourceHelperTest is Test {
    /// Call fails but send value works, both should fail
    function test_revert_atomicity() public {
        SupplySourceHelperMock mock = new SupplySourceHelperMock();

        IERC20 token = new ERC20PresetFixedSupply("TestToken", "TEST", 1_000_000, address(mock));

        SupplySource memory source = SupplySource({kind: SupplyKind.ERC20, tokenAddress: address(token)});

        bytes memory params = bytes("hello");

        mock.functionCallWithERC20Value(source, address(1), params, 100);
    }
}
