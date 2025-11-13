import express from 'express'
import cors from 'cors'
import rateLimit from 'express-rate-limit'
import { ethers } from 'ethers'
import dotenv from 'dotenv'
import { fileURLToPath } from 'url'
import { dirname, join } from 'path'

const __filename = fileURLToPath(import.meta.url)
const __dirname = dirname(__filename)

// Load .env from the parent directory (faucet/.env)
dotenv.config({ path: join(__dirname, '../../.env') })

const app = express()
const PORT = process.env.PORT || 3001

// Configuration
const config = {
  rpcUrl: process.env.RPC_URL || 'http://node-1.test.ipc.space:8545',
  privateKey: process.env.PRIVATE_KEY,
  amount: process.env.FAUCET_AMOUNT || '1', // Amount in FIL
  rateLimitWindow: parseInt(process.env.RATE_LIMIT_WINDOW || '86400000'), // 24 hours in ms
  rateLimitMax: parseInt(process.env.RATE_LIMIT_MAX || '1'),
  enableCors: process.env.ENABLE_CORS !== 'false',
  serveStatic: process.env.SERVE_STATIC === 'true'
}

// Middleware
app.use(express.json())

if (config.enableCors) {
  app.use(cors())
}

// Rate limiting per IP
const ipLimiter = rateLimit({
  windowMs: config.rateLimitWindow,
  max: config.rateLimitMax,
  message: { error: 'Too many requests from this IP, please try again later' },
  standardHeaders: true,
  legacyHeaders: false,
})

// Rate limiting per address
const addressLimitStore = new Map()

function checkAddressRateLimit(address) {
  const now = Date.now()
  const lastRequest = addressLimitStore.get(address.toLowerCase())

  if (lastRequest && (now - lastRequest) < config.rateLimitWindow) {
    const timeLeft = config.rateLimitWindow - (now - lastRequest)
    const hoursLeft = Math.ceil(timeLeft / (1000 * 60 * 60))
    return {
      allowed: false,
      error: `This address has already requested tokens. Please try again in ${hoursLeft} hour(s).`
    }
  }

  return { allowed: true }
}

function recordAddressRequest(address) {
  addressLimitStore.set(address.toLowerCase(), Date.now())
}

// Cleanup old entries every hour
setInterval(() => {
  const now = Date.now()
  const cutoff = now - config.rateLimitWindow

  for (const [address, timestamp] of addressLimitStore.entries()) {
    if (timestamp < cutoff) {
      addressLimitStore.delete(address)
    }
  }
}, 3600000) // 1 hour

// Provider setup
let provider
let wallet
let isConfigured = false

function initializeWallet() {
  try {
    if (!config.privateKey) {
      console.warn('⚠️  WARNING: No PRIVATE_KEY configured. Faucet will not be able to send tokens.')
      console.warn('⚠️  Please set PRIVATE_KEY in your .env file')
      return false
    }

    provider = new ethers.JsonRpcProvider(config.rpcUrl)
    wallet = new ethers.Wallet(config.privateKey, provider)
    isConfigured = true

    console.log('✅ Wallet initialized')
    console.log(`   Address: ${wallet.address}`)

    return true
  } catch (error) {
    console.error('❌ Error initializing wallet:', error.message)
    return false
  }
}

// Routes
app.get('/api/health', (req, res) => {
  res.json({
    status: 'ok',
    configured: isConfigured,
    network: config.rpcUrl
  })
})

app.get('/api/config', (req, res) => {
  res.json({
    amount: config.amount,
    rateLimit: `1 request per ${config.rateLimitWindow / (1000 * 60 * 60)} hours per address`,
    network: config.rpcUrl
  })
})

app.post('/api/request', ipLimiter, async (req, res) => {
  try {
    const { address } = req.body

    // Validation
    if (!address) {
      return res.status(400).json({
        success: false,
        error: 'Address is required'
      })
    }

    if (!ethers.isAddress(address)) {
      return res.status(400).json({
        success: false,
        error: 'Invalid Ethereum address'
      })
    }

    if (!isConfigured) {
      return res.status(500).json({
        success: false,
        error: 'Faucet is not configured. Please contact the administrator.'
      })
    }

    // Check address rate limit
    const rateLimitCheck = checkAddressRateLimit(address)
    if (!rateLimitCheck.allowed) {
      return res.status(429).json({
        success: false,
        error: rateLimitCheck.error
      })
    }

    // Check faucet balance
    const balance = await provider.getBalance(wallet.address)
    const amountWei = ethers.parseEther(config.amount)

    if (balance < amountWei) {
      return res.status(503).json({
        success: false,
        error: 'Faucet is currently out of funds. Please contact the administrator.'
      })
    }

    console.log(`📤 Sending ${config.amount} tFIL to ${address}`)

    // Send transaction
    const tx = await wallet.sendTransaction({
      to: address,
      value: amountWei
    })

    console.log(`   Transaction hash: ${tx.hash}`)
    console.log(`   Waiting for confirmation...`)

    // Wait for confirmation
    const receipt = await tx.wait()

    console.log(`✅ Transaction confirmed in block ${receipt.blockNumber}`)

    // Record the request
    recordAddressRequest(address)

    res.json({
      success: true,
      txHash: tx.hash,
      amount: config.amount,
      blockNumber: receipt.blockNumber
    })

  } catch (error) {
    console.error('❌ Error processing request:', error)

    let errorMessage = 'Failed to process request'

    if (error.code === 'INSUFFICIENT_FUNDS') {
      errorMessage = 'Faucet has insufficient funds'
    } else if (error.code === 'NETWORK_ERROR') {
      errorMessage = 'Network error. Please try again later.'
    } else if (error.message) {
      errorMessage = error.message
    }

    res.status(500).json({
      success: false,
      error: errorMessage
    })
  }
})

// Serve static files in production
if (config.serveStatic) {
  const staticPath = join(__dirname, '../../frontend/dist')
  app.use(express.static(staticPath))

  app.get('*', (req, res) => {
    res.sendFile(join(staticPath, 'index.html'))
  })
}

// Start server
async function start() {
  console.log('🚀 Starting IPC tFIL Faucet Backend...')
  console.log('')
  console.log('Configuration:')
  console.log(`   RPC URL: ${config.rpcUrl}`)
  console.log(`   Amount per request: ${config.amount} tFIL`)
  console.log(`   Rate limit: ${config.rateLimitMax} request(s) per ${config.rateLimitWindow / (1000 * 60 * 60)} hour(s)`)
  console.log(`   Port: ${PORT}`)
  console.log('')

  const initialized = initializeWallet()

  if (initialized) {
    // Check and display balance
    try {
      const balance = await provider.getBalance(wallet.address)
      const balanceFIL = ethers.formatEther(balance)
      console.log(`💰 Faucet balance: ${balanceFIL} tFIL`)

      const maxRequests = Math.floor(parseFloat(balanceFIL) / parseFloat(config.amount))
      console.log(`   Can serve ~${maxRequests} requests`)
    } catch (error) {
      console.error('⚠️  Could not fetch balance:', error.message)
    }
  }

  console.log('')

  app.listen(PORT, () => {
    console.log(`✅ Server running on port ${PORT}`)
    console.log(`   Health check: http://localhost:${PORT}/api/health`)
    console.log('')

    if (!initialized) {
      console.log('⚠️  IMPORTANT: Configure PRIVATE_KEY to enable token distribution')
      console.log('')
    }
  })
}

start()

