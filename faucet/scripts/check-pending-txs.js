#!/usr/bin/env node

/**
 * Check Pending Transactions for IPC Faucet
 *
 * Helps diagnose stuck transactions
 */

import { ethers } from 'ethers'
import dotenv from 'dotenv'
import { fileURLToPath } from 'url'
import { dirname, join } from 'path'

const __filename = fileURLToPath(import.meta.url)
const __dirname = dirname(__filename)

// Load environment variables from parent directory
dotenv.config({ path: join(__dirname, '..', '.env') })

const RPC_URL = process.env.RPC_URL || 'http://node-1.test.ipc.space:8545'
const PRIVATE_KEY = process.env.PRIVATE_KEY

async function checkPendingTransactions() {
  try {
    if (!PRIVATE_KEY) {
      console.error('❌ Error: PRIVATE_KEY not found in .env file')
      process.exit(1)
    }

    console.log('\n🔍 Checking for pending transactions...\n')
    console.log(`RPC: ${RPC_URL}\n`)

    const provider = new ethers.JsonRpcProvider(RPC_URL)
    const wallet = new ethers.Wallet(PRIVATE_KEY, provider)

    console.log(`Wallet Address: ${wallet.address}\n`)

    // Get current nonce from network (includes pending)
    const pendingNonce = await provider.getTransactionCount(wallet.address, 'pending')

    // Get confirmed nonce
    const confirmedNonce = await provider.getTransactionCount(wallet.address, 'latest')

    // Get balance
    const balance = await provider.getBalance(wallet.address)
    const balanceFIL = ethers.formatEther(balance)

    console.log('━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━')
    console.log('📊 Wallet Status')
    console.log('━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━')
    console.log(`Balance: ${balanceFIL} tFIL`)
    console.log(`Confirmed Nonce: ${confirmedNonce}`)
    console.log(`Pending Nonce: ${pendingNonce}`)
    console.log(`Stuck Transactions: ${pendingNonce - confirmedNonce}`)
    console.log('━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n')

    if (pendingNonce === confirmedNonce) {
      console.log('✅ No pending transactions!\n')
      return
    }

    console.log('⚠️  Pending transactions detected!\n')
    console.log('Checking transaction details...\n')

    // Try to get pending transactions
    try {
      // Note: Not all RPC endpoints support this method
      const pendingBlock = await provider.send('eth_getBlockByNumber', ['pending', true])

      if (pendingBlock && pendingBlock.transactions) {
        const myPendingTxs = pendingBlock.transactions.filter(
          tx => tx.from && tx.from.toLowerCase() === wallet.address.toLowerCase()
        )

        if (myPendingTxs.length > 0) {
          console.log(`Found ${myPendingTxs.length} pending transaction(s):\n`)

          myPendingTxs.forEach((tx, index) => {
            console.log(`Transaction ${index + 1}:`)
            console.log(`  Hash: ${tx.hash}`)
            console.log(`  To: ${tx.to}`)
            console.log(`  Value: ${ethers.formatEther(tx.value)} tFIL`)
            console.log(`  Nonce: ${parseInt(tx.nonce)}`)
            console.log(`  Gas Price: ${tx.gasPrice ? ethers.formatUnits(tx.gasPrice, 'gwei') : 'N/A'} Gwei`)
            console.log('')
          })
        }
      }
    } catch (error) {
      console.log('ℹ️  Could not fetch pending transaction details (RPC may not support this)\n')
    }

    // Check recent confirmed transactions
    console.log('📜 Recent confirmed transactions:\n')

    try {
      const latestBlock = await provider.getBlockNumber()
      const fromBlock = Math.max(0, latestBlock - 20) // Check last 20 blocks

      let foundTxs = 0
      for (let i = latestBlock; i >= fromBlock && foundTxs < 5; i--) {
        const block = await provider.getBlock(i, true)
        if (block && block.transactions) {
          for (const tx of block.transactions) {
            if (tx.from && tx.from.toLowerCase() === wallet.address.toLowerCase()) {
              const receipt = await provider.getTransactionReceipt(tx.hash)
              console.log(`Block ${i}:`)
              console.log(`  Hash: ${tx.hash}`)
              console.log(`  To: ${tx.to}`)
              console.log(`  Value: ${ethers.formatEther(tx.value || 0)} tFIL`)
              console.log(`  Nonce: ${parseInt(tx.nonce)}`)
              console.log(`  Status: ${receipt.status === 1 ? '✅ Success' : '❌ Failed'}`)
              console.log('')
              foundTxs++
              if (foundTxs >= 5) break
            }
          }
        }
      }

      if (foundTxs === 0) {
        console.log('  No recent transactions found\n')
      }
    } catch (error) {
      console.log('  Could not fetch recent transactions\n')
    }

    // Provide solutions
    console.log('━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━')
    console.log('💡 Solutions to Clear Stuck Transactions')
    console.log('━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n')

    console.log('Option 1: Wait for transactions to be mined')
    console.log('  - Transactions may just need more time\n')

    console.log('Option 2: Speed up with higher gas (if RPC supports)')
    console.log('  - Use node scripts/speed-up-tx.js\n')

    console.log('Option 3: Cancel stuck transactions')
    console.log('  - Send 0 value tx to yourself with same nonce')
    console.log('  - Use node scripts/cancel-tx.js <nonce>\n')

    console.log('Option 4: Check gas price settings')
    console.log('  - Ensure faucet is using adequate gas price')
    console.log('  - Check network congestion\n')

    // Get network gas info
    try {
      const feeData = await provider.getFeeData()
      console.log('Current Network Gas Prices:')
      if (feeData.gasPrice) {
        console.log(`  Gas Price: ${ethers.formatUnits(feeData.gasPrice, 'gwei')} Gwei`)
      }
      if (feeData.maxFeePerGas) {
        console.log(`  Max Fee: ${ethers.formatUnits(feeData.maxFeePerGas, 'gwei')} Gwei`)
      }
      if (feeData.maxPriorityFeePerGas) {
        console.log(`  Max Priority Fee: ${ethers.formatUnits(feeData.maxPriorityFeePerGas, 'gwei')} Gwei`)
      }
      console.log('')
    } catch (error) {
      console.log('  Could not fetch gas prices\n')
    }

  } catch (error) {
    console.error('❌ Error:', error.message)
    process.exit(1)
  }
}

checkPendingTransactions()



