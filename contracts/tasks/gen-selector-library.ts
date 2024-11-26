import { task } from 'hardhat/config'
import '@nomiclabs/hardhat-ethers'
import * as fs from 'fs'
import { HardhatRuntimeEnvironment } from 'hardhat/types'
import { selectors } from './lib'

task('gen-selector-library', 'Generates a Solidity library with contract selectors for tests').setAction(
    async (_args, hre: HardhatRuntimeEnvironment) => {
        // ridiculously, this appears to be the only way to get hardhat to compile a specific subtree
        // we are only updating the in-memory representation of the config, so it won't write this value out to disk
        // be careful if you compose this task with other tasks in larger scripts!
        console.log("compiling mocks...")
        const oldSources = hre.config.paths.sources
        hre.config.paths.sources = './test/mocks'
        await hre.run('compile')

        hre.config.paths.sources = oldSources
        console.log("compiling contracts...")
        await hre.run('compile')

        const contracts: string[] = [
            'OwnershipFacet',
            'DiamondCutFacet',
            'DiamondLoupeFacet',
            'GatewayGetterFacet',
            'GatewayManagerFacet',
            'GatewayMessengerFacet',
            'CheckpointingFacet',
            'TopDownFinalityFacet',
            'XnetMessagingFacet',
            'SubnetActorGetterFacet',
            'SubnetActorManagerFacet',
            'SubnetActorPauseFacet',
            'SubnetActorRewardFacet',
            'SubnetActorCheckpointingFacet',
            'RegisterSubnetFacet',
            'SubnetGetterFacet',
            'SubnetActorMock',
            'ValidatorRewardFacet',
        ]

        const resolveSelectors = async (contractName: string) => {
            console.log(`Resolving selectors for ${contractName}...`)
            const artifact = await hre.artifacts.readArtifact(contractName)
            const iface = new hre.ethers.utils.Interface(artifact.abi)
            const encodedSelectors = hre.ethers.utils.defaultAbiCoder
                .encode(['bytes4[]'], [selectors({ interface: iface })])
                .slice(2)
            return [contractName, encodedSelectors]
        }

        const allSelectors = Object.fromEntries(await Promise.all(contracts.map(resolveSelectors)))

        // Codegen.
        let code = `// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity ^0.8.19;

library SelectorLibrary {
    function resolveSelectors(string memory facetName) public pure returns (bytes4[] memory facetSelectors) {`

        for (const [contractName, selector] of Object.entries(allSelectors)) {
            code += `
        if (keccak256(abi.encodePacked(facetName)) == keccak256(abi.encodePacked("${contractName}"))) {
            return abi.decode(hex"${selector}", (bytes4[]));
        }`
        }

        code += `
        revert(string.concat("Selectors not found for facet: ", facetName));
    }
}
`
        // Write the generated code to a file.
        const outputPath = 'test/helpers/SelectorLibrary.sol'
        fs.writeFileSync(outputPath, code)
        console.log(`Selector library written to ${outputPath}`)
    },
)
