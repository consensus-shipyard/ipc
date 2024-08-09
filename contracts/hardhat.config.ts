import '@nomicfoundation/hardhat-foundry'
import '@nomiclabs/hardhat-ethers'
import '@typechain/hardhat'
import 'hardhat-contract-sizer'
import 'hardhat-storage-layout-changes'
import { HardhatUserConfig } from 'hardhat/config'

// Hardhat deploy stuff.
import 'hardhat-deploy'
import 'hardhat-deploy-ethers'

// Import our extensions.
import './extensions'

// Load environment variables from .env file.
import { config as dotenvConfig } from 'dotenv'
dotenvConfig({ path: './.env' })

// Import our tasks.
import './tasks'

// Define network configurations.
const networkDefinition = (chainId: number) => ({
    chainId,
    url: process.env.RPC_URL!,
    accounts: [process.env.PRIVATE_KEY!],
    // timeout to support also slow networks (like calibration/mainnet)
    timeout: 1000000,
    saveDeployments: true,
})

const config: HardhatUserConfig = {
    defaultNetwork: 'calibrationnet',
    networks: {
        mainnet: networkDefinition(314),
        calibrationnet: networkDefinition(314159),
        localnet: networkDefinition(31415926),
        // automatically fetch chainID for network
        auto: networkDefinition(parseInt(process.env.CHAIN_ID!, 16)),
    },
    solidity: {
        compilers: [
            {
                version: '0.8.23',
                settings: {
                    viaIR: true,
                    optimizer: {
                        enabled: true,
                        runs: 200,
                    },
                },
            },
        ],
    },
    typechain: {
        outDir: 'typechain',
        target: 'ethers-v5',
    },
    paths: {
        storageLayouts: '.storage-layouts',
    },
    storageLayoutChanges: {
        contracts: [
            'GatewayDiamond',
            'SubnetActorDiamond',
            'GatewayActorModifiers',
            'SubnetActorModifiers',
        ],
        fullPath: false,
    },
}

export default config
