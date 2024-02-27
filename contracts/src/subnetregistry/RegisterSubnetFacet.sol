// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity ^0.8.23;

import {IDiamond} from "../interfaces/IDiamond.sol";
import {SubnetActorDiamond} from "../SubnetActorDiamond.sol";
import {SubnetRegistryActorStorage} from "../lib/LibSubnetRegistryStorage.sol";

import {ReentrancyGuard} from "../lib/LibReentrancyGuard.sol";
import {WrongGateway} from "../errors/IPCErrors.sol";

contract RegisterSubnetFacet is ReentrancyGuard {
    SubnetRegistryActorStorage internal s;

    /// @notice Event emitted when a new subnet is deployed.
    event SubnetDeployed(address subnetAddr);

    /// @notice Deploys a new subnet actor.
    /// @param _params The constructor params for Subnet Actor Diamond.
    function newSubnetActor(
        SubnetActorDiamond.ConstructorParams calldata _params
    ) external nonReentrant returns (address subnetAddr) {
        if (_params.ipcGatewayAddr != s.GATEWAY) {
            revert WrongGateway();
        }

        IDiamond.FacetCut[] memory diamondCut = new IDiamond.FacetCut[](5);

        // set the diamond cut for subnet getter
        diamondCut[0] = IDiamond.FacetCut({
            facetAddress: s.SUBNET_ACTOR_GETTER_FACET,
            action: IDiamond.FacetCutAction.Add,
            functionSelectors: s.subnetActorGetterSelectors
        });

        // set the diamond cut for subnet manager
        diamondCut[1] = IDiamond.FacetCut({
            facetAddress: s.SUBNET_ACTOR_MANAGER_FACET,
            action: IDiamond.FacetCutAction.Add,
            functionSelectors: s.subnetActorManagerSelectors
        });

        diamondCut[2] = IDiamond.FacetCut({
            facetAddress: s.SUBNET_ACTOR_REWARD_FACET,
            action: IDiamond.FacetCutAction.Add,
            functionSelectors: s.subnetActorRewarderSelectors
        });

        diamondCut[3] = IDiamond.FacetCut({
            facetAddress: s.SUBNET_ACTOR_CHECKPOINTING_FACET,
            action: IDiamond.FacetCutAction.Add,
            functionSelectors: s.subnetActorCheckpointerSelectors
        });

        diamondCut[4] = IDiamond.FacetCut({
            facetAddress: s.SUBNET_ACTOR_PAUSE_FACET,
            action: IDiamond.FacetCutAction.Add,
            functionSelectors: s.subnetActorPauserSelectors
        });

        // slither-disable-next-line reentrancy-benign
        subnetAddr = address(new SubnetActorDiamond(diamondCut, _params));

        //nonces start with 1, similar to eip 161
        ++s.userNonces[msg.sender];
        s.subnets[msg.sender][s.userNonces[msg.sender]] = subnetAddr;

        emit SubnetDeployed(subnetAddr);

        return subnetAddr;
    }
}
