import { task, types } from 'hardhat/config'
import { HardhatRuntimeEnvironment, TaskArguments } from 'hardhat/types'
import { Deployments } from './lib'

// Customize your gateway parameters here.
const gatewayConstructorParams = {
    bottomUpCheckPeriod: 10,
    activeValidatorsLimit: 100,
    majorityPercentage: 66,
    networkName: {
        root: undefined, // Will be set later.
        route: [],
    },
    genesisValidators: [],
    commitSha: undefined, // Will be set later.
}

task('deploy-gateway')
    .setDescription('Builds and deploys the gateway contract')
    .addOptionalPositionalParam(
        'upgrade',
        'Must be set to true if upgrading to skip the diamond deployment',
        false,
        types.boolean,
    )
    .setAction(async (args: TaskArguments, hre: HardhatRuntimeEnvironment): Promise<Deployments> => {
        await hre.run('compile')

        const [deployer] = await hre.getUnnamedAccounts()

        // Deploy the facets.
        const facets = await deployFacets(hre, deployer)

        if (args.upgrade) {
            console.log('Running as part of an upgrade; skipping deployment of GatewayDiamond')
            return facets
        }

        // Deploy the diamond.
        const diamond = await deployGatewayDiamond(hre, deployer, facets)
        return facets.join(diamond)
    })

async function deployFacets(hre: HardhatRuntimeEnvironment, deployer: string): Promise<Deployments> {
    const facets = [
        {
            name: 'GatewayGetterFacet',
            libraries: ['SubnetIDHelper', 'LibQuorum'],
        },
        { name: 'DiamondLoupeFacet' },
        { name: 'DiamondCutFacet' },
        {
            name: 'GatewayManagerFacet',
            libraries: ['CrossMsgHelper', 'SubnetIDHelper'],
        },
        {
            name: 'GatewayMessengerFacet',
            libraries: ['CrossMsgHelper', 'SubnetIDHelper'],
        },
        {
            name: 'CheckpointingFacet',
            libraries: ['AccountHelper', 'SubnetIDHelper', 'CrossMsgHelper'],
        },
        {
            name: 'XnetMessagingFacet',
            libraries: ['AccountHelper', 'CrossMsgHelper', 'SubnetIDHelper'],
        },
        { name: 'TopDownFinalityFacet', libraries: ['AccountHelper'] },
        { name: 'OwnershipFacet' },
    ]

    return await Deployments.deploy(hre, deployer, ...facets)
}

async function deployGatewayDiamond(
    hre: HardhatRuntimeEnvironment,
    deployer: string,
    facets: Deployments,
): Promise<Deployments> {
    gatewayConstructorParams.networkName.root = await hre.getChainId()
    gatewayConstructorParams.commitSha = hre.ethers.utils.formatBytes32String(gitCommitSha())

    const deployments = await Deployments.deploy(hre, deployer, {
        name: 'GatewayDiamond',
        args: [facets.asFacetCuts(), gatewayConstructorParams],
    })
    return deployments
}

function gitCommitSha(): string {
    return require('child_process').execSync('git rev-parse --short HEAD').toString().trim()
}
