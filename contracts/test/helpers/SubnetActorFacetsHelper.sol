// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity ^0.8.23;

import {SubnetActorManagerFacet} from "../../contracts/subnet/SubnetActorManagerFacet.sol";
import {SubnetActorPauseFacet} from "../../contracts/subnet/SubnetActorPauseFacet.sol";
import {SubnetActorCheckpointingFacet} from "../../contracts/subnet/SubnetActorCheckpointingFacet.sol";
import {SubnetActorRewardFacet} from "../../contracts/subnet/SubnetActorRewardFacet.sol";
import {SubnetActorGetterFacet} from "../../contracts/subnet/SubnetActorGetterFacet.sol";
import {SubnetActorDiamond} from "../../contracts/SubnetActorDiamond.sol";
import {DiamondLoupeFacet} from "../../contracts/diamond/DiamondLoupeFacet.sol";
import {DiamondCutFacet} from "../../contracts/diamond/DiamondCutFacet.sol";

library SubnetActorFacetsHelper {
    function manager(address sa) internal pure returns (SubnetActorManagerFacet) {
        SubnetActorManagerFacet facet = SubnetActorManagerFacet(sa);
        return facet;
    }

    function pauser(address sa) internal pure returns (SubnetActorPauseFacet) {
        SubnetActorPauseFacet facet = SubnetActorPauseFacet(sa);
        return facet;
    }

    function checkpointer(address sa) internal pure returns (SubnetActorCheckpointingFacet) {
        SubnetActorCheckpointingFacet facet = SubnetActorCheckpointingFacet(sa);
        return facet;
    }

    function rewarder(address sa) internal pure returns (SubnetActorRewardFacet) {
        SubnetActorRewardFacet facet = SubnetActorRewardFacet(sa);
        return facet;
    }

    function getter(address sa) internal pure returns (SubnetActorGetterFacet) {
        SubnetActorGetterFacet facet = SubnetActorGetterFacet(sa);
        return facet;
    }

    function diamondLouper(SubnetActorDiamond a) internal pure returns (DiamondLoupeFacet) {
        DiamondLoupeFacet facet = DiamondLoupeFacet(address(a));
        return facet;
    }

    function diamondCutter(SubnetActorDiamond a) internal pure returns (DiamondCutFacet) {
        DiamondCutFacet facet = DiamondCutFacet(address(a));
        return facet;
    }

    //

    function manager(SubnetActorDiamond sa) internal pure returns (SubnetActorManagerFacet) {
        SubnetActorManagerFacet facet = SubnetActorManagerFacet(address(sa));
        return facet;
    }

    function pauser(SubnetActorDiamond sa) internal pure returns (SubnetActorPauseFacet) {
        SubnetActorPauseFacet facet = SubnetActorPauseFacet(address(sa));
        return facet;
    }

    function checkpointer(SubnetActorDiamond sa) internal pure returns (SubnetActorCheckpointingFacet) {
        SubnetActorCheckpointingFacet facet = SubnetActorCheckpointingFacet(address(sa));
        return facet;
    }

    function rewarder(SubnetActorDiamond sa) internal pure returns (SubnetActorRewardFacet) {
        SubnetActorRewardFacet facet = SubnetActorRewardFacet(address(sa));
        return facet;
    }

    function getter(SubnetActorDiamond sa) internal pure returns (SubnetActorGetterFacet) {
        SubnetActorGetterFacet facet = SubnetActorGetterFacet(address(sa));
        return facet;
    }
}
