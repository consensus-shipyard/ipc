/**
 * deploy-bridge-lock
 *
 * Hardhat task to deploy BridgeLock.sol on Filecoin Calibration (or any EVM network).
 *
 * Usage:
 *   npx hardhat deploy-bridge-lock \
 *     --network calibration \
 *     --gateway  <IPC_GATEWAY_ADDRESS> \
 *     --dest-root 11155111 \
 *     --dest-receiver <BRIDGE_MINT_ADDRESS_ON_SEPOLIA> \
 *     --ipc-fee 10000000000000000
 *
 * Environment variables (via .env):
 *   PRIVATE_KEY        — deployer private key
 *
 * Outputs: deployments/bridge-lock-<network>.json
 */

import { task, types } from 'hardhat/config'
import { HardhatRuntimeEnvironment, TaskArguments } from 'hardhat/types'
import * as fs from 'fs'
import * as path from 'path'

task('deploy-bridge-lock', 'Deploy the BridgeLock contract (UUPS proxy) on Filecoin Calibration')
    .addParam('gateway', 'Address of the IPC Gateway on this chain', undefined, types.string)
    .addParam('destRoot', 'Chain ID (root) of the destination subnet', undefined, types.int)
    .addParam('destReceiver', 'Address of BridgeMint contract on the destination chain', undefined, types.string)
    .addOptionalParam('ipcFee', 'IPC fee in wei forwarded with each cross-message', '10000000000000000', types.string)
    .addOptionalParam('admin', 'Admin address (defaults to deployer)', '', types.string)
    .setAction(async (args: TaskArguments, hre: HardhatRuntimeEnvironment) => {
        await hre.run('compile')

        const [deployer] = await hre.ethers.getSigners()
        const adminAddr = args.admin || deployer.address

        console.log(`\n=== BridgeLock Deployment ===`)
        console.log(`Network:       ${hre.network.name}`)
        console.log(`Deployer:      ${deployer.address}`)
        console.log(`Admin:         ${adminAddr}`)
        console.log(`Gateway:       ${args.gateway}`)
        console.log(`Dest root:     ${args.destRoot}`)
        console.log(`Dest receiver: ${args.destReceiver}`)
        console.log(`IPC fee:       ${args.ipcFee} wei`)

        // 1. Deploy implementation
        const BridgeLock = await hre.ethers.getContractFactory('BridgeLock')
        const impl = await BridgeLock.deploy(args.gateway)
        await impl.waitForDeployment()
        const implAddr = await impl.getAddress()
        console.log(`\nImplementation deployed: ${implAddr}`)

        // 2. Encode initializer
        const destSubnet = {
            root: args.destRoot,
            route: [] as string[],
        }
        const initData = BridgeLock.interface.encodeFunctionData('initialize', [
            adminAddr,
            destSubnet,
            args.destReceiver,
            BigInt(args.ipcFee),
        ])

        // 3. Deploy ERC1967 proxy
        const ERC1967Proxy = await hre.ethers.getContractFactory(
            '@openzeppelin/contracts/proxy/ERC1967/ERC1967Proxy.sol:ERC1967Proxy',
        )
        const proxy = await ERC1967Proxy.deploy(implAddr, initData)
        await proxy.waitForDeployment()
        const proxyAddr = await proxy.getAddress()
        console.log(`Proxy deployed:          ${proxyAddr}`)

        // 4. Verify roles and state
        const bridge = BridgeLock.attach(proxyAddr)
        const adminRole = await bridge.DEFAULT_ADMIN_ROLE()
        const hasAdmin = await bridge.hasRole(adminRole, adminAddr)
        const ipcFeeOnChain = await bridge.ipcFee()
        console.log(`\nVerification:`)
        console.log(`  Admin has DEFAULT_ADMIN_ROLE: ${hasAdmin}`)
        console.log(`  ipcFee on-chain:              ${ipcFeeOnChain}`)
        console.log(`  destReceiver on-chain:        ${await bridge.destReceiver()}`)

        // 5. Measure gas
        const implDeployReceipt = await hre.ethers.provider.getTransactionReceipt(impl.deploymentTransaction()!.hash)
        const proxyDeployReceipt = await hre.ethers.provider.getTransactionReceipt(proxy.deploymentTransaction()!.hash)
        console.log(`\nGas used:`)
        console.log(`  Implementation deploy: ${implDeployReceipt?.gasUsed?.toString() ?? 'n/a'}`)
        console.log(`  Proxy deploy:          ${proxyDeployReceipt?.gasUsed?.toString() ?? 'n/a'}`)

        // Estimate lock() gas (requires a mock token — skip on mainnet/calibration if no token available)
        try {
            const ERC20Mock = await hre.ethers.getContractFactory('ERC20Mock')
            const mockToken = await ERC20Mock.deploy('TestToken', 'TT')
            await mockToken.waitForDeployment()
            await mockToken.mint(deployer.address, hre.ethers.parseEther('1000'))
            await mockToken.approve(proxyAddr, hre.ethers.parseEther('100'))
            const lockGas = await bridge.lock.estimateGas(
                await mockToken.getAddress(),
                hre.ethers.parseEther('100'),
                deployer.address,
                { value: BigInt(args.ipcFee) },
            )
            console.log(`  lock() estimate:       ${lockGas.toString()} gas`)
        } catch {
            console.log(`  lock() estimate:       skipped (gateway not live or no test token)`)
        }

        // 6. Persist deployment record
        const deploymentDir = path.join(__dirname, '..', 'deployments')
        if (!fs.existsSync(deploymentDir)) fs.mkdirSync(deploymentDir, { recursive: true })
        const record = {
            network: hre.network.name,
            chainId: (await hre.ethers.provider.getNetwork()).chainId.toString(),
            deployer: deployer.address,
            admin: adminAddr,
            implementation: implAddr,
            proxy: proxyAddr,
            gateway: args.gateway,
            destRoot: args.destRoot,
            destReceiver: args.destReceiver,
            ipcFee: args.ipcFee,
            deployedAt: new Date().toISOString(),
            implDeployBlock: implDeployReceipt?.blockNumber ?? null,
            proxyDeployBlock: proxyDeployReceipt?.blockNumber ?? null,
        }
        const outPath = path.join(deploymentDir, `bridge-lock-${hre.network.name}.json`)
        fs.writeFileSync(outPath, JSON.stringify(record, null, 2))
        console.log(`\nDeployment record saved: ${outPath}`)
        console.log(`\n✅ BridgeLock deployed successfully at: ${proxyAddr}`)

        return record
    })
