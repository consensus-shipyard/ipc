// SPDX-License-Identifier: MIT
pragma solidity 0.8.23;

import {IERC20} from "openzeppelin-contracts/token/ERC20/IERC20.sol";

interface IStrategy {
    function deposit(IERC20 token, uint256 amount) external returns (uint256);

    function withdraw(
        address recipient,
        IERC20 token,
        uint256 amountShares
    ) external;

    function sharesToUnderlying(
        uint256 amountShares
    ) external returns (uint256);

    function underlyingToShares(
        uint256 amountUnderlying
    ) external returns (uint256);

    function userUnderlying(address user) external returns (uint256);

    function shares(address user) external view returns (uint256);

    function sharesToUnderlyingView(
        uint256 amountShares
    ) external view returns (uint256);

    function underlyingToSharesView(
        uint256 amountUnderlying
    ) external view returns (uint256);

    function userUnderlyingView(address user) external view returns (uint256);

    function underlyingToken() external view returns (IERC20);

    function totalShares() external view returns (uint256);

    function explanation() external view returns (string memory);
}
