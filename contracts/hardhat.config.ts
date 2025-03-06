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
const networkDefinition = (chainId: number, url: string) => ({
    chainId,
    url: url,
    accounts: [process.env.PRIVATE_KEY!],
    // timeout to support also slow networks (like calibration/mainnet)
    timeout: 1000000,
    saveDeployments: true,
})

// A boolean flag for whether to disable viaIR for solidity optimization.
// With viaIR enabled, it will generally lead to a smaller and optimized contract, but sometimes
// it will cause stack too deep issues. Currently in test, viaIR actually creates stack too deep.
// Dynamically disable viaIR for test, it's not required for tests as well.
const isDisableVIAIR = process.env.VIAIR_DISABLED === 'true'

let config: HardhatUserConfig = {
    solidity: {
        compilers: [
            {
                version: '0.8.23',
                settings: {
                    viaIR: !isDisableVIAIR,
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
        contracts: ['GatewayDiamond', 'SubnetActorDiamond', 'GatewayActorModifiers', 'SubnetActorModifiers'],
        fullPath: false,
    },
}

// Only add the network configurations if we have a private key.
// Some targets don't require networks, e.g. gen-selector-library.
if (process.env.PRIVATE_KEY) {
    config = Object.assign(config, {
        defaultNetwork: 'calibrationnet',
        networks: {
            // Static networks.
            mainnet: networkDefinition(314, 'https://api.node.glif.io/rpc/v1'),
            calibrationnet: networkDefinition(314159, 'https://api.calibration.node.glif.io/rpc/v1'),
            localnet: networkDefinition(31415926, 'http://localhost:8545'),
            // Auto uses RPC_URL provided by the user, and an optional CHAIN_ID.
            // If provided, Hardhat will assert that the chain ID matches the one returned by the RPC.
            auto: networkDefinition(parseInt(process.env.CHAIN_ID, 10), process.env.RPC_URL!),
        },
    })
}

export default config
