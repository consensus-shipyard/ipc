import { task, types } from 'hardhat/config'
import { HardhatRuntimeEnvironment, TaskArguments } from 'hardhat/types'
import { Deployments, Diamond } from './lib'
import { IDiamond, IDiamondCut } from '../typechain'
import _ = require('lodash')

task('upgrade', 'Upgrades a diamond contract')
    .addParam('contract', 'The contract to upgrade. One of: gateway, registry', null, types.string)
    .addOptionalParam(
        'initAddress',
        'The address of the contract to call init on (optional)',
        '0x0000000000000000000000000000000000000000',
        types.string,
    )
    .addOptionalParam('initCalldata', 'The calldata to pass to init in hex(optional)', '0x', types.string)
    .setAction(async (args: TaskArguments, hre: HardhatRuntimeEnvironment) => {
        await hre.run('compile')

        const { contract, initAddress, initCalldata } = args
        if (contract !== 'gateway' && contract !== 'registry') {
            throw new Error('Invalid contract specified')
        }

        const libraries: Deployments = await hre.run('deploy-libraries')
        logNewDeployments('library', libraries)

        // Run the corresponding deploy task passing in upgrade=true to skip the diamond deployment.
        // This will only deploy the facets that have changed, but will return the full set of facets.
        // One can know which facets were newly deployed by checking the newlyDeployed property of the result.
        const updatedFacets: Deployments = await hre.run('deploy-' + contract, {
            upgrade: true,
        })
        logNewDeployments('facet', updatedFacets)

        const diamond = await (async (diamondName) => {
            const deployments = await Deployments.resolve(hre, diamondName)
            const contract = deployments.contracts[diamondName]
            return new Diamond(hre, contract)
        })(contract === 'gateway' ? 'GatewayDiamond' : 'SubnetRegistryDiamond')

        const facetCuts = await diamond.computeFacetCuts(updatedFacets)

        if (facetCuts.length === 0) {
            console.log('No facet cuts needed')
            return
        }

        const [deployer] = await hre.getUnnamedAccounts()
        const balance = await hre.ethers.provider.getBalance(deployer)

        console.log(
            `Performing facet cuts with account: ${deployer} and balance: ${hre.ethers.utils.formatEther(balance.toString())}`,
        )

        console.log(`Facet cuts:\n\n${JSON.stringify(facetCuts, null, 2)}`)

        const diamondCut = (await hre.ethers.getContractAt('IDiamondCut', diamond.contract.address)) as IDiamondCut
        const result = await diamondCut.diamondCut(facetCuts, initAddress, initCalldata)
        const receipt = await result.wait(1)
        console.log(`Diamond cut transaction: ${receipt.transactionHash}`)
        console.log(`Included in block: ${receipt.blockNumber}`)
        console.log(`Gas used: ${receipt.gasUsed.toString()}`)
        console.log(`Status: ${receipt.status}`)
        console.log(`Events:\n\n${JSON.stringify(receipt.events, null, 2)}`)
    })

function logNewDeployments(kind: string, deployments: Deployments) {
    const newlyDeployed = _.pickBy(deployments.results, ({ newlyDeployed }) => newlyDeployed)
    if (Object.keys(newlyDeployed).length === 0) {
        console.log(`No ${kind} contracts were newly deployed`)
        return
    }
    console.log(`Newly deployed ${kind} contracts:`)
    for (const [name, { address }] of Object.entries(newlyDeployed)) {
        console.log(`  ${name} at ${address}`)
    }
}
