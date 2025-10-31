<template>
  <div class="min-h-screen flex flex-col items-center justify-center p-4">
    <!-- Background decoration -->
    <div class="absolute inset-0 overflow-hidden pointer-events-none">
      <div class="absolute top-20 left-10 w-72 h-72 bg-blue-500 rounded-full mix-blend-multiply filter blur-3xl opacity-10 animate-pulse-slow"></div>
      <div class="absolute top-40 right-10 w-72 h-72 bg-purple-500 rounded-full mix-blend-multiply filter blur-3xl opacity-10 animate-pulse-slow animation-delay-2000"></div>
      <div class="absolute bottom-20 left-1/2 w-72 h-72 bg-pink-500 rounded-full mix-blend-multiply filter blur-3xl opacity-10 animate-pulse-slow animation-delay-4000"></div>
    </div>

    <!-- Main content -->
    <div class="relative z-10 w-full max-w-2xl">
      <!-- Header -->
      <div class="text-center mb-8">
        <h1 class="text-5xl font-bold text-white mb-3 tracking-tight">
          IPC tFIL Faucet
        </h1>
        <p class="text-blue-200 text-lg">
          Get test FIL tokens for IPC testnet development
        </p>
      </div>

      <!-- Main card -->
      <div class="bg-white/10 backdrop-blur-lg rounded-2xl shadow-2xl p-8 border border-white/20">
        <!-- Network info -->
        <div class="mb-6 p-4 bg-blue-500/20 rounded-lg border border-blue-400/30">
          <div class="flex items-center justify-between">
            <div>
              <p class="text-sm text-blue-200 mb-1">Current Network</p>
              <p class="text-white font-semibold">{{ networkInfo.name }}</p>
            </div>
            <div class="text-right">
              <p class="text-sm text-blue-200 mb-1">Chain ID</p>
              <p class="text-white font-mono">{{ networkInfo.chainId }}</p>
            </div>
          </div>
        </div>

        <!-- Network warning -->
        <div v-if="connectedAddress && !isCorrectNetwork" class="mb-6 p-4 bg-amber-500/20 rounded-lg border border-amber-400/30">
          <div class="flex items-start gap-3">
            <svg class="w-6 h-6 text-amber-400 flex-shrink-0 mt-0.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
            </svg>
            <div class="flex-1">
              <p class="text-amber-100 font-semibold mb-1">Wrong Network</p>
              <p class="text-amber-200 text-sm mb-3">You're currently on the wrong network. Please switch to IPC Testnet to request tokens.</p>
              <button
                @click="switchNetwork"
                class="py-2 px-4 bg-amber-600 hover:bg-amber-700 text-white font-medium rounded-lg transition-all duration-200 text-sm"
              >
                Switch to IPC Testnet
              </button>
            </div>
          </div>
        </div>

        <!-- Wallet connection -->
        <div class="mb-6">
          <button
            v-if="!connectedAddress"
            @click="connectWallet"
            class="w-full py-4 px-6 bg-gradient-to-r from-blue-600 to-blue-700 hover:from-blue-700 hover:to-blue-800 text-white font-semibold rounded-lg transition-all duration-200 shadow-lg hover:shadow-xl transform hover:-translate-y-0.5"
          >
            Connect MetaMask
          </button>
          <div v-else class="p-4 rounded-lg border" :class="isCorrectNetwork ? 'bg-green-500/20 border-green-400/30' : 'bg-blue-500/20 border-blue-400/30'">
            <p class="text-sm mb-1" :class="isCorrectNetwork ? 'text-green-200' : 'text-blue-200'">Connected Wallet</p>
            <p class="text-white font-mono text-sm break-all">{{ connectedAddress }}</p>
          </div>
        </div>

        <!-- Address input -->
        <div class="mb-6">
          <label class="block text-white text-sm font-medium mb-2">
            Recipient Address
          </label>
          <input
            v-model="recipientAddress"
            type="text"
            placeholder="0x..."
            class="w-full px-4 py-3 bg-white/10 border border-white/20 rounded-lg text-white placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent transition-all"
          />
        </div>

        <!-- Request button -->
        <button
          @click="requestTokens"
          :disabled="!recipientAddress || isLoading || !isCorrectNetwork"
          class="w-full py-4 px-6 bg-gradient-to-r from-purple-600 to-pink-600 hover:from-purple-700 hover:to-pink-700 disabled:from-gray-600 disabled:to-gray-700 disabled:cursor-not-allowed text-white font-bold rounded-lg transition-all duration-200 shadow-lg hover:shadow-xl transform hover:-translate-y-0.5 disabled:transform-none disabled:shadow-none"
        >
          <span v-if="isLoading" class="flex items-center justify-center">
            <svg class="animate-spin -ml-1 mr-3 h-5 w-5 text-white" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
              <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
              <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
            </svg>
            Processing...
          </span>
          <span v-else-if="!isCorrectNetwork && connectedAddress">Switch Network to Continue</span>
          <span v-else>Request {{ faucetConfig.amount }} tFIL</span>
        </button>

        <!-- Status messages -->
        <transition name="fade">
          <div v-if="statusMessage" class="mt-6 p-4 rounded-lg" :class="statusClass">
            <p class="text-sm font-medium">{{ statusMessage }}</p>
            <a v-if="txHash" :href="`${networkInfo.explorer}/tx/${txHash}`" target="_blank" class="text-sm underline mt-2 block break-all">
              View transaction: {{ txHash }}
            </a>
          </div>
        </transition>

        <!-- Network switcher (only show if not on correct network or not connected) -->
        <div v-if="!connectedAddress || !isCorrectNetwork" class="mt-6 pt-6 border-t border-white/10">
          <button
            @click="switchNetwork"
            class="w-full py-3 px-4 bg-white/5 hover:bg-white/10 border border-white/20 text-white font-medium rounded-lg transition-all duration-200"
          >
            <span v-if="!connectedAddress">Need to add IPC Testnet?</span>
            <span v-else>Switch to IPC Testnet</span>
          </button>
        </div>

        <!-- Info section -->
        <div class="mt-6 p-4 bg-white/5 rounded-lg border border-white/10">
          <h3 class="text-white font-semibold mb-2">Faucet Information</h3>
          <ul class="text-sm text-gray-300 space-y-1">
            <li>• Amount per request: {{ faucetConfig.amount }} tFIL</li>
            <li>• Rate limit: {{ faucetConfig.rateLimit }}</li>
            <li>• Network: {{ networkInfo.name }}</li>
          </ul>
        </div>
      </div>

      <!-- Footer -->
      <div class="text-center mt-8">
        <p class="text-gray-400 text-sm">
          Need help? Check the <a href="https://docs.ipc.space" target="_blank" class="text-blue-400 hover:text-blue-300 underline">IPC documentation</a>
        </p>
      </div>
    </div>
  </div>
</template>

<script setup>
import axios from 'axios'
import { ethers } from 'ethers'
import { computed, onMounted, ref } from 'vue'

// State
const connectedAddress = ref('')
const recipientAddress = ref('')
const isLoading = ref(false)
const statusMessage = ref('')
const statusType = ref('') // 'success', 'error', 'info'
const txHash = ref('')
const currentChainId = ref('')
const isCorrectNetwork = ref(false)

// Network configuration
const networkInfo = ref({
  name: 'IPC Testnet',
  chainId: '0x5e179', // 385401 in hex
  rpcUrl: 'http://node-1.test.ipc.space:8545',
  explorer: 'http://node-1.test.ipc.space:8545' // Update if explorer available
})

// Faucet configuration (will be fetched from backend)
const faucetConfig = ref({
  amount: '1',
  rateLimit: '1 request per 24 hours per address'
})

// Computed
const statusClass = computed(() => {
  const base = 'border'
  if (statusType.value === 'success') return `${base} bg-green-500/20 border-green-400/30 text-green-100`
  if (statusType.value === 'error') return `${base} bg-red-500/20 border-red-400/30 text-red-100`
  return `${base} bg-blue-500/20 border-blue-400/30 text-blue-100`
})

// Methods
async function checkNetwork() {
  try {
    if (!window.ethereum) return

    const provider = new ethers.BrowserProvider(window.ethereum)
    const network = await provider.getNetwork()
    currentChainId.value = '0x' + network.chainId.toString(16)
    isCorrectNetwork.value = currentChainId.value.toLowerCase() === networkInfo.value.chainId.toLowerCase()
  } catch (error) {
    console.error('Error checking network:', error)
    isCorrectNetwork.value = false
  }
}

async function connectWallet() {
  try {
    if (!window.ethereum) {
      setStatus('Please install MetaMask to use this faucet', 'error')
      return
    }

    const provider = new ethers.BrowserProvider(window.ethereum)
    const accounts = await provider.send('eth_requestAccounts', [])

    if (accounts.length > 0) {
      connectedAddress.value = accounts[0]
      recipientAddress.value = accounts[0]
      await checkNetwork()
      
      if (isCorrectNetwork.value) {
        setStatus('Wallet connected successfully!', 'success')
      } else {
        setStatus('Wallet connected! Please switch to IPC Testnet to continue.', 'info')
      }

      // Clear success message after 3 seconds
      setTimeout(() => {
        if (statusType.value === 'success') {
          statusMessage.value = ''
        }
      }, 3000)
    }
  } catch (error) {
    console.error('Error connecting wallet:', error)
    setStatus('Failed to connect wallet: ' + error.message, 'error')
  }
}

async function requestTokens() {
  if (!recipientAddress.value) {
    setStatus('Please enter a recipient address', 'error')
    return
  }

  // Validate address
  if (!ethers.isAddress(recipientAddress.value)) {
    setStatus('Invalid Ethereum address', 'error')
    return
  }

  // Check network before proceeding
  if (!isCorrectNetwork.value) {
    setStatus('Please switch to IPC Testnet first', 'error')
    return
  }

  isLoading.value = true
  txHash.value = ''
  setStatus('Requesting tokens...', 'info')

  try {
    const response = await axios.post('/api/request', {
      address: recipientAddress.value
    })

    if (response.data.success) {
      txHash.value = response.data.txHash
      setStatus(
        `Success! ${faucetConfig.value.amount} tFIL sent to your address.`,
        'success'
      )
    } else {
      setStatus(response.data.error || 'Request failed', 'error')
    }
  } catch (error) {
    console.error('Error requesting tokens:', error)
    const errorMsg = error.response?.data?.error || error.message || 'Failed to request tokens'
    setStatus(errorMsg, 'error')
  } finally {
    isLoading.value = false
  }
}

async function switchNetwork() {
  try {
    if (!window.ethereum) {
      setStatus('Please install MetaMask', 'error')
      return
    }

    await window.ethereum.request({
      method: 'wallet_switchEthereumChain',
      params: [{ chainId: networkInfo.value.chainId }],
    })

    await checkNetwork()
    setStatus('Network switched successfully!', 'success')
    
    // Clear success message after 3 seconds
    setTimeout(() => {
      statusMessage.value = ''
    }, 3000)
  } catch (error) {
    // If network doesn't exist, add it
    if (error.code === 4902) {
      try {
        await window.ethereum.request({
          method: 'wallet_addEthereumChain',
          params: [{
            chainId: networkInfo.value.chainId,
            chainName: networkInfo.value.name,
            nativeCurrency: {
              name: 'tFIL',
              symbol: 'tFIL',
              decimals: 18
            },
            rpcUrls: [networkInfo.value.rpcUrl],
            blockExplorerUrls: [networkInfo.value.explorer]
          }]
        })
        await checkNetwork()
        setStatus('Network added and switched successfully!', 'success')
        
        // Clear success message after 3 seconds
        setTimeout(() => {
          statusMessage.value = ''
        }, 3000)
      } catch (addError) {
        console.error('Error adding network:', addError)
        setStatus('Failed to add network: ' + addError.message, 'error')
      }
    } else {
      console.error('Error switching network:', error)
      setStatus('Failed to switch network: ' + error.message, 'error')
    }
  }
}

function setStatus(message, type) {
  statusMessage.value = message
  statusType.value = type
}

async function fetchFaucetConfig() {
  try {
    const response = await axios.get('/api/config')
    if (response.data) {
      faucetConfig.value = response.data
    }
  } catch (error) {
    console.error('Error fetching faucet config:', error)
  }
}

// Lifecycle
onMounted(() => {
  fetchFaucetConfig()

  // Check if already connected
  if (window.ethereum?.selectedAddress) {
    connectedAddress.value = window.ethereum.selectedAddress
    recipientAddress.value = window.ethereum.selectedAddress
    checkNetwork()
  }

  // Listen for account changes
  if (window.ethereum) {
    window.ethereum.on('accountsChanged', (accounts) => {
      if (accounts.length > 0) {
        connectedAddress.value = accounts[0]
        recipientAddress.value = accounts[0]
      } else {
        connectedAddress.value = ''
        recipientAddress.value = ''
        isCorrectNetwork.value = false
      }
    })

    // Listen for network changes
    window.ethereum.on('chainChanged', () => {
      checkNetwork()
    })
  }
})
</script>

<style scoped>
.fade-enter-active, .fade-leave-active {
  transition: opacity 0.3s ease;
}

.fade-enter-from, .fade-leave-to {
  opacity: 0;
}
</style>

