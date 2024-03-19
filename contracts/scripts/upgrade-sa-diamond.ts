import { ethers } from 'hardhat'
import {
    getFacets,
    getBytecodeFromFacet,
    getOnChainBytecodeFromFacets,
    upgradeFacetOnChain,
    upgradeFacet,
    logMissingFacetInfo,
} from './util'

/**
 * Upgrade the Subnet Actor Diamond.
 * @param deployments - The deployment data.
 * @returns An object of updated facets.
 */
async function upgradeSubnetActorDiamond(deployments) {
    const subnetActorDiamondAddress = deployments.SubnetActorDiamond
    console.log("subnetActorDiamondAddress", subnetActorDiamondAddress)

    const onChainFacets = await getFacets(subnetActorDiamondAddress)
    console.log('onChainFacets',onChainFacets)
    
    const updatedFacets = {}
    const onChainFacetBytecodes = await getOnChainBytecodeFromFacets(
        onChainFacets,
    )
    console.log("onChainFacetBytecodes", onChainFacetBytecodes)

    for (const facet of deployments.Facets) {
        console.log("Facet", facet);
        await upgradeFacet(
            facet,
            onChainFacets,
            subnetActorDiamondAddress,
            updatedFacets,
            onChainFacetBytecodes,
            deployments,
        )
    }

    return updatedFacets
}

export { upgradeSubnetActorDiamond as upgradeDiamond }
