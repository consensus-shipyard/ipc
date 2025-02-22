import { task } from 'hardhat/config'
import { HardhatRuntimeEnvironment, TaskArguments } from 'hardhat/types'
import { Deployments } from './lib'

// step 1. deploy the cross network messenger util contract
// sample command: pnpm exec hardhat cross-network-messenger-deploy --network calibrationnet <GATEWAY ADDRESS>
task('cross-network-messenger-deploy')
    .addPositionalParam('gatewayAddr', 'the address of the gateway contract')
    .setDescription('Deploy example cross network messenger util contract')
    .setAction(async (args: TaskArguments, hre: HardhatRuntimeEnvironment) => {
        await hre.run('compile')

        const [deployer] = await hre.getUnnamedAccounts()
        const balance = await hre.ethers.provider.getBalance(deployer)
        console.log(
            `Deploying cross network messenger contract with account: ${deployer} and balance: ${hre.ethers.utils.formatEther(balance.toString())}`,
        )

        await Deployments.deploy(hre, deployer, {
            name: 'CrossMessengerCaller',
            args: [args.gatewayAddr],
            libraries: ['SubnetIDHelper'],
        })
    })

// step 2. invoke a cross network send message
// sample command: pnpm exec hardhat cross-network-send --network calibrationnet 314159 <YOUR SUBNET ETH ROUTE> <RECIPIENT> <VALUE>
task('cross-network-send')
    .addPositionalParam('root', 'the chain id of root subnet')
    .addPositionalParam('route', 'the addresses of the subnet routes, use "," to separate the addresses')
    .addPositionalParam('recipient', 'the recipient of the send')
    .addPositionalParam('value', 'the value to send over')
    .setDescription('Invoke a cross network send in the target subnet')
    .setAction(async (args: TaskArguments, hre: HardhatRuntimeEnvironment) => {
        await hre.run('compile')

        const subnetId = { root: args.root, route: args.route.split(',') }
        console.log('sending to subnet', subnetId)

        const amount = hre.ethers.utils.parseEther(args.value)
        console.log('sending to address', args.recipient, 'with amount', amount)

        const contracts = await Deployments.resolve(hre, 'CrossMessengerCaller')
        const contract = contracts.contracts.CrossMessengerCaller
        await contract.invokeSendMessage(subnetId, args.recipient, amount, { value: Number(amount) })
    })
