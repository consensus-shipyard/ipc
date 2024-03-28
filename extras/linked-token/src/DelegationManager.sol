// SPDX-License-Identifier: MIT
pragma solidity 0.8.23;

import {StrategyManager} from "./StrategyManager.sol";
import {ISlasher} from "./ISlasher.sol";
import {IStrategy} from "./IStrategy.sol";
import {ReentrancyGuard} from "openzeppelin-contracts/utils/ReentrancyGuard.sol";

contract DelegationManager is ReentrancyGuard {
    StrategyManager public strategyManager;
    ISlasher public slasher;

    struct OperatorDetails {
        address earningsReceiver;
        address delegationApprover;
        uint32 stakerOptOutDelayBlocks;
    }

    uint256 public constant MAX_WITHDRAWAL_DELAY_BLOCKS = 60;

    mapping(address => mapping(IStrategy => uint256)) public operatorShares;

    mapping(address => OperatorDetails) internal _operatorDetails;

    mapping(address => address) public delegatedTo;

    modifier onlyStrategyManager() {
        require(
            msg.sender == address(strategyManager),
            "DelegationManager: caller is not the StrategyManager"
        );
        _;
    }

    constructor(StrategyManager _strategyManager, ISlasher _slasher) {
        strategyManager = _strategyManager;
        slasher = _slasher;
    }

    function registerAsOperator(
        OperatorDetails calldata registeringOperatorDetails,
        string calldata metadataURI
    ) external {
        require(
            _operatorDetails[msg.sender].earningsReceiver == address(0),
            "DelegationManager.registerAsOperator: operator has already registered"
        );
        _setOperatorDetails(msg.sender, registeringOperatorDetails);
        _delegate(msg.sender, msg.sender);
    }

    function modifyOperatorDetails(
        OperatorDetails calldata newOperatorDetails
    ) external {
        require(
            isOperator(msg.sender),
            "DelegationManager.modifyOperatorDetails: caller is not an operator"
        );
        _setOperatorDetails(msg.sender, newOperatorDetails);
    }

    function delegateTo(address operator) external {
        _delegate(msg.sender, operator);
    }

    function increaseDelegatedShares(
        address staker,
        IStrategy strategy,
        uint256 shares
    ) external onlyStrategyManager {
        if (isDelegated(staker)) {
            address operator = delegatedTo[staker];
            _increaseOperatorShares({
                operator: operator,
                staker: staker,
                strategy: strategy,
                shares: shares
            });
        }
    }

    function decreaseDelegatedShares(
        address staker,
        IStrategy strategy,
        uint256 shares
    ) external onlyStrategyManager {
        if (isDelegated(staker)) {
            address operator = delegatedTo[staker];
            _decreaseOperatorShares({
                operator: operator,
                staker: staker,
                strategy: strategy,
                shares: shares
            });
        }
    }

    function _setOperatorDetails(
        address operator,
        OperatorDetails calldata newOperatorDetails
    ) internal {
        require(
            newOperatorDetails.earningsReceiver != address(0),
            "DelegationManager._setOperatorDetails: earningsReceiver cannot be 0"
        );
        _operatorDetails[operator] = newOperatorDetails;
    }

    function _delegate(address staker, address operator) internal {
        require(
            !isDelegated(staker),
            "DelegationManager._delegate: staker is already actively delegated"
        );
        require(
            isOperator(operator),
            "DelegationManager._delegate: operator is not registered in EigenLayer"
        );
        address _delegationApprover = _operatorDetails[operator]
            .delegationApprover;
        delegatedTo[staker] = operator;
        (
            IStrategy[] memory strategies,
            uint256[] memory shares
        ) = getDelegatableShares(staker);
        for (uint256 i = 0; i < strategies.length; i++) {
            _increaseOperatorShares({
                operator: operator,
                staker: staker,
                strategy: strategies[i],
                shares: shares[i]
            });
        }
    }

    function _increaseOperatorShares(
        address operator,
        address staker,
        IStrategy strategy,
        uint256 shares
    ) internal {
        operatorShares[operator][strategy] += shares;
    }

    function _decreaseOperatorShares(
        address operator,
        address staker,
        IStrategy strategy,
        uint256 shares
    ) internal {
        operatorShares[operator][strategy] -= shares;
    }

    function isDelegated(address staker) public view returns (bool) {
        return delegatedTo[staker] != address(0);
    }

    function isOperator(address operator) public view returns (bool) {
        return _operatorDetails[msg.sender].earningsReceiver == address(0);
    }

    function operatorDetails(
        address operator
    ) external view returns (OperatorDetails memory) {
        return _operatorDetails[operator];
    }

    function earningsReceiver(
        address operator
    ) external view returns (address) {
        return _operatorDetails[operator].earningsReceiver;
    }

    function delegationApprover(
        address operator
    ) external view returns (address) {
        return _operatorDetails[operator].delegationApprover;
    }

    function getOperatorShares(
        address operator,
        IStrategy[] memory strategies
    ) public view returns (uint256[] memory) {
        uint256[] memory shares = new uint256[](strategies.length);
        for (uint256 i = 0; i < strategies.length; ++i) {
            shares[i] = operatorShares[operator][strategies[i]];
        }
        return shares;
    }

    function getDelegatableShares(
        address staker
    ) public view returns (IStrategy[] memory, uint256[] memory) {
        (
            IStrategy[] memory strategyManagerStrats,
            uint256[] memory strategyManagerShares
        ) = strategyManager.getDeposits(staker);
        return (strategyManagerStrats, strategyManagerShares);
    }
}
