import { task } from 'hardhat/config'
import { HardhatRuntimeEnvironment } from 'hardhat/types'
import { Deployments } from './lib'

task(
    'deploy-libraries',
    'Build and deploys all libraries on the selected network',
    async (_args, hre: HardhatRuntimeEnvironment): Promise<Deployments> => {
        await hre.run('compile')

        const [deployer] = await hre.getUnnamedAccounts()

        const contracts = [
            { name: 'AccountHelper' },
            { name: 'SubnetIDHelper' },
            { name: 'LibStaking' },
            { name: 'LibQuorum' },
            { name: 'CrossMsgHelper', libraries: ['SubnetIDHelper'] },
        ]

        return await Deployments.deploy(hre, deployer, ...contracts)
    },
)
