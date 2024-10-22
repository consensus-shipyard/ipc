import { task } from 'hardhat/config'
import { HardhatRuntimeEnvironment, TaskArguments } from 'hardhat/types'
import { Deployments } from './lib'

// step 1. deploy the validator rewarder
// sample command: pnpm exec hardhat validator-rewarder-deploy --network calibrationnet
task('validator-rewarder-deploy')
    .setDescription('Deploy example subnet validator rewarder contract')
    .setAction(async (_: TaskArguments, hre: HardhatRuntimeEnvironment) => {
        await hre.run('compile')

        const [deployer] = await hre.getUnnamedAccounts()
        const balance = await hre.ethers.provider.getBalance(deployer)
        console.log(
            `Deploying validator rewarder contract with account: ${deployer} and balance: ${hre.ethers.utils.formatEther(balance.toString())}`,
        )

        await Deployments.deploy(hre, deployer, {
            name: 'ValidatorRewarderMap',
            libraries: [],
        })
    })

// step 2. set the subnet for the rewarder
// sample command: pnpm exec hardhat validator-rewarder-set-subnet --network calibrationnet 314159 <YOUR SUBNET ETH ROUTE ADDRESS>
task('validator-rewarder-set-subnet')
    .setDescription('Deploy example subnet validator rewarder contract')
    .addPositionalParam('root', 'the chain id of parent subnet')
    .addPositionalParam('address', 'the address of the subnet actor contract, L2 only')
    .setAction(async (args: TaskArguments, hre: HardhatRuntimeEnvironment) => {
        await hre.run('compile')

        const [deployer] = await hre.getUnnamedAccounts()
        const balance = await hre.ethers.provider.getBalance(deployer)
        console.log(
            `Set validator rewarder subnet with account: ${deployer} and balance: ${hre.ethers.utils.formatEther(balance.toString())}`,
        )

        // only L2 for now
        const subnetId = { root: args.root, route: [args.address] }
        console.log('pointing to', subnetId)

        const contracts = await Deployments.resolve(hre, 'ValidatorRewarderMap')
        const contract = contracts.contracts.ValidatorRewarderMap
        await contract.setSubnet(subnetId)
    })