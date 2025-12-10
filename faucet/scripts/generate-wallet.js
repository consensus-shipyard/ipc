#!/usr/bin/env node

/**
 * Wallet Generator for IPC Faucet
 *
 * Generates a new Ethereum wallet with address and private key
 * Use this to create a new wallet for your faucet
 */

import { ethers } from 'ethers'

console.log('\n🔐 Generating new wallet for IPC Faucet...\n')

const wallet = ethers.Wallet.createRandom()

console.log('✅ Wallet generated successfully!\n')
console.log('━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━')
console.log('📋 ADDRESS:')
console.log('   ' + wallet.address)
console.log('\n🔑 PRIVATE KEY:')
console.log('   ' + wallet.privateKey)
console.log('━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n')

console.log('⚠️  IMPORTANT SECURITY NOTES:')
console.log('   • Keep your private key SECRET')
console.log('   • Never share it or commit it to version control')
console.log('   • Store it securely (use a password manager)')
console.log('   • This wallet is only for testnet use\n')

console.log('📝 Next steps:')
console.log('   1. Save the private key securely')
console.log('   2. Fund this address with tFIL tokens')
console.log('   3. Add the private key to your .env file:')
console.log('      PRIVATE_KEY=' + wallet.privateKey)
console.log('')

