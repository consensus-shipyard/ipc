/**
 * set-bridge-destination
 * Update the destination subnet + BridgeMint address on a deployed BridgeLock proxy.
 * Run this after deploying BridgeMint to wire the two contracts together.
 *
 * Usage:
 *   npx hardhat set-bridge-destination \
 *     --network calibration \
 *     --bridge-lock <BRIDGE_LOCK_PROXY> \
 *     --dest-root 314159 \
 *     --dest-receiver <BRIDGE_MINT_PROXY>
 */
import { task, types } from 'hardhat/config'
import { HardhatRuntimeEnvironment, TaskArguments } from 'hardhat/types'

task('set-bridge-destination', 'Wire BridgeLock to a BridgeMint destination')
    .addParam('bridgeLock', 'BridgeLock proxy address', undefined, types.string)
    .addParam('destRoot', 'Destination subnet root chain ID', undefined, types.int)
    .addParam('destReceiver', 'BridgeMint proxy address on destination chain', undefined, types.string)
    .setAction(async (args: TaskArguments, hre: HardhatRuntimeEnvironment) => {
        const [deployer] = await hre.ethers.getSigners()
        const BridgeLock = await hre.ethers.getContractFactory('BridgeLock')
        const bridge = BridgeLock.attach(args.bridgeLock)

        const destSubnet = { root: args.destRoot, route: [] as string[] }

        console.log(`\n=== Set Bridge Destination ===`)
        console.log(`Network:       ${hre.network.name}`)
        console.log(`BridgeLock:    ${args.bridgeLock}`)
        console.log(`Dest root:     ${args.destRoot}`)
        console.log(`Dest receiver: ${args.destReceiver}`)

        const tx = await bridge.connect(deployer).setDestination(destSubnet, args.destReceiver)
        await tx.wait()
        console.log(`\n✅ Destination updated. Tx: ${tx.hash}`)
    })
