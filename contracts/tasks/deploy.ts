import { task } from 'hardhat/config'
import { HardhatRuntimeEnvironment } from 'hardhat/types'

task(
    'deploy-stack',
    'Builds and deploys the IPC stack on the selected network',
    async (args, hre: HardhatRuntimeEnvironment) => {
        await hre.run('compile')

        console.log()
        console.log(
            '==== LIBRARY DEPLOYMENT ===========================================================================',
        )
        await hre.run('deploy-libraries')
        console.log()

        console.log(
            '==== GATEWAY DEPLOYMENT ===========================================================================',
        )
        await hre.run('deploy-gateway')
        console.log()

        console.log(
            '==== REGISTRY DEPLOYMENT ==========================================================================',
        )
        await hre.run('deploy-registry')
        console.log()

        console.log('████████████████████████████████████████████████████████████')
        console.log('█                                                          █')
        console.log('█                IPC STACK DEPLOYED! 🚀                    █')
        console.log('█                                                          █')
        console.log('████████████████████████████████████████████████████████████')
    },
)
