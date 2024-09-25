import { task, types } from 'hardhat/config'
import { HardhatRuntimeEnvironment, TaskArguments } from 'hardhat/types'
import { Deployments, selectors } from './lib'

export enum SubnetCreationPrivileges {
    Unrestricted = 0,
    Owner = 1,
}

task('deploy-registry')
    .setDescription('Builds and deploys the registry contract')
    .addOptionalPositionalParam(
        'upgrade',
        'Must be set to true if upgrading to skip the diamond deployment',
        false,
        types.boolean,
    )
    .setAction(async (args: TaskArguments, hre: HardhatRuntimeEnvironment) => {
        await hre.run('compile')

        const [deployer] = await hre.getUnnamedAccounts()
        const balance = await hre.ethers.provider.getBalance(deployer)
        console.log(
            `Deploying registry contracts with account: ${deployer} and balance: ${hre.ethers.utils.formatEther(balance.toString())}`,
        )

        const subnetActorFacets = await Deployments.deploy(
            hre,
            deployer,
            {
                name: 'SubnetActorGetterFacet',
                libraries: ['SubnetIDHelper'],
            },
            {
                name: 'SubnetActorManagerFacet',
                libraries: ['SubnetIDHelper'],
            },
            { name: 'SubnetActorPauseFacet' },
            { name: 'SubnetActorRewardFacet' },
            { name: 'SubnetActorCheckpointingFacet' },
            { name: 'DiamondCutFacet' },
            { name: 'DiamondLoupeFacet' },
            { name: 'OwnershipFacet' },
        )

        const registryFacets = await Deployments.deploy(
            hre,
            deployer,
            {
                name: 'RegisterSubnetFacet',
                libraries: ['SubnetIDHelper'],
            },
            { name: 'SubnetGetterFacet' },
            { name: 'DiamondLoupeFacet' },
            { name: 'DiamondCutFacet' },
            { name: 'OwnershipFacet' },
        )

        if (args.upgrade) {
            console.log('Running as part of an upgrade; skipping deployment of SubnetRegistryDiamond')
            return
        }

        // TODO(raulk): document changed default to owner from unrestricted
        const mode =
            process.env.REGISTRY_CREATION_PRIVILEGES === 'unrestricted'
                ? SubnetCreationPrivileges.Unrestricted
                : SubnetCreationPrivileges.Owner

        console.log(`
    ***************************************************************
    **                                                           **
    **  Subnet creation privileges: ${mode}                            **
    **                                                           **
    ***************************************************************
  `)

        const registryConstructorParams = {
            gateway: (await hre.deployments.get('GatewayDiamond')).address,
            getterFacet: subnetActorFacets.addresses['SubnetActorGetterFacet'],
            managerFacet: subnetActorFacets.addresses['SubnetActorManagerFacet'],
            rewarderFacet: subnetActorFacets.addresses['SubnetActorRewardFacet'],
            checkpointerFacet: subnetActorFacets.addresses['SubnetActorCheckpointingFacet'],
            pauserFacet: subnetActorFacets.addresses['SubnetActorPauseFacet'],
            diamondCutFacet: subnetActorFacets.addresses['DiamondCutFacet'],
            diamondLoupeFacet: subnetActorFacets.addresses['DiamondLoupeFacet'],
            ownershipFacet: subnetActorFacets.addresses['OwnershipFacet'],

            subnetActorGetterSelectors: selectors(subnetActorFacets.contracts['SubnetActorGetterFacet']),
            subnetActorManagerSelectors: selectors(subnetActorFacets.contracts['SubnetActorManagerFacet']),
            subnetActorRewarderSelectors: selectors(subnetActorFacets.contracts['SubnetActorRewardFacet']),
            subnetActorCheckpointerSelectors: selectors(subnetActorFacets.contracts['SubnetActorCheckpointingFacet']),
            subnetActorPauserSelectors: selectors(subnetActorFacets.contracts['SubnetActorPauseFacet']),
            subnetActorDiamondCutSelectors: selectors(subnetActorFacets.contracts['DiamondCutFacet']),
            subnetActorDiamondLoupeSelectors: selectors(subnetActorFacets.contracts['DiamondLoupeFacet']),
            subnetActorOwnershipSelectors: selectors(subnetActorFacets.contracts['OwnershipFacet']),
            creationPrivileges: Number(mode),
        }

        console.log(`Deploying SubnetRegistryDiamond...`)
        const registry = await hre.deployments.deploy('SubnetRegistryDiamond', {
            from: deployer,
            args: [registryFacets.asFacetCuts(), registryConstructorParams],
            log: true,
            waitConfirmations: 1,
        })
        console.log(`SubnetRegistryDiamond deployed at ${registry.address}`)
    })
