// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity ^0.8.23;

import {IValidatorRewarder} from "../activities/IValidatorRewarder.sol";
import {ValidatorSummary} from "../activities/Activity.sol";
import {SubnetID} from "../structs/Subnet.sol";
import {ERC20} from "@openzeppelin/contracts/token/ERC20/ERC20.sol";
import {IERC20} from "@openzeppelin/contracts/token/ERC20/IERC20.sol";

/// @notice Externally mintable ERC20 token
interface MintableERC20 is IERC20 {
    function mint(address recipient, uint256 amount) external;
}

contract ValidatorRewardERC20 is ERC20, MintableERC20 {
    /// @dev only minter can call mint
    address public minter;

    constructor(string memory name, string memory symbol, address caller) ERC20(name, symbol) {
        minter = caller;
    }

    function mint(address recipient, uint256 amount) external override {
        require(msg.sender == minter, "not minter");
        _mint(recipient, amount);
    }
}

/// An example validator rewarder implementation that mint ERC20 token for valiator
contract ValidatorRewarder is IValidatorRewarder {
    SubnetID public subnetId;
    address public owner;

    MintableERC20 public token;

    mapping(address => uint64) public blocksCommitted;

    constructor() {
        // We can also pass this address as a constructor parameter or update
        // using a setter as well.
        token = new ValidatorRewardERC20("test", "TST", address(this));
        owner = msg.sender;
    }

    function setSubnet(SubnetID calldata id) external {
        require(msg.sender == owner, "not owner");
        require(id.route.length > 0, "root not allowed");

        subnetId = id;
    }

    function disburseRewards(SubnetID calldata id, ValidatorSummary calldata summary) external {
        require(keccak256(abi.encode(id)) == keccak256(abi.encode(subnetId)), "not my subnet");

        address actor = id.route[id.route.length - 1];
        require(actor == msg.sender, "not from subnet");

        uint256 reward = calculateReward(summary);

        token.mint(summary.validator, reward);
    }

    /// @notice The internal method to derive the amount of reward that each validator should receive
    ///         based on their subnet activities
    function calculateReward(ValidatorSummary calldata summary) internal pure returns (uint256 reward) {
        // Reward is the same as blocks mined
        reward = summary.blocksCommitted;

        // Could also decode summary.metadata here to decode more data
        // In this example, the metadata is containing two extra custom reward params from child subnet
        if (summary.metadata.length != 0) {
            (uint256 storageReward, uint256 uptimeReward) = abi.decode(summary.metadata, (uint256, uint256));

            reward += storageReward;
            reward += uptimeReward;
        }
    }
}
