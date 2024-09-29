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
const networkDefinition = (chainId: number, url: string, accounts: string[]) => ({
    chainId,
    url: url,
    accounts,
    // timeout to support also slow networks (like calibration/mainnet)
    timeout: 1000000,
    saveDeployments: true,
})

let config: HardhatUserConfig = {
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
            mainnet: networkDefinition(314, 'https://api.node.glif.io/rpc/v1', [process.env.PRIVATE_KEY!]),
            calibrationnet: networkDefinition(314159, 'https://api.calibration.node.glif.io/rpc/v1', [process.env.PRIVATE_KEY!]),
            localnet: networkDefinition(31337, 'http://localhost:8545', [
                '0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80',
                '0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d',
                '0x5de4111afa1a4b94908f83103eb1f1706367c2e68ca870fc3fb9a804cdab365a',
                '0x7c852118294e51e653712a81e05800f419141751be58f605c371e15141b007a6',
                '0x47e179ec197488593b187f80a00eb0da91f1b9d0b13f8733639f19c30a34926a',
                '0x8b3a350cf5c34c9194ca85829a2df0ec3153be0318b5e2d3348e872092edffba',
                '0x92db14e403b83dfe3df233f83dfa3a0d7096f21ca9b0d6d6b8d88b2b4ec1564e',
                '0x4bbbf85ce3377467afe5d46f804f221813b2bb87f24d81f60f1fcdbf7cbf4356',
                '0xdbda1821b80551c9d65939329250298aa3472ba22feea921c0cf5d620ea67b97',
                '0x2a871d0798f97d79848a013d4936a73bf4cc922c825d33c1cf7073dff6d409c6',
            ]),
            // Auto uses RPC_URL provided by the user, and an optional CHAIN_ID.
            // If provided, Hardhat will assert that the chain ID matches the one returned by the RPC.
            auto: networkDefinition(parseInt(process.env.CHAIN_ID, 10), process.env.RPC_URL!, [process.env.PRIVATE_KEY!]),
        },
    })
}

export default config
