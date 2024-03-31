// SPDX-License-Identifier: MIT
pragma solidity 0.8.23;

import {Ownable} from "openzeppelin-contracts/access/Ownable.sol";
import {SafeERC20} from "openzeppelin-contracts/token/ERC20/utils/SafeERC20.sol";
import {IERC20} from "openzeppelin-contracts/token/ERC20/IERC20.sol";
import {ReentrancyGuard} from "openzeppelin-contracts/utils/ReentrancyGuard.sol";
import {DelegationManager} from "./DelegationManager.sol";
import {IStrategy} from "./IStrategy.sol";

contract StrategyManager is Ownable, ReentrancyGuard {
    using SafeERC20 for IERC20;
    DelegationManager public delegation;
    mapping(address => mapping(IStrategy => uint256))
        public stakerStrategyShares;
    mapping(address => IStrategy[]) public stakerStrategyList;
    mapping(IStrategy => bool) public strategyIsWhitelistedForDeposit;

    modifier onlyDelegationManager() {
        require(
            msg.sender == address(delegation),
            "StrategyManager: caller is not the DelegationManager"
        );
        _;
    }

    modifier onlyStrategiesWhitelistedForDeposit(IStrategy strategy) {
        require(
            strategyIsWhitelistedForDeposit[strategy],
            "StrategyManager.onlyStrategiesWhitelistedForDeposit: strategy not whitelisted"
        );
        _;
    }

    constructor() Ownable(msg.sender) {}

    function setDelegationManager(
        DelegationManager _delegation
    ) external onlyOwner {
        delegation = _delegation;
    }

    function depositIntoStrategy(
        IStrategy strategy,
        IERC20 token,
        uint256 amount
    ) external nonReentrant returns (uint256 shares) {
        shares = _depositIntoStrategy(msg.sender, strategy, token, amount);
    }

    function removeShares(
        address staker,
        IStrategy strategy,
        uint256 shares
    ) external onlyDelegationManager {
        _removeShares(staker, strategy, shares);
    }

    function addShares(
        address staker,
        IERC20 token,
        IStrategy strategy,
        uint256 shares
    ) external onlyDelegationManager {
        _addShares(staker, token, strategy, shares);
    }

    function withdrawSharesAsTokens(
        address recepient,
        IStrategy strategy,
        uint256 shares,
        IERC20 token
    ) external onlyDelegationManager {
        strategy.withdraw(recepient, token, shares);
    }

    function addStrategiesToDepositWhitelist(
        IStrategy[] calldata strategiesToWhitelist
    ) external onlyOwner {
        uint256 strategiesToWhitelistLength = strategiesToWhitelist.length;
        for (uint256 i = 0; i < strategiesToWhitelistLength; i++) {
            strategyIsWhitelistedForDeposit[strategiesToWhitelist[i]] = true;
        }
    }

    function _addShares(
        address staker,
        IERC20 token,
        IStrategy strategy,
        uint256 shares
    ) internal {
        require(
            staker != address(0),
            "StrategyManager._addShares: staker cannot be zero address"
        );
        require(
            shares != 0,
            "StrategyManager._addShares: shares should not be zero!"
        );
        if (stakerStrategyShares[staker][strategy] == 0) {
            stakerStrategyList[staker].push(strategy);
        }
        stakerStrategyShares[staker][strategy] += shares;
    }

    function _depositIntoStrategy(
        address staker,
        IStrategy strategy,
        IERC20 token,
        uint256 amount
    )
        internal
        onlyStrategiesWhitelistedForDeposit(strategy)
        returns (uint256 shares)
    {
        token.safeTransferFrom(msg.sender, address(strategy), amount);
        shares = strategy.deposit(token, amount);
        _addShares(staker, token, strategy, shares);
        // TODO
        delegation.increaseDelegatedShares(staker, strategy, shares);
        return shares;
    }

    function _removeShares(
        address staker,
        IStrategy strategy,
        uint256 shareAmount
    ) internal returns (bool) {
        require(
            shareAmount != 0,
            "StrategyManager._removeShares: shareAmount should not be zero!"
        );
        uint256 userShares = stakerStrategyShares[staker][strategy];
        require(
            shareAmount <= userShares,
            "StrategyManager._removeShares: shareAmount too high"
        );
        userShares = userShares - shareAmount;
        stakerStrategyShares[staker][strategy] = userShares;
        if (userShares == 0) {
            _removeStrategyFromStakerStrategyList(staker, strategy);
            return true;
        }
        return false;
    }

    function _removeStrategyFromStakerStrategyList(
        address staker,
        IStrategy strategy
    ) internal {
        uint256 stratsLength = stakerStrategyList[staker].length;
        uint256 j = 0;
        for (; j < stratsLength; ) {
            if (stakerStrategyList[staker][j] == strategy) {
                stakerStrategyList[staker][j] = stakerStrategyList[staker][
                    stakerStrategyList[staker].length - 1
                ];
                break;
            }
            unchecked {
                ++j;
            }
        }
        require(
            j != stratsLength,
            "StrategyManager._removeStrategyFromStakerStrategyList: strategy not found"
        );
        stakerStrategyList[staker].pop();
    }

    function getDeposits(
        address staker
    ) external view returns (IStrategy[] memory, uint256[] memory) {
        uint256 strategiesLength = stakerStrategyList[staker].length;
        uint256[] memory shares = new uint256[](strategiesLength);

        for (uint256 i = 0; i < strategiesLength; ) {
            shares[i] = stakerStrategyShares[staker][
                stakerStrategyList[staker][i]
            ];
            unchecked {
                ++i;
            }
        }
        return (stakerStrategyList[staker], shares);
    }

    function stakerStrategyListLength(
        address staker
    ) external view returns (uint256) {
        return stakerStrategyList[staker].length;
    }
}
