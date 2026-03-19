/**
 * deploy-eth-gateway
 * Deploy EthGatewayMessenger on Ethereum Sepolia (or any EVM network).
 *
 * Usage:
 *   npx hardhat deploy-eth-gateway \
 *     --network sepolia \
 *     --subnet-root 11155111 \
 *     [--owner <address>]
 *     [--require-approved false]
 */
import { task, types } from 'hardhat/config'
import { HardhatRuntimeEnvironment, TaskArguments } from 'hardhat/types'
import * as fs from 'fs'
import * as path from 'path'

task('deploy-eth-gateway', 'Deploy EthGatewayMessenger on Ethereum Sepolia')
    .addParam('subnetRoot', 'Chain ID of the subnet root (11155111 for Sepolia)', undefined, types.int)
    .addOptionalParam('owner', 'Owner address (defaults to deployer)', '', types.string)
    .addOptionalParam('requireApproved', 'Enable subnet allowlist', 'false', types.string)
    .setAction(async (args: TaskArguments, hre: HardhatRuntimeEnvironment) => {
        await hre.run('compile')
        const [deployer] = await hre.ethers.getSigners()
        const ownerAddr = args.owner || deployer.address

        console.log(`\n=== EthGatewayMessenger Deployment ===`)
        console.log(`Network:       ${hre.network.name}`)
        console.log(`Deployer:      ${deployer.address}`)
        console.log(`Owner:         ${ownerAddr}`)
        console.log(`Subnet root:   ${args.subnetRoot}`)

        const EthGateway = await hre.ethers.getContractFactory('EthGatewayMessenger')
        const networkName = { root: args.subnetRoot, route: [] as string[] }
        const gw = await EthGateway.deploy(ownerAddr, networkName)
        await gw.waitForDeployment()
        const addr = await gw.getAddress()
        console.log(`\nDeployed: ${addr}`)

        if (args.requireApproved === 'true') {
            await (await gw.setRequireApprovedSubnet(true)).wait()
            console.log('  Subnet allowlist: enabled')
        }

        const receipt = await hre.ethers.provider.getTransactionReceipt(gw.deploymentTransaction()!.hash)
        console.log(`Gas used: ${receipt?.gasUsed?.toString() ?? 'n/a'}`)

        const deploymentDir = path.join(__dirname, '..', 'deployments')
        if (!fs.existsSync(deploymentDir)) fs.mkdirSync(deploymentDir, { recursive: true })
        const record = {
            network: hre.network.name,
            chainId: (await hre.ethers.provider.getNetwork()).chainId.toString(),
            ethGatewayMessenger: addr,
            owner: ownerAddr,
            subnetRoot: args.subnetRoot,
            deployedAt: new Date().toISOString(),
            deployBlock: receipt?.blockNumber ?? null,
        }
        const outPath = path.join(deploymentDir, `eth-gateway-${hre.network.name}.json`)
        fs.writeFileSync(outPath, JSON.stringify(record, null, 2))
        console.log(`\nDeployment record saved: ${outPath}`)
        console.log(`\n✅ EthGatewayMessenger deployed at: ${addr}`)
        return record
    })
