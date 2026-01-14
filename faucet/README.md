# IPC tFIL Faucet

A modern, production-ready faucet for distributing test FIL tokens on the IPC testnet. Built with Vue 3, Tailwind CSS, and Express.

![Faucet Preview](https://img.shields.io/badge/Vue-3.x-4FC08D?logo=vue.js&logoColor=white)
![Tailwind CSS](https://img.shields.io/badge/Tailwind-3.x-38B2AC?logo=tailwind-css&logoColor=white)
![Express](https://img.shields.io/badge/Express-4.x-000000?logo=express&logoColor=white)
![Docker](https://img.shields.io/badge/Docker-Ready-2496ED?logo=docker&logoColor=white)

## Features

✨ **Modern UI**
- Clean, responsive design with Tailwind CSS
- Beautiful gradient backgrounds and animations
- Dark theme optimized for crypto applications

🔐 **Secure & Robust**
- IP-based rate limiting
- Address-based rate limiting
- Configurable distribution amounts
- Environment-based configuration

🦊 **Web3 Integration**
- MetaMask wallet connection
- Network switcher for easy testnet setup
- Address validation
- Transaction status tracking

🐳 **Production Ready**
- Docker containerization
- Health checks
- Structured logging
- Easy GCP VM deployment

## Quick Start

### Prerequisites

- Node.js 18+ and npm
- Docker and Docker Compose (for containerized deployment)
- A funded wallet with tFIL tokens
- Access to IPC testnet RPC endpoint

### Local Development

1. **Clone and install dependencies:**

\`\`\`bash
cd faucet
npm run install:all
\`\`\`

2. **Configure the faucet:**

Create a `.env` file in the root directory:

\`\`\`bash
# Required: Your faucet wallet private key
PRIVATE_KEY=0x1234567890abcdef...

# RPC endpoint
RPC_URL=http://node-1.test.ipc.space:8545

# Amount to send per request (in FIL)
FAUCET_AMOUNT=1

# Rate limiting (24 hours in milliseconds)
RATE_LIMIT_WINDOW=86400000
RATE_LIMIT_MAX=1

# Server port
PORT=3001

# Development settings
ENABLE_CORS=true
SERVE_STATIC=false
\`\`\`

3. **Start the development servers:**

\`\`\`bash
npm run dev
\`\`\`

This will start:
- Frontend on http://localhost:3000
- Backend on http://localhost:3001

### Docker Deployment (Recommended for Production)

1. **Create `.env` file:**

\`\`\`bash
PRIVATE_KEY=your_private_key_here
RPC_URL=http://node-1.test.ipc.space:8545
FAUCET_AMOUNT=1
RATE_LIMIT_WINDOW=86400000
RATE_LIMIT_MAX=1
\`\`\`

2. **Build and run with Docker Compose:**

\`\`\`bash
docker-compose up -d
\`\`\`

The faucet will be available on http://localhost:3001

3. **Check logs:**

\`\`\`bash
docker-compose logs -f
\`\`\`

4. **Stop the faucet:**

\`\`\`bash
docker-compose down
\`\`\`

## GCP VM Deployment

### Option 1: Using Docker Compose (Recommended)

1. **SSH into your GCP VM:**

\`\`\`bash
gcloud compute ssh your-vm-name --zone=your-zone
\`\`\`

2. **Install Docker and Docker Compose:**

\`\`\`bash
# Install Docker
curl -fsSL https://get.docker.com -o get-docker.sh
sudo sh get-docker.sh
sudo usermod -aG docker $USER

# Install Docker Compose
sudo curl -L "https://github.com/docker/compose/releases/latest/download/docker-compose-$(uname -s)-$(uname -m)" -o /usr/local/bin/docker-compose
sudo chmod +x /usr/local/bin/docker-compose

# Log out and back in for group changes to take effect
exit
\`\`\`

3. **Clone the repository:**

\`\`\`bash
git clone https://github.com/your-org/ipc.git
cd ipc/faucet
\`\`\`

4. **Create `.env` file:**

\`\`\`bash
nano .env
# Add your configuration (see example above)
\`\`\`

5. **Start the faucet:**

\`\`\`bash
docker-compose up -d
\`\`\`

6. **Configure firewall:**

\`\`\`bash
# Allow port 3001
gcloud compute firewall-rules create allow-faucet \
  --allow tcp:3001 \
  --source-ranges 0.0.0.0/0 \
  --description "Allow IPC faucet access"
\`\`\`

7. **Access your faucet:**

Visit `http://YOUR_VM_EXTERNAL_IP:3001`

### Option 2: Using Systemd Service

1. **Build the application:**

\`\`\`bash
cd ipc/faucet
npm run install:all
cd frontend && npm run build
\`\`\`

2. **Create systemd service:**

\`\`\`bash
sudo nano /etc/systemd/system/ipc-faucet.service
\`\`\`

Add the following content:

\`\`\`ini
[Unit]
Description=IPC tFIL Faucet
After=network.target

[Service]
Type=simple
User=your_username
WorkingDirectory=/home/your_username/ipc/faucet/backend
Environment=NODE_ENV=production
Environment=SERVE_STATIC=true
EnvironmentFile=/home/your_username/ipc/faucet/.env
ExecStart=/usr/bin/node src/index.js
Restart=always
RestartSec=10

[Install]
WantedBy=multi-user.target
\`\`\`

3. **Enable and start the service:**

\`\`\`bash
sudo systemctl daemon-reload
sudo systemctl enable ipc-faucet
sudo systemctl start ipc-faucet
sudo systemctl status ipc-faucet
\`\`\`

## Setting Up Your Faucet Wallet

### Creating a New Wallet

1. **Generate a new wallet:**

\`\`\`bash
# Using ethers.js CLI or any Ethereum wallet tool
node -e "const ethers = require('ethers'); const wallet = ethers.Wallet.createRandom(); console.log('Address:', wallet.address); console.log('Private Key:', wallet.privateKey);"
\`\`\`

2. **Fund the wallet:**

Transfer tFIL tokens to the generated address. The amount depends on how many requests you expect to serve.

**Example calculation:**
- 1 tFIL per request
- 1000 expected requests
- Total needed: 1000 tFIL + buffer for gas fees = ~1010 tFIL

3. **Secure your private key:**

Store your private key securely:
- Use environment variables (never commit to git)
- Use secret management services (GCP Secret Manager, AWS Secrets Manager, etc.)
- Limit access to the server

### Using an Existing Wallet

If you already have a wallet with tFIL:

1. **Export private key from MetaMask:**
   - Click on account details
   - Click "Export Private Key"
   - Enter your password
   - Copy the private key

2. **Add to `.env` file:**
   \`\`\`
   PRIVATE_KEY=0xYOUR_PRIVATE_KEY_HERE
   \`\`\`

## Configuration Options

### Environment Variables

| Variable | Description | Default | Required |
|----------|-------------|---------|----------|
| `PRIVATE_KEY` | Faucet wallet private key | - | ✅ Yes |
| `RPC_URL` | IPC testnet RPC endpoint | `http://node-1.test.ipc.space:8545` | No |
| `FAUCET_AMOUNT` | Amount of tFIL per request | `1` | No |
| `RATE_LIMIT_WINDOW` | Rate limit window in ms | `86400000` (24h) | No |
| `RATE_LIMIT_MAX` | Max requests per window per IP | `1` | No |
| `PORT` | Server port | `3001` | No |
| `ENABLE_CORS` | Enable CORS | `true` | No |
| `SERVE_STATIC` | Serve frontend files | `false` (dev), `true` (prod) | No |

### Customizing Rate Limits

**Per hour instead of 24 hours:**
\`\`\`bash
RATE_LIMIT_WINDOW=3600000  # 1 hour in milliseconds
RATE_LIMIT_MAX=1
\`\`\`

**Multiple requests per day:**
\`\`\`bash
RATE_LIMIT_WINDOW=86400000  # 24 hours
RATE_LIMIT_MAX=3  # 3 requests per 24 hours
\`\`\`

**Higher distribution amount:**
\`\`\`bash
FAUCET_AMOUNT=5  # 5 tFIL per request
\`\`\`

## Monitoring

### Health Check

\`\`\`bash
curl http://localhost:3001/api/health
\`\`\`

Response:
\`\`\`json
{
  "status": "ok",
  "configured": true,
  "network": "http://node-1.test.ipc.space:8545"
}
\`\`\`

### Check Faucet Balance

The backend logs the faucet balance on startup:

\`\`\`bash
docker-compose logs faucet | grep "Faucet balance"
\`\`\`

### Logs

**Docker Compose:**
\`\`\`bash
docker-compose logs -f
\`\`\`

**Systemd:**
\`\`\`bash
sudo journalctl -u ipc-faucet -f
\`\`\`

## Security Best Practices

1. **Private Key Security**
   - Never commit private keys to version control
   - Use environment variables or secret management services
   - Rotate keys periodically
   - Use a dedicated wallet for the faucet

2. **Rate Limiting**
   - Adjust rate limits based on your token supply
   - Monitor for abuse patterns
   - Consider adding CAPTCHA for additional protection

3. **Network Security**
   - Use HTTPS with reverse proxy (Nginx, Caddy)
   - Configure firewall rules appropriately
   - Keep dependencies updated

4. **Monitoring**
   - Set up alerts for low faucet balance
   - Monitor request patterns
   - Log suspicious activity

## Troubleshooting

### Faucet not sending tokens

1. Check if private key is configured:
\`\`\`bash
docker-compose logs | grep "WARNING"
\`\`\`

2. Verify wallet has sufficient balance:
\`\`\`bash
docker-compose logs | grep "balance"
\`\`\`

3. Check RPC connection:
\`\`\`bash
curl http://node-1.test.ipc.space:8545 -X POST -H "Content-Type: application/json" --data '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}'
\`\`\`

### Rate limit errors

Rate limits are per IP and per address. Wait for the rate limit window to expire, or adjust the configuration.

### MetaMask connection issues

1. Make sure MetaMask is installed
2. Check that you're on the correct network
3. Use the "Switch to IPC Testnet" button to add the network

### Docker build failures

1. Ensure Docker is running:
\`\`\`bash
docker info
\`\`\`

2. Check Docker Compose version:
\`\`\`bash
docker-compose --version
\`\`\`

3. Rebuild from scratch:
\`\`\`bash
docker-compose down
docker-compose build --no-cache
docker-compose up -d
\`\`\`

## Project Structure

\`\`\`
faucet/
├── frontend/               # Vue 3 frontend
│   ├── src/
│   │   ├── App.vue        # Main application component
│   │   ├── main.js        # Entry point
│   │   └── style.css      # Global styles (Tailwind)
│   ├── public/
│   │   └── favicon.svg    # Faucet icon
│   ├── index.html
│   ├── package.json
│   ├── vite.config.js
│   └── tailwind.config.js
├── backend/               # Express backend
│   ├── src/
│   │   └── index.js      # Main server file
│   └── package.json
├── Dockerfile            # Multi-stage Docker build
├── docker-compose.yml    # Docker Compose configuration
├── .dockerignore
├── .gitignore
├── package.json          # Root package file
└── README.md            # This file
\`\`\`

## API Reference

### GET `/api/health`

Health check endpoint.

**Response:**
\`\`\`json
{
  "status": "ok",
  "configured": true,
  "network": "http://node-1.test.ipc.space:8545"
}
\`\`\`

### GET `/api/config`

Get faucet configuration.

**Response:**
\`\`\`json
{
  "amount": "1",
  "rateLimit": "1 request per 24 hours per address",
  "network": "http://node-1.test.ipc.space:8545"
}
\`\`\`

### POST `/api/request`

Request tFIL tokens.

**Request Body:**
\`\`\`json
{
  "address": "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb"
}
\`\`\`

**Success Response:**
\`\`\`json
{
  "success": true,
  "txHash": "0x123abc...",
  "amount": "1",
  "blockNumber": 12345
}
\`\`\`

**Error Response:**
\`\`\`json
{
  "success": false,
  "error": "Rate limit exceeded"
}
\`\`\`

## Contributing

Contributions are welcome! Please follow these guidelines:

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Test thoroughly
5. Submit a pull request

## License

This project is part of the IPC (InterPlanetary Consensus) project.

## Support

- Documentation: https://docs.ipc.space
- Issues: https://github.com/consensus-shipyard/ipc/issues
- Community: [IPC Discord/Forum]

## Changelog

### v1.0.0 (2024-10-31)
- Initial release
- Vue 3 frontend with Tailwind CSS
- Express backend with rate limiting
- MetaMask integration
- Network switcher
- Docker support
- GCP deployment ready

