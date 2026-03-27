/**
 * deploy-test-token
 * Deploy a mintable ERC20 test token (ERC20Mock) for smoke testing on any network.
 *
 * Usage:
 *   npx hardhat deploy-test-token --network calibration \
 *     [--name "Test USDC"] [--symbol "tUSDC"] [--mint-to <address>] [--mint-amount <uint256>]
 */
import { task, types } from 'hardhat/config'
import { HardhatRuntimeEnvironment, TaskArguments } from 'hardhat/types'
import * as fs from 'fs'
import * as path from 'path'

task('deploy-test-token', 'Deploy a mintable ERC20Mock test token')
    .addOptionalParam('name', 'Token name', 'Test Token', types.string)
    .addOptionalParam('symbol', 'Token symbol', 'TT', types.string)
    .addOptionalParam('mintTo', 'Address to mint initial supply to (defaults to deployer)', '', types.string)
    .addOptionalParam('mintAmount', 'Amount to mint in wei', '1000000000000000000000', types.string) // 1000 tokens
    .setAction(async (args: TaskArguments, hre: HardhatRuntimeEnvironment) => {
        await hre.run('compile')
        const [deployer] = await hre.ethers.getSigners()
        const mintTo = args.mintTo || deployer.address

        console.log(`\n=== Test Token Deployment ===`)
        console.log(`Network:  ${hre.network.name}`)
        console.log(`Deployer: ${deployer.address}`)
        console.log(`Name:     ${args.name}`)
        console.log(`Symbol:   ${args.symbol}`)
        console.log(`MintTo:   ${mintTo}`)
        console.log(`Amount:   ${args.mintAmount}`)

        const ERC20Mock = await hre.ethers.getContractFactory('ERC20Mock')
        const token = await ERC20Mock.deploy(args.name, args.symbol)
        await token.waitForDeployment()
        const addr = await token.getAddress()
        console.log(`\nDeployed: ${addr}`)

        await (await token.mint(mintTo, BigInt(args.mintAmount))).wait()
        console.log(`Minted ${args.mintAmount} to ${mintTo}`)

        const deploymentDir = path.join(__dirname, '..', 'deployments')
        if (!fs.existsSync(deploymentDir)) fs.mkdirSync(deploymentDir, { recursive: true })
        const record = {
            network: hre.network.name,
            testToken: addr,
            name: args.name,
            symbol: args.symbol,
            mintTo,
            mintAmount: args.mintAmount,
            deployedAt: new Date().toISOString(),
        }
        const outPath = path.join(deploymentDir, `test-token-${hre.network.name}.json`)
        fs.writeFileSync(outPath, JSON.stringify(record, null, 2))
        console.log(`Record saved: ${outPath}`)
        console.log(`\n"testToken": "${addr}"`)
        return record
    })
