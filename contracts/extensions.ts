import { extendProvider } from 'hardhat/config'
import '@nomiclabs/hardhat-ethers'

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

extendProvider((provider) => {
    const interceptedRpcMethods = ['eth_getBlockByNumber', 'eth_getBlockByHash']

    const originalProvider = provider.request.bind(provider)
    provider.request = async (args) => {
        try {
            return await originalProvider(args)
        } catch (err) {
            if (
                interceptedRpcMethods.includes(args.method) &&
                err.message.includes('requested epoch was a null round')
            ) {
                console.warn(`[${args.method}] null round hit, returning empty block`)
                return emptyBlock
            }
            console.log(`Rethrowing error ${err}`)
            throw err
        }
    }

    return provider
})
