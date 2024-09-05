// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity ^0.8.23;

import "forge-std/Test.sol";
import "@openzeppelin/contracts/utils/Strings.sol";
import "../../contracts/lib/SubnetIDHelper.sol";

import {GenericToken, GenericTokenKind} from "../../contracts/structs/Subnet.sol";
import {GenericTokenHelper} from "../../contracts/lib/GenericTokenHelper.sol";

import {GenericTokenHelperMock} from "../mocks/GenericTokenHelperMock.sol";

import {IERC20} from "@openzeppelin/contracts/token/ERC20/IERC20.sol";

import {ERC20PresetFixedSupply} from "../helpers/ERC20PresetFixedSupply.sol";

contract FailingContract {
    error BOOM();

    function failing() external pure {
        revert BOOM();
    }
}

contract GenericTokenHelperTest is Test {
    /// Call fails but send value works, both should fail
    function test_revert_atomicity_no_ret() public {
        uint256 balance = 1_000_000;
        GenericTokenHelperMock mock = new GenericTokenHelperMock();

        IERC20 token = new ERC20PresetFixedSupply("TestToken", "TEST", balance, address(mock));

        GenericToken memory source = GenericToken({kind: GenericTokenKind.ERC20, tokenAddress: address(token)});

        bytes memory params = bytes("hello");

        vm.expectRevert();
        mock.performCall(source, payable(address(this)), params, 100);

        require(token.balanceOf(address(mock)) == balance, "invalid balance");
    }

    function test_revert_atomicity_with_ret() public {
        uint256 balance = 1_000_000;
        GenericTokenHelperMock mock = new GenericTokenHelperMock();

        IERC20 token = new ERC20PresetFixedSupply("TestToken", "TEST", balance, address(mock));

        GenericToken memory source = GenericToken({kind: GenericTokenKind.ERC20, tokenAddress: address(token)});

        bytes memory params = abi.encodeWithSelector(FailingContract.failing.selector);

        address c = address(new FailingContract());
        vm.expectRevert(FailingContract.BOOM.selector);
        mock.performCall(source, payable(c), params, 100);

        require(token.balanceOf(address(mock)) == balance, "invalid balance");
    }

    function test_call_with_erc20_ok() public {
        uint256 balance = 1_000_000;
        uint256 value = 100;
        GenericTokenHelperMock mock = new GenericTokenHelperMock();

        IERC20 token = new ERC20PresetFixedSupply("TestToken", "TEST", balance, address(mock));

        GenericToken memory source = GenericToken({kind: GenericTokenKind.ERC20, tokenAddress: address(token)});

        bytes memory params = bytes("hello");

        mock.performCall(source, payable(address(1)), params, value);

        require(token.balanceOf(address(mock)) == balance - value, "invalid balance");
        require(token.balanceOf(address(1)) == value, "invalid user balance");
    }

    function test_call_with_native_zero_balance_ok() public {
        uint256 value = 0;
        GenericTokenHelperMock mock = new GenericTokenHelperMock();

        GenericToken memory source = GenericToken({kind: GenericTokenKind.Native, tokenAddress: address(0)});

        bytes memory params = bytes("hello");

        mock.performCall(source, payable(address(1)), params, value);
        require(address(1).balance == 0, "invalid user balance");
    }

    function test_call_with_native_ok() public {
        uint256 value = 10;
        GenericTokenHelperMock mock = new GenericTokenHelperMock();

        vm.deal(address(mock), 1 ether);

        GenericToken memory source = GenericToken({kind: GenericTokenKind.Native, tokenAddress: address(0)});

        bytes memory params = bytes("hello");

        mock.performCall(source, payable(address(1)), params, value);
        require(address(1).balance == value, "invalid user balance");
    }

    function test_call_with_native_reverts() public {
        uint256 value = 10;
        GenericTokenHelperMock mock = new GenericTokenHelperMock();

        vm.deal(address(mock), 1 ether);

        GenericToken memory source = GenericToken({kind: GenericTokenKind.Native, tokenAddress: address(0)});

        bytes memory params = bytes("hello");

        mock.performCall(source, payable(address(this)), params, value);
        require(address(1).balance == 0, "invalid user balance");
    }
}
