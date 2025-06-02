// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity ^0.8.23;

import "forge-std/Test.sol";
import "@openzeppelin/contracts/utils/Strings.sol";
import "../../contracts/lib/SubnetIDHelper.sol";

import {Asset, AssetKind} from "../../contracts/structs/Subnet.sol";
import {AssetHelper} from "../../contracts/lib/AssetHelper.sol";

import {AssetHelperMock} from "../mocks/AssetHelperMock.sol";
import {MockFallbackContract} from "../helpers/TestUtils.sol";

import {IERC20} from "@openzeppelin/contracts/token/ERC20/IERC20.sol";

import {ERC20PresetFixedSupply} from "../helpers/ERC20PresetFixedSupply.sol";

contract FailingContract {
    error BOOM();

    function failing() external pure {
        revert BOOM();
    }
}

contract AssetHelperTest is Test {
    /// Call fails but send value works, both should fail
    function test_revert_atomicity_no_ret() public {
        uint256 balance = 1_000_000;
        AssetHelperMock mock = new AssetHelperMock();

        IERC20 token = new ERC20PresetFixedSupply("TestToken", "TEST", balance, address(mock));

        Asset memory source = Asset({kind: AssetKind.ERC20, tokenAddress: address(token)});

        bytes memory params = bytes("hello");

        vm.expectRevert();
        mock.performCall(source, payable(address(this)), params, 100);

        require(token.balanceOf(address(mock)) == balance, "invalid balance");
    }

    function test_revert_atomicity_with_ret() public {
        uint256 balance = 1_000_000;
        AssetHelperMock mock = new AssetHelperMock();

        IERC20 token = new ERC20PresetFixedSupply("TestToken", "TEST", balance, address(mock));

        Asset memory source = Asset({kind: AssetKind.ERC20, tokenAddress: address(token)});

        bytes memory params = abi.encodeWithSelector(FailingContract.failing.selector);

        address c = address(new FailingContract());
        vm.expectRevert(FailingContract.BOOM.selector);
        mock.performCall(source, payable(c), params, 100);

        require(token.balanceOf(address(mock)) == balance, "invalid balance");
    }

    function test_call_with_erc20_ok() public {
        uint256 balance = 1_000_000;
        uint256 value = 100;
        AssetHelperMock mock = new AssetHelperMock();
        MockFallbackContract actor = new MockFallbackContract();

        IERC20 token = new ERC20PresetFixedSupply("TestToken", "TEST", balance, address(mock));

        Asset memory source = Asset({kind: AssetKind.ERC20, tokenAddress: address(token)});

        bytes memory params = bytes("hello");

        mock.performCall(source, payable(address(actor)), params, value);

        require(token.balanceOf(address(mock)) == balance - value, "invalid balance");
        require(token.balanceOf(address(actor)) == value, "invalid user balance");
    }

    function test_call_with_native_zero_balance_ok() public {
        uint256 value = 0;
        AssetHelperMock mock = new AssetHelperMock();
        MockFallbackContract actor = new MockFallbackContract();

        Asset memory source = Asset({kind: AssetKind.Native, tokenAddress: address(0)});

        bytes memory params = bytes("hello");

        mock.performCall(source, payable(address(actor)), params, value);
        require(address(actor).balance == 0, "invalid user balance");
    }

    function test_call_with_native_ok() public {
        uint256 value = 10;
        AssetHelperMock mock = new AssetHelperMock();
        MockFallbackContract actor = new MockFallbackContract();

        vm.deal(address(mock), 1 ether);

        Asset memory source = Asset({kind: AssetKind.Native, tokenAddress: address(0)});

        bytes memory params = bytes("hello");

        mock.performCall(source, payable(address(actor)), params, value);

        console.log("actor balance", address(actor).balance);
        require(address(actor).balance == value, "invalid user balance");
    }

    function test_call_with_native_reverts() public {
        uint256 value = 10;
        AssetHelperMock mock = new AssetHelperMock();

        vm.deal(address(mock), 1 ether);

        Asset memory source = Asset({kind: AssetKind.Native, tokenAddress: address(0)});

        bytes memory params = bytes("hello");

        mock.performCall(source, payable(address(this)), params, value);
        require(address(1).balance == 0, "invalid user balance");
    }
}
