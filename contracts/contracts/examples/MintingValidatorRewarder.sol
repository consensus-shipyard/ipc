// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity ^0.8.23;

import {IValidatorRewarder} from "../interfaces/IValidatorRewarder.sol";
import {Consensus} from "../structs/Activity.sol";
import {SubnetID} from "../structs/Subnet.sol";
import {ERC20} from "@openzeppelin/contracts/token/ERC20/ERC20.sol";
import {IERC20} from "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import {Ownable} from "@openzeppelin/contracts/access/Ownable.sol";

contract MintableERC20 is ERC20, Ownable {
    constructor(string memory name, string memory symbol, address caller) ERC20(name, symbol) Ownable(msg.sender) {}

    function mint(address recipient, uint256 amount) onlyOwner external override {
        _mint(recipient, amount);
    }
}

/// An example validator rewarder implementation that mint ERC20 token for valiator
contract ValidatorRewarder is IValidatorRewarder, Ownable {
    address immutable public owner;
    SubnetID public subnetId;
    MintableERC20 public token;

    constructor() Ownable(msg.sender) {
        // We can also pass this address as a constructor parameter or update
        // using a setter as well.
        token = new MintableERC20("test", "TST", address(this));
    }

    function setSubnet(SubnetID calldata id) external {
        require(msg.sender == owner, "not owner");
        require(id.route.length > 0, "root not allowed");

        subnetId = id;
    }

    function notifyValidClaim(SubnetID calldata id, uint64 checkpointHeight, Consensus.ValidatorData calldata data) external {
        require(keccak256(abi.encode(id)) == keccak256(abi.encode(subnetId)), "not my subnet");

        address actor = id.route[id.route.length - 1];
        require(actor == msg.sender, "not from subnet");

        uint256 reward = calculateReward(data);

        token.mint(data.validator, reward);
    }

    /// @notice The internal method to derive the amount of reward that each validator should receive
    ///         based on their subnet activities
    function calculateReward(Consensus.ValidatorData calldata data) internal pure returns (uint256) {
        // Reward is the same as blocks mined for convenience.
        return data.blocksCommitted;
    }
}
