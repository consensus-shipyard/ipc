// SPDX-License-Identifier: MIT
pragma solidity 0.8.23;

import {StrategyManager} from "./StrategyManager.sol";
import {IERC20} from "openzeppelin-contracts/token/ERC20/IERC20.sol";
import {SafeERC20} from "openzeppelin-contracts/token/ERC20/utils/SafeERC20.sol";
import {IStrategy} from "./IStrategy.sol";

contract StrategyBase is IStrategy {
    using SafeERC20 for IERC20;
    StrategyManager public strategyManager;
    IERC20 public underlyingToken;
    uint256 public totalShares;
    uint256 internal constant SHARES_OFFSET = 1e3;
    uint256 internal constant BALANCE_OFFSET = 1e3;
    modifier onlyStrategyManager() {
        require(
            msg.sender == address(strategyManager),
            "StrategyBase: caller is not the strategy manager"
        );
        _;
    }

    constructor(IERC20 _underlyingToken) {
        underlyingToken = _underlyingToken;
    }

    function deposit(
        IERC20 token,
        uint256 amount
    )
        external
        virtual
        override
        onlyStrategyManager
        returns (uint256 newShares)
    {
        _beforeDeposit(token, amount);
        uint256 priorTotalShares = totalShares;
        uint256 virtualShareAmount = priorTotalShares + SHARES_OFFSET;
        uint256 virtualTokenBalance = _tokenBalance() + BALANCE_OFFSET;
        uint256 virtualPriorTokenBalance = virtualTokenBalance - amount;
        newShares = (virtualShareAmount * amount) / virtualPriorTokenBalance;
        require(newShares != 0, "StrategyBase.deposit: newShares cannot be 0");
        totalShares = priorTotalShares + newShares;
        return newShares;
    }

    function withdraw(
        address recepient,
        IERC20 token,
        uint256 amountShares
    ) external virtual override onlyStrategyManager {
        _beforeWithdrawal(recepient, token, amountShares);
        uint256 priorTotalShares = totalShares;
        require(
            amountShares <= priorTotalShares,
            "StrategyBase.withdraw: amountShares exceeds totalShares"
        );
        uint256 virtualPriorTotalShares = priorTotalShares + SHARES_OFFSET;
        uint256 virtualTokenBalance = _tokenBalance() + BALANCE_OFFSET;
        uint256 amountToSend = (virtualTokenBalance * amountShares) /
            virtualPriorTotalShares;
        totalShares = priorTotalShares - amountShares;
        _afterWithdrawal(recepient, token, amountToSend);
    }

    function _beforeDeposit(IERC20 token, uint256 amount) internal virtual {
        require(
            token == underlyingToken,
            "StrategyBase.deposit: Can only deposit underlyingToken"
        );
    }

    function _beforeWithdrawal(
        address recipient,
        IERC20 token,
        uint256 amountShares
    ) internal virtual {
        require(
            token == underlyingToken,
            "StrategyBase.withdraw: Can only withdraw the strategy token"
        );
    }

    function _tokenBalance() internal view virtual returns (uint256) {
        return underlyingToken.balanceOf(address(this));
    }

    function _afterWithdrawal(
        address recipient,
        IERC20 token,
        uint256 amountToSend
    ) internal virtual {
        token.safeTransfer(recipient, amountToSend);
    }

    function explanation()
        external
        pure
        virtual
        override
        returns (string memory)
    {
        return
            "Base Strategy implementation to inherit from for more complex implementations";
    }

    function sharesToUnderlyingView(
        uint256 amountShares
    ) public view virtual override returns (uint256) {
        uint256 virtualTotalShares = totalShares + SHARES_OFFSET;
        uint256 virtualTokenBalance = _tokenBalance() + BALANCE_OFFSET;
        return (virtualTokenBalance * amountShares) / virtualTotalShares;
    }

    function sharesToUnderlying(
        uint256 amountShares
    ) public view virtual override returns (uint256) {
        return sharesToUnderlyingView(amountShares);
    }

    function underlyingToSharesView(
        uint256 amountUnderlying
    ) public view virtual returns (uint256) {
        uint256 virtualTotalShares = totalShares + SHARES_OFFSET;
        uint256 virtualTokenBalance = _tokenBalance() + BALANCE_OFFSET;
        return (amountUnderlying * virtualTotalShares) / virtualTokenBalance;
    }

    function underlyingToShares(
        uint256 amountUnderlying
    ) external view virtual returns (uint256) {
        return underlyingToSharesView(amountUnderlying);
    }

    function userUnderlyingView(
        address user
    ) external view virtual returns (uint256) {
        return sharesToUnderlyingView(shares(user));
    }

    function userUnderlying(address user) external virtual returns (uint256) {
        return sharesToUnderlying(shares(user));
    }

    function shares(address user) public view virtual returns (uint256) {
        return
            strategyManager.stakerStrategyShares(
                user,
                IStrategy(address(this))
            );
    }
}
