import { extendEnvironment } from 'hardhat/config'
import { HardhatRuntimeEnvironment } from 'hardhat/types'
import '@nomiclabs/hardhat-ethers'
import { ProviderError } from 'hardhat/internal/core/providers/errors'

extendEnvironment((hre) => {
    injectFilecoinProvider(hre)
})

const emptyBlock = {
    number: '0x0',
    hash: '0x0000000000000000000000000000000000000000000000000000000000000000',
    parentHash: '0x0000000000000000000000000000000000000000000000000000000000000000',
    mixHash: '0x0000000000000000000000000000000000000000000000000000000000000000',
    nonce: '0x0000000000000000',
    sha3Uncles: '0x0000000000000000000000000000000000000000000000000000000000000000',
    logsBloom: '0x' + '00'.repeat(256),
    transactionsRoot: '0x0000000000000000000000000000000000000000000000000000000000000000',
    stateRoot: '0x0000000000000000000000000000000000000000000000000000000000000000',
    receiptsRoot: '0x0000000000000000000000000000000000000000000000000000000000000000',
    miner: '0x0000000000000000000000000000000000000000',
    difficulty: '0x0',
    totalDifficulty: '0x0',
    extraData: '0x',
    size: '0x0',
    gasLimit: '0x0',
    gasUsed: '0x0',
    timestamp: '0x0',
    transactions: [],
    uncles: [],
}

function injectFilecoinProvider(hre: HardhatRuntimeEnvironment) {
    const interceptedRpcMethods = ['eth_getBlockByNumber', 'eth_getBlockByHash']
    hre.network.provider = new Proxy(hre.network.provider, {
        get(target, prop, _receiver) {
            const orig = (target as any)[prop]
            // (prop === 'send' || prop === 'sendAsync')
            // Ethers / Web3 / Hardhat provider classes are a mess and they intermix through the call stack.
            // With this exact configuration, this code sees ExternalProvider#request(request: { method: string; params?: Array<any>; });
            // calls only. But if we switch to Hardhat Ignition, this is likely to change.
            if (!(typeof orig === 'function')) {
                return orig
            }
            let methodFunc: (args: any[]) => string
            if (prop === 'send' || prop === 'sendAsync') {
                methodFunc = ([method]: any[]) => method
            } else if (prop === 'request') {
                methodFunc = ([{ method }]: any[]) => method
            } else {
                return orig
            }
            return async (...args: any[]) => {
                try {
                    return await (target as any)[prop](...args)
                } catch (err) {
                    const method = methodFunc(args)
                    if (
                        interceptedRpcMethods.includes(method) &&
                        err.message.includes('requested epoch was a null round')
                    ) {
                        console.warn('null round hit, returning empty block')
                        return emptyBlock
                    }
                    console.log(`Rethrowing error ${err}`)
                    throw err
                }
            }
        },
    })
}
