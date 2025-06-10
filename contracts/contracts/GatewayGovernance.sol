// SPDX-License-Identifier: MIT
pragma solidity ^0.8.23;

import {Governor} from "@openzeppelin/contracts/governance/Governor.sol";
import {GovernorStorage} from "@openzeppelin/contracts/governance/extensions/GovernorStorage.sol";
import {EnumerableSet} from "@openzeppelin/contracts/utils/structs/EnumerableSet.sol";

import {Membership} from "./structs/Subnet.sol";
import {IGateway} from "./interfaces/IGateway.sol";

/// @notice The governance contract for gateway diamond. This contract holds the ownership
/// of the gateway diamond, any upgrades or admin related operations must go through this
/// contract.
///
/// This contract is used for L2 subnet gateway deployed from fendermint node.
///
/// To create a proposal, this implementation follows that of Governor proposal.
/// However to vote for a proposal, validators should just call `castVote(proposalId)`,
/// no vote is assumed to be against the proposal. This means once a quorum is reached,
/// the voting is successful. This simplifies the Governor flow.
contract GatewayGovernance is GovernorStorage {
    using EnumerableSet for EnumerableSet.AddressSet;

    error NotSupported(string reason);
    error OnlyValidatorCanVote(address who);

    /// @notice The voting period of each proposal. __ used to avoid function naming clash.
    uint256 private __votingPeriod;
    /// @notice The gateway contract
    address public gateway;
    /// @dev Tracks the proposal id mapping to the validators who have already voted
    mapping(uint256 => EnumerableSet.AddressSet) private proposalVoters;

    constructor(address _gateway, uint256 _votingPeriod) Governor("GatewayGovernor") {
        gateway = _gateway;
        __votingPeriod = _votingPeriod;
    }

    /// @notice See {Governor-_quorumReached}.
    function _quorumReached(uint256 _proposalId) internal view override returns (bool) {
        EnumerableSet.AddressSet storage voters = proposalVoters[_proposalId];
        Membership memory membership = IGateway(gateway).getCurrentMembership();

        uint256 totalWeight = 0;
        uint256 totalVotedWeight = 0;

        uint256 totalValidators = membership.validators.length;
        for (uint256 i = 0; i < totalValidators; ) {
            uint256 validatorWeight = membership.validators[i].weight;

            totalWeight += validatorWeight;

            // add voted validator weight
            if (voters.contains(membership.validators[i].addr)) {
                totalVotedWeight += validatorWeight;
            }

            unchecked {
                i++;
            }
        }

        return totalWeight * 2 < totalVotedWeight * 3;
    }

    /// @dev To vote for a proposal, validators should just call `castVote(proposalId)`,
    /// no vote is assumed to be against the proposal. This means once a quorum is reached,
    /// the voting is successful, hence default to true.
    function _voteSucceeded(uint256) internal pure override returns (bool) {
        return true;
    }

    function _getVotes(address _account, uint256, bytes memory) internal view override returns (uint256) {
        IGateway(gateway).getValidatorPower(_account);
    }

    /// @notice See {Governor-_countVote}.
    function _countVote(uint256 _proposalId, address _account, uint8, uint256 _weight, bytes memory) internal override {
        if (_weight == 0) {
            revert OnlyValidatorCanVote(_account);
        }
        proposalVoters[_proposalId].add(_account);
    }

    function hasVoted(uint256 _proposalId, address _account) public view override returns (bool) {
        return proposalVoters[_proposalId].contains(_account);
    }

    /// @notice See {IGovernor-quorum}.
    /// @dev Minimum number of cast voted required for a proposal to be successful. Timepoint is not required
    function quorum(uint256) public view override returns (uint256) {
        Membership memory membership = IGateway(gateway).getCurrentMembership();

        uint256 totalWeight = 0;
        uint256 totalValidators = membership.validators.length;
        for (uint256 i = 0; i < totalValidators; ) {
            totalWeight += membership.validators[i].weight;
            unchecked {
                i++;
            }
        }
        return (totalWeight * 2) / 3;
    }

    // solhint-disable-next-line func-name-mixedcase
    function COUNTING_MODE() external pure override returns (string memory) {
        return "membership";
    }

    // solhint-disable-next-line func-name-mixedcase
    function CLOCK_MODE() public pure override returns (string memory) {
        return "blocknumber";
    }

    function clock() public view override returns (uint48) {
        return uint48(block.number);
    }

    /// @notice See {Governor-votingPeriod}.
    function votingPeriod() public view override returns (uint256) {
        return __votingPeriod;
    }

    /// @notice See {Governor-votingDelay}.
    /// @dev Disable voting deplay, it is ok to start voting after proposal created.
    function votingDelay() public pure override returns (uint256) {
        return 0;
    }

    // ====================== Disabled methods ================
    /// @notice There is no need for queue proposals, revert as {IGovernor-queue} specified
    function queue(address[] memory, uint256[] memory, bytes[] memory, bytes32) public pure override returns (uint256) {
        revert NotSupported("no need queue");
    }

    function castVote(uint256, uint8) public pure override returns (uint256) {
        revert NotSupported("no need cast vote with support");
    }

    function castVoteWithReason(uint256, uint8, string calldata) public pure override returns (uint256) {
        revert NotSupported("no need cast vote with support");
    }

    function castVoteWithReasonAndParams(
        uint256,
        uint8,
        string calldata,
        bytes memory
    ) public pure override returns (uint256) {
        revert NotSupported("no need cast vote with support");
    }

    function castVoteBySig(uint256, uint8, address, bytes memory) public pure override returns (uint256) {
        revert NotSupported("no need cast vote with support");
    }

    function castVoteWithReasonAndParamsBySig(
        uint256,
        uint8,
        address,
        string calldata,
        bytes memory,
        bytes memory
    ) public pure override returns (uint256) {
        revert NotSupported("no need cast vote with support");
    }
}
