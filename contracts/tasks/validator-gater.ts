import { task } from 'hardhat/config'
import { HardhatRuntimeEnvironment, TaskArguments } from 'hardhat/types'
import { Deployments } from './lib'
import { artifacts } from 'hardhat'

// step 1. deploy the validator gater
// sample command: pnpm exec hardhat validator-gater-deploy --network calibrationnet
task('validator-gater-deploy')
    .setDescription('Deploy example subnet validator gater contract')
    .setAction(async (_: TaskArguments, hre: HardhatRuntimeEnvironment) => {
        await hre.run('compile')

        const [deployer] = await hre.getUnnamedAccounts()
        const balance = await hre.ethers.provider.getBalance(deployer)
        console.log(
            `Deploying validator gater contract with account: ${deployer} and balance: ${hre.ethers.utils.formatEther(balance.toString())}`,
        )

        await Deployments.deploy(hre, deployer, {
            name: 'SubnetValidatorGater',
            libraries: ['SubnetIDHelper'],
        })
    })

// step 2. create the subnet with validator gater
// if you try to set the federated power or join with collateral, the txn should now revert

// step 3. update the subnet id of the validator gater
// sample command: pnpm exec hardhat validator-gater-set-subnet --network calibrationnet 314159 <YOUR SUBNET ETH ROUTE ADDRESS>
task('validator-gater-set-subnet')
    .addPositionalParam('root', 'the chain id of parent subnet')
    .addPositionalParam('address', 'the address of the subnet actor contract, L2 only')
    .setDescription('Update the target subnet id to gate')
    .setAction(async (args: TaskArguments, hre: HardhatRuntimeEnvironment) => {
        await hre.run('compile')

        // only L2 for now
        const subnetId = { root: args.root, route: [args.address] }
        console.log('pointing to', subnetId)

        const contracts = await Deployments.resolve(hre, 'SubnetValidatorGater')
        const contract = contracts.contracts.SubnetValidatorGater
        await contract.setSubnet(subnetId)
    })

// step 4. approve the validator accordingly
// sample command: pnpm exec hardhat validator-gater-set-subnet --network calibrationnet 314159 <YOUR SUBNET ETH ROUTE ADDRESS>
task('validator-gater-approve-validator')
    .addPositionalParam('validator', 'the address of the validator')
    .addPositionalParam('min', 'minimal power the validator must have, in wei')
    .addPositionalParam('max', 'maximal power the validator can have, in wei')
    .setDescription('Approve the power of validator accordingly')
    .setAction(async (args: TaskArguments, hre: HardhatRuntimeEnvironment) => {
        await hre.run('compile')

        const contracts = await Deployments.resolve(hre, 'SubnetValidatorGater')
        const contract = contracts.contracts.SubnetValidatorGater
        await contract.approve(args.validator, args.min, args.max)
    })

task('validator-gater-info')
    .setDescription('Get the info of the gater')
    .setAction(async (_: TaskArguments, hre: HardhatRuntimeEnvironment) => {
        await hre.run('compile')

        const contracts = await Deployments.resolve(hre, 'SubnetValidatorGater')
        const contract = contracts.contracts.SubnetValidatorGater

        const subnet = await contract.subnet()
        console.log({
            owner: await contract.owner(),
            subnet,
        })
    })
