// SPDX-License-Identifier: MIT
pragma solidity 0.8.23;

import {Ownable} from "openzeppelin-contracts/access/Ownable.sol";
import {ISlasher} from "./ISlasher.sol";
import {DelegationManager} from "./DelegationManager.sol";
import {MarketAPI} from "filecoin-solidity/contracts/v0.8/MarketAPI.sol";
import {CommonTypes} from "filecoin-solidity/contracts/v0.8/types/CommonTypes.sol";
import {MarketTypes} from "filecoin-solidity/contracts/v0.8/types/MarketTypes.sol";

contract MinerSlasher is ISlasher, Ownable {
    mapping(uint64 => bool) public slashedDeals;

    DelegationManager public delegation;

    constructor(DelegationManager _delegation) Ownable(msg.sender) {
        delegation = _delegation;
    }

    function setDelegationManager(DelegationManager _delegation) external onlyOwner {
        delegation = _delegation;
    }

    function getDealClient(uint64 dealID) public view returns (int256, uint64) {
        return MarketAPI.getDealClient(dealID);
    }

    function getDealProvider(uint64 dealID) public view returns (int256, uint64) {
        return MarketAPI.getDealProvider(dealID);
    }

    function getDealTerm(uint64 dealID) public view returns (int256, MarketTypes.GetDealTermReturn memory) {
        return MarketAPI.getDealTerm(dealID);
    }

    function getDealDataCommitment(
        uint64 dealID
    ) public view returns (int256, MarketTypes.GetDealDataCommitmentReturn memory) {
        return MarketAPI.getDealDataCommitment(dealID);
    }

    function getDealLabel(uint64 dealID) public view returns (int256, CommonTypes.DealLabel memory) {
        return MarketAPI.getDealLabel(dealID);
    }

    function getDealProviderCollateral(uint64 dealID) public view returns (int256, CommonTypes.BigInt memory) {
        return MarketAPI.getDealProviderCollateral(dealID);
    }

    function getDealActivation(uint64 dealID) public view returns (int256, MarketTypes.GetDealActivationReturn memory) {
        return MarketAPI.getDealActivation(dealID);
    }

    function slash(address operator, uint64 dealID) external {
        // This check will be not be present in test mode to make sure it can be tested properly
        // require(slashedDeals[dealID] != true, "MinerSlasher: Deal is already slashed");
        int256 getProviderCode;
        uint64 dealProvider;
        (getProviderCode, dealProvider) = getDealProvider(dealID);
        require(getProviderCode == 0, "MinerSlasher: Failed to get deal provider");
        // TODO: get deal provider from operator and match it with the provider in the deal
        // For now, assume this to be true
        // require(dealProvider == delegate.getMinerId(operator), "MinerSlasher: Miner ID does not match operator's Miner ID");
        int256 getActivationCode;
        MarketTypes.GetDealActivationReturn memory activation;
        (getActivationCode, activation) = getDealActivation(dealID);
        require(getActivationCode == 0, "MinerSlasher: Failed to get deal activation status");
        // TODO: only check if terminated is 1
        // require(CommonTypes.ChainEpoch.unwrap(activation.activated) > 0, "MinerSlasher: Deal is not activated");
        // TODO: uncheck this after the a terminated deal is found on calibration
        // require(int64(activated) > 0, "MinerSlasher: Deal is not activated");
        delegation.slashOperator(operator);
        slashedDeals[dealID] = true;
    }
}
