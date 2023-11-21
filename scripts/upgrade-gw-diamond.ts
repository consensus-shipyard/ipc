import hre, { ethers } from 'hardhat'
import {
    getFacets,
    getBytecodeFromFacet,
    getOnChainBytecodeFromFacets,
    upgradeFacet,
} from './util'

const lazyImport = async (module: any) => {
    return await import(module)
}


// Function to upgrade the Subnet Actor Diamond
async function upgradeGatewayActorDiamond(deployments) {
    // Get the Gateway Diamond address from the deployments
    const gatewayDiamondAddress = deployments.Gateway

    // Get the facets of the Gateway Diamond
    const facets = await getFacets(gatewayDiamondAddress)
    const provider = ethers.provider

    //return this object to update the caller on what facets where updated
    const updatedFacets = {}

    const onChainFacetBytecodes = await getOnChainBytecodeFromFacets(facets)

    // Loop through each facet in the deployments
    for (let facetIndex in deployments.Facets) {
        const facet = deployments.Facets[facetIndex]
        const facetBytecode = await getBytecodeFromFacet(facet)
       // compare the bytecode of the facets deployed with the local version of the contract.
        if (!onChainFacetBytecodes.hasOwnProperty(facetBytecode)) {
            let formattedLibs = Object.entries(facet.libs)
                .map(([key, value]) => `  - ${key}: ${value}`)
                .join('\n')

            console.info(`
Facet Bytecode Not Found:
---------------------------------
Facet Name: ${facet.name}
Libraries:
${formattedLibs}
Address: ${facet.address}
`)

            const newFacet = await upgradeFacet(
                gatewayDiamondAddress,
                facet.name,
                facet.libs,
            )
            for (let key in newFacet) updatedFacets[key] = newFacet[key]

            console.info(`
Deployment Status:
-------------------------
New replacement facet (${facet.name}) deployed.
`)
        }
    }
    return updatedFacets
}
exports.upgradeDiamond = upgradeGatewayActorDiamond
