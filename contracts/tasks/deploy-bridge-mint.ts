/**
 * deploy-bridge-mint
 *
 * Hardhat task to deploy BridgeMint.sol + WrappedToken.sol on Ethereum Sepolia.
 *
 * Usage:
 *   npx hardhat deploy-bridge-mint \
 *     --network sepolia \
 *     --gateway  <IPC_GATEWAY_ADDRESS_ON_SEPOLIA> \
 *     --src-root 314159 \
 *     --bridge-lock <BRIDGE_LOCK_ADDRESS_ON_CALIBRATION> \
 *     [--filecoin-token <ADDR> --token-name "Wrapped USDC (IPC Bridge)" --token-symbol "wUSDC.ipc"]
 *
 * Environment variables (via .env):
 *   PRIVATE_KEY  — deployer private key
 *
 * Outputs: deployments/bridge-mint-<network>.json
 */

import { task, types } from 'hardhat/config'
import { HardhatRuntimeEnvironment, TaskArguments } from 'hardhat/types'
import * as fs from 'fs'
import * as path from 'path'

task('deploy-bridge-mint', 'Deploy BridgeMint + WrappedToken (UUPS proxies) on Ethereum Sepolia')
    .addParam('gateway', 'Address of the IPC Gateway on this chain', undefined, types.string)
    .addParam('srcRoot', 'Chain ID (root) of the source subnet (Filecoin Calibration = 314159)', undefined, types.int)
    .addParam('bridgeLock', 'Address of BridgeLock on the source subnet', undefined, types.string)
    .addOptionalParam('admin', 'Admin address (defaults to deployer)', '', types.string)
    .addOptionalParam('filecoinToken', 'Filecoin-side token address to register (optional)', '', types.string)
    .addOptionalParam('tokenName', 'WrappedToken name (required if filecoinToken set)', '', types.string)
    .addOptionalParam('tokenSymbol', 'WrappedToken symbol (required if filecoinToken set)', '', types.string)
    .setAction(async (args: TaskArguments, hre: HardhatRuntimeEnvironment) => {
        await hre.run('compile')

        const [deployer] = await hre.ethers.getSigners()
        const adminAddr = args.admin || deployer.address

        console.log(`\n=== BridgeMint Deployment ===`)
        console.log(`Network:        ${hre.network.name}`)
        console.log(`Deployer:       ${deployer.address}`)
        console.log(`Admin:          ${adminAddr}`)
        console.log(`Gateway:        ${args.gateway}`)
        console.log(`Src root:       ${args.srcRoot}`)
        console.log(`BridgeLock:     ${args.bridgeLock}`)

        // ── 1. Deploy WrappedToken implementation ──────────────────────────────
        const WrappedToken = await hre.ethers.getContractFactory('WrappedToken')
        const wtImpl = await WrappedToken.deploy()
        await wtImpl.waitForDeployment()
        const wtImplAddr = await wtImpl.getAddress()
        console.log(`\nWrappedToken impl:      ${wtImplAddr}`)

        // ── 2. Deploy BridgeMint implementation ───────────────────────────────
        const BridgeMint = await hre.ethers.getContractFactory('BridgeMint')
        const bmImpl = await BridgeMint.deploy(args.gateway)
        await bmImpl.waitForDeployment()
        const bmImplAddr = await bmImpl.getAddress()
        console.log(`BridgeMint impl:        ${bmImplAddr}`)

        // ── 3. Deploy BridgeMint proxy ─────────────────────────────────────────
        const srcSubnet = { root: args.srcRoot, route: [] as string[] }
        const initData = BridgeMint.interface.encodeFunctionData('initialize', [
            adminAddr,
            srcSubnet,
            args.bridgeLock,
        ])
        const ERC1967Proxy = await hre.ethers.getContractFactory(
            '@openzeppelin/contracts/proxy/ERC1967/ERC1967Proxy.sol:ERC1967Proxy',
        )
        const bmProxy = await ERC1967Proxy.deploy(bmImplAddr, initData)
        await bmProxy.waitForDeployment()
        const bmProxyAddr = await bmProxy.getAddress()
        console.log(`BridgeMint proxy:       ${bmProxyAddr}`)

        const bridge = BridgeMint.attach(bmProxyAddr)

        // ── 4. Optionally register an initial asset ────────────────────────────
        let wrappedTokenAddr = ''
        if (args.filecoinToken && args.filecoinToken !== '') {
            console.log(`\nRegistering asset ${args.filecoinToken} → new WrappedToken...`)
            const tx = await bridge.deployAndRegisterAsset(
                args.filecoinToken,
                args.tokenName,
                args.tokenSymbol,
                wtImplAddr,
            )
            const receipt = await tx.wait()
            // Parse AssetRegistered event
            const iface = BridgeMint.interface
            const event = receipt?.logs
                .map((l: any) => { try { return iface.parseLog(l) } catch { return null } })
                .find((e: any) => e?.name === 'AssetRegistered')
            wrappedTokenAddr = event?.args?.wrappedToken ?? ''
            console.log(`  WrappedToken proxy: ${wrappedTokenAddr}`)
        }

        // ── 5. Verify state ───────────────────────────────────────────────────
        const adminRole = await bridge.DEFAULT_ADMIN_ROLE()
        const hasAdmin = await bridge.hasRole(adminRole, adminAddr)
        console.log(`\nVerification:`)
        console.log(`  Admin has DEFAULT_ADMIN_ROLE: ${hasAdmin}`)
        console.log(`  bridgeLockAddr:              ${await bridge.bridgeLockAddr()}`)

        // ── 6. Gas measurement ────────────────────────────────────────────────
        const bmImplReceipt = await hre.ethers.provider.getTransactionReceipt(bmImpl.deploymentTransaction()!.hash)
        const bmProxyReceipt = await hre.ethers.provider.getTransactionReceipt(bmProxy.deploymentTransaction()!.hash)
        console.log(`\nGas used:`)
        console.log(`  WrappedToken impl deploy: ${(await hre.ethers.provider.getTransactionReceipt(wtImpl.deploymentTransaction()!.hash))?.gasUsed?.toString() ?? 'n/a'}`)
        console.log(`  BridgeMint impl deploy:   ${bmImplReceipt?.gasUsed?.toString() ?? 'n/a'}`)
        console.log(`  BridgeMint proxy deploy:  ${bmProxyReceipt?.gasUsed?.toString() ?? 'n/a'}`)

        // ── 7. Save deployment record ─────────────────────────────────────────
        const deploymentDir = path.join(__dirname, '..', 'deployments')
        if (!fs.existsSync(deploymentDir)) fs.mkdirSync(deploymentDir, { recursive: true })

        const record = {
            network: hre.network.name,
            chainId: (await hre.ethers.provider.getNetwork()).chainId.toString(),
            deployer: deployer.address,
            admin: adminAddr,
            wrappedTokenImpl: wtImplAddr,
            bridgeMintImpl: bmImplAddr,
            bridgeMintProxy: bmProxyAddr,
            gateway: args.gateway,
            srcRoot: args.srcRoot,
            bridgeLock: args.bridgeLock,
            initialAsset: args.filecoinToken || null,
            initialWrappedToken: wrappedTokenAddr || null,
            deployedAt: new Date().toISOString(),
            bmImplDeployBlock: bmImplReceipt?.blockNumber ?? null,
            bmProxyDeployBlock: bmProxyReceipt?.blockNumber ?? null,
        }
        const outPath = path.join(deploymentDir, `bridge-mint-${hre.network.name}.json`)
        fs.writeFileSync(outPath, JSON.stringify(record, null, 2))
        console.log(`\nDeployment record saved: ${outPath}`)
        console.log(`\n✅ BridgeMint deployed at: ${bmProxyAddr}`)

        return record
    })
