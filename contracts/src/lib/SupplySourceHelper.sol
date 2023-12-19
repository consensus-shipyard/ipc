// SPDX-License-Identifier: UNLICENSED
pragma solidity 0.8.19;

import {SupplySource, SupplyKind} from "../structs/Subnet.sol";
import {IERC20} from "openzeppelin-contracts/token/ERC20/IERC20.sol";
import {Address} from "openzeppelin-contracts/utils/Address.sol";
import {SafeERC20} from "openzeppelin-contracts/token/ERC20/utils/SafeERC20.sol";
import {SubnetActorGetterFacet} from "../subnet/SubnetActorGetterFacet.sol";

/// @notice Helpers to deal with a supply source.
library SupplySourceHelper {
    using SafeERC20 for IERC20;

    error InvalidERC20Address();
    error UnexpectedSupplySource();
    error UnknownSupplySource();

    /// @notice Assumes that the address provided belongs to a subnet rooted on this network,
    ///         and checks if its supply kind matches the provided one.
    ///         It reverts if the address does not correspond to a subnet actor.
    function hasSupplyOfKind(address subnetActor, SupplyKind compare) internal view returns (bool) {
        return SubnetActorGetterFacet(subnetActor).supplySource().kind == compare;
    }

    /// @notice Checks that a given supply strategy is correctly formed and its preconditions are met.
    ///         It reverts if conditions are not met.
    function validate(SupplySource memory supplySource) internal view {
        if (supplySource.kind == SupplyKind.ERC20) {
            if (supplySource.tokenAddress == address(0)) {
                revert InvalidERC20Address();
            }
            // We require that the ERC20 token contract exists beforehand.
            // The call to balanceOf will revert if the supplied address does not exist, or if it's not an ERC20 contract.
            // Ideally we'd use ERC165 to check if the contract implements the ERC20 standard, but the latter does not support supportsInterface().
            IERC20 token = IERC20(supplySource.tokenAddress);
            token.balanceOf(address(0));
        }
    }

    /// @notice Asserts that the supply strategy is of the given kind. If not, it reverts.
    function expect(SupplySource memory supplySource, SupplyKind kind) internal pure {
        if (supplySource.kind != kind) {
            revert UnexpectedSupplySource();
        }
    }

    /// @notice Locks the specified amount sent by the msg.sender into custody.
    function lock(SupplySource memory supplySource, uint256 value) internal {
        if (supplySource.kind == SupplyKind.ERC20) {
            IERC20 token = IERC20(supplySource.tokenAddress);
            token.safeTransferFrom({from: msg.sender, to: address(this), value: value});
        }
        // Do nothing for native.
    }

    /// @notice Transfers the specified amount out of our treasury to the recipient address.
    function transfer(SupplySource memory supplySource, address payable recipient, uint256 value) internal {
        if (supplySource.kind == SupplyKind.Native) {
            Address.sendValue(payable(recipient), value);
        } else if (supplySource.kind == SupplyKind.ERC20) {
            IERC20(supplySource.tokenAddress).safeTransfer({to: recipient, value: value});
        }
    }

    /// @notice Calls the target with the specified data, ensuring it receives the specified value.
    function performCall(SupplySource memory supplySource, address payable target, bytes memory data, uint256 value) internal returns (bytes memory ret) {
        // If value is zero, we can just go ahead and call the function.
        if (value == 0) {
            ret = Address.functionCall(target, data);
        }

        // Otherwise, we need to do something different.
        if (supplySource.kind == SupplyKind.Native) {
            // Use the optimized path to send value along with the call.
            ret = Address.functionCallWithValue({target: target, data: data, value: value});
        } else if (supplySource.kind == SupplyKind.ERC20) {
            // Transfer the tokens first, _then_ perform the call.
            IERC20(supplySource.tokenAddress).safeTransfer({to: target, value: value});
            ret = Address.functionCall(target, data);
        }
        return ret;
    }

    /// @notice Gets the balance in our treasury.
    function balance(SupplySource memory supplySource) internal view returns (uint256 ret) {
        if (supplySource.kind == SupplyKind.Native) {
            ret = address(this).balance;
        } else if (supplySource.kind == SupplyKind.ERC20) {
            ret = IERC20(supplySource.tokenAddress).balanceOf(address(this));
        }
        return ret;
    }

    function native() internal pure returns (SupplySource memory) {
        return SupplySource({
            kind: SupplyKind.Native,
            tokenAddress: address(0)
        });
    }

}
