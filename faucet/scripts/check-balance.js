#!/usr/bin/env node

/**
 * Balance Checker for IPC Faucet
 *
 * Checks the balance of the faucet wallet
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
const FAUCET_AMOUNT = process.env.FAUCET_AMOUNT || '1'

async function checkBalance() {
  try {
    if (!PRIVATE_KEY) {
      console.error('❌ Error: PRIVATE_KEY not found in .env file')
      console.error('   Please configure your .env file first')
      process.exit(1)
    }

    console.log('\n🔍 Checking faucet balance...\n')
    console.log(`RPC: ${RPC_URL}`)

    const provider = new ethers.JsonRpcProvider(RPC_URL)
    const wallet = new ethers.Wallet(PRIVATE_KEY, provider)

    console.log(`Address: ${wallet.address}\n`)

    const balance = await provider.getBalance(wallet.address)
    const balanceFIL = ethers.formatEther(balance)
    const balanceNum = parseFloat(balanceFIL)

    console.log('━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━')
    console.log(`💰 Balance: ${balanceFIL} tFIL`)
    console.log('━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n')

    const amountPerRequest = parseFloat(FAUCET_AMOUNT)
    const maxRequests = Math.floor(balanceNum / amountPerRequest)

    console.log(`📊 Statistics:`)
    console.log(`   • Amount per request: ${FAUCET_AMOUNT} tFIL`)
    console.log(`   • Estimated requests remaining: ~${maxRequests}`)
    console.log(`   • Days of operation (at 100 req/day): ~${Math.floor(maxRequests / 100)}`)
    console.log('')

    if (balanceNum < amountPerRequest) {
      console.log('⚠️  WARNING: Insufficient balance!')
      console.log('   Please fund the faucet wallet with more tFIL\n')
    } else if (balanceNum < amountPerRequest * 10) {
      console.log('⚠️  WARNING: Balance is running low!')
      console.log('   Consider adding more tFIL soon\n')
    } else {
      console.log('✅ Balance looks good!\n')
    }

  } catch (error) {
    console.error('❌ Error:', error.message)
    process.exit(1)
  }
}

checkBalance()

