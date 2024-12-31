// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.23;

import {NotEnoughBalance, InvalidSubnetActor} from "../errors/IPCErrors.sol";
import {Asset, AssetKind} from "../structs/Subnet.sol";
import {EMPTY_BYTES} from "../constants/Constants.sol";
import {IERC20} from "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import {SafeERC20} from "@openzeppelin/contracts/token/ERC20/utils/SafeERC20.sol";
import {ISubnetActor} from "../interfaces/ISubnetActor.sol";

/// @notice Helpers to deal with a supply source.
library AssetHelper {
    using SafeERC20 for IERC20;

    /// @notice Assumes that the address provided belongs to a subnet rooted on this network,
    ///         and checks if its supply kind matches the provided one.
    ///         It reverts if the address does not correspond to a subnet actor.
    function hasSupplyOfKind(address subnetActor, AssetKind compare) internal view returns (bool) {
        return ISubnetActor(subnetActor).supplySource().kind == compare;
    }

    /// @notice Checks that a given supply strategy is correctly formed and its preconditions are met.
    ///         It reverts if conditions are not met.
    function validate(Asset memory asset) internal view {
        if (asset.kind == AssetKind.ERC20) {
            require(asset.tokenAddress != address(0), "Invalid ERC20 address");
            // We require that the ERC20 token contract exists beforehand.
            // The call to balanceOf will revert if the supplied address does not exist, or if it's not an ERC20 contract.
            // Ideally we'd use ERC165 to check if the contract implements the ERC20 standard, but the latter does not support supportsInterface().
            IERC20 token = IERC20(asset.tokenAddress);
            token.balanceOf(address(0));
        }
    }

    /// @notice Asserts that the supply strategy is of the given kind. If not, it reverts.
    function expect(Asset memory asset, AssetKind kind) internal pure {
        require(asset.kind == kind, "Unexpected asset");
    }

    function equals(Asset memory asset, Asset memory asset2) internal pure returns (bool) {
        return asset.tokenAddress == asset2.tokenAddress && asset.kind == asset2.kind;
    }

    /// @notice Locks the specified amount from msg.sender into custody.
    ///         Reverts with NoBalanceIncrease if the token balance does not increase.
    ///         May return more than requested for inflationary tokens due to balance rise.
    function lock(Asset memory asset, uint256 value) internal returns (uint256) {
        if (asset.kind == AssetKind.ERC20) {
            IERC20 token = IERC20(asset.tokenAddress);

            // Dealing with deflationary tokens.
            uint256 initialBalance = token.balanceOf(address(this));
            token.safeTransferFrom({from: msg.sender, to: address(this), value: value});
            uint256 finalBalance = token.balanceOf(address(this));
            require(finalBalance > initialBalance, "No balance increase");
            // Safe arithmetic is not necessary because underflow is not possible due to the assert above
            return finalBalance - initialBalance;
        } else {
            // Ensure we have received enough funds to cover the value.
            // msg.value might have coins in excess of the amount that we need to lock (e.g. when contributing both native collateral and supply at the same time).
            // That's why we can't do a strict equality check.
            require(msg.value >= value, "Insufficient funds");
        }
        // Do nothing for native.
        return value;
    }

    /// @notice Transfers the specified amount out of our treasury to the recipient address.
    function transferFunds(Asset memory asset,
        address payable recipient,
        uint256 value
    ) internal returns (bool success, bytes memory ret) {
        if (asset.kind == AssetKind.Native) {
            success = sendValue(payable(recipient), value);
            return (success, EMPTY_BYTES);
        } else if (asset.kind == AssetKind.ERC20) {
            return ierc20Transfer(asset, recipient, value);
        }
    }

    /// @notice Wrapper for an IERC20 transfer that bubbles up the success or failure
    /// and the return value instead of reverting so a cross-message receipt can be
    /// triggered from the execution.
    /// This function is the same as `safeTransfer` function used before.
    function ierc20Transfer(
        Asset memory asset,
        address recipient,
        uint256 value
    ) internal returns (bool success, bytes memory ret) {
        return
            asset.tokenAddress.call(
        // using IERC20 transfer instead of safe transfer so we can
        // bubble-up the failure instead of reverting on failure so we
        // can send the receipt.
            abi.encodePacked(IERC20.transfer.selector, abi.encode(recipient, value))
        );
    }

    /// @notice Calls the target with the specified data, ensuring it receives the specified value.
    function performCall(
        Asset memory asset,
        address payable target,
        bytes memory data,
        uint256 value
    ) internal returns (bool success, bytes memory ret) {
        // If value is zero, we can just go ahead and call the function.
        if (value == 0) {
            return functionCallWithValue(target, data, 0);
        }

        // Otherwise, we need to do something different.
        if (asset.kind == AssetKind.Native) {
            // Use the optimized path to send value along with the call.
            (success, ret) = functionCallWithValue({target: target, data: data, value: value});
        } else if (asset.kind == AssetKind.ERC20) {
            (success, ret) = functionCallWithERC20Value({asset: asset, target: target, data: data, value: value});
        }
        return (success, ret);
    }

    /// @dev Performs the function call with ERC20 value atomically
    function functionCallWithERC20Value(
        Asset memory asset,
        address target,
        bytes memory data,
        uint256 value
    ) internal returns (bool success, bytes memory ret) {
        // Transfer the tokens first, _then_ perform the call.
        (success, ret) = ierc20Transfer(asset, target, value);

        if (success) {
            // Perform the call only if the ERC20 was successful.
            (success, ret) = functionCallWithValue(target, data, 0);
        }

        if (!success) {
            // following the implementation of `openzeppelin-contracts/utils/Address.sol`
            if (ret.length > 0) {
                assembly {
                    let returndata_size := mload(ret)
                // see https://ethereum.stackexchange.com/questions/133748/trying-to-understand-solidity-assemblys-revert-function
                    revert(add(32, ret), returndata_size)
                }
            }
            // disable solhint as the failing call does not have return data as well.
            /* solhint-disable reason-string */
            revert();
        }
        return (success, ret);
    }

    /// @notice Checks if the given address is a contract.
    function isContract(address account) internal view returns (bool) {
        uint256 size;
        assembly {
            size := extcodesize(account)
        }
        return size > 0;
    }

    /// @dev Adaptation from implementation `openzeppelin-contracts/utils/Address.sol`
    /// that doesn't revert immediately in case of failure and merely notifies of the outcome.
    function functionCallWithValue(
        address target,
        bytes memory data,
        uint256 value
    ) internal returns (bool success, bytes memory) {
        if (address(this).balance < value) {
            revert NotEnoughBalance();
        }

        if (!isContract(target)) {
            revert InvalidSubnetActor();
        }

        return target.call{value: value}(data);
    }

    /**
     *
     * @dev Adaptation from implementation `openzeppelin-contracts/utils/Address.sol`
     * so it doesn't revert immediately and bubbles up the success of the call
     *
     * Replacement for Solidity's `transfer`: sends `value` wei to
     * `recipient`, forwarding all available gas and reverting on errors.
     *
     * https://eips.ethereum.org/EIPS/eip-1884[EIP1884] increases the gas cost
     * of certain opcodes, possibly making contracts go over the 2300 gas limit
     * imposed by `transfer`, making them unable to receive funds via
     * `transfer`. {sendValue} removes this limitation.
     *
     * https://diligence.consensys.net/posts/2019/09/stop-using-soliditys-transfer-now/[Learn more].
     *
     * IMPORTANT: because control is transferred to `recipient`, care must be
     * taken to not create reentrancy vulnerabilities. Consider using
     * {ReentrancyGuard} or the
     * https://solidity.readthedocs.io/en/v0.5.11/security-considerations.html#use-the-checks-effects-interactions-pattern[checks-effects-interactions pattern].
     */
    function sendValue(address payable recipient, uint256 value) internal returns (bool) {
        if (address(this).balance < value) {
            revert NotEnoughBalance();
        }
        (bool success,) = recipient.call{value: value}("");
        return success;
    }

    /// @notice Gets the balance in our treasury.
    function balance(Asset memory asset) internal view returns (uint256 ret) {
        if (asset.kind == AssetKind.Native) {
            ret = address(this).balance;
        } else if (asset.kind == AssetKind.ERC20) {
            ret = IERC20(asset.tokenAddress).balanceOf(address(this));
        }
    }

    // @notice Gets the balance of the account.
    function balanceOf(Asset memory asset, address holder) internal view returns (uint256 ret) {
        if (asset.kind == AssetKind.Native) {
            ret = holder.balance;
        } else if (asset.kind == AssetKind.ERC20) {
            ret = IERC20(asset.tokenAddress).balanceOf(holder);
        }
    }

    // @notice Makes the asset available for spending by the given spender, without actually sending it.
    // @return msgValue The amount of msg.value that needs to be sent along with the subsequent call that will _actually_ spend that asset.
    //                  Will be 0 if the asset is a token, since no native coins are to be sent.
    function makeAvailable(Asset memory self, address spender, uint256 amount) internal returns (uint256 msgValue) {
        if (self.kind == AssetKind.Native) {
            msgValue = amount;
        } else if (self.kind == AssetKind.ERC20) {
            IERC20 token = IERC20(self.tokenAddress);
            token.safeIncreaseAllowance(spender, amount);
            msgValue = 0;
        }
        return msgValue;
    }

    function native() internal pure returns (Asset memory) {
        return Asset({kind: AssetKind.Native, tokenAddress: address(0)});
    }

    function erc20(address token) internal pure returns (Asset memory) {
        return Asset({kind: AssetKind.ERC20, tokenAddress: token});
    }
}