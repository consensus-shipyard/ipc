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