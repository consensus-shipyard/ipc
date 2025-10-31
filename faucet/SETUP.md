# Quick Setup Guide

This guide will help you get your IPC tFIL faucet up and running in minutes.

## Step 1: Prepare Your Wallet

### Option A: Create a New Wallet (Recommended)

\`\`\`bash
# Generate a new wallet using Node.js
node -e "const ethers = require('ethers'); const wallet = ethers.Wallet.createRandom(); console.log('Address:', wallet.address); console.log('Private Key:', wallet.privateKey);"
\`\`\`

**Save the output securely!**

Example output:
\`\`\`
Address: 0x1234567890abcdef1234567890abcdef12345678
Private Key: 0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890
\`\`\`

### Option B: Use Existing Wallet

Export your private key from MetaMask:
1. Open MetaMask
2. Click on the account menu (three dots)
3. Account Details → Export Private Key
4. Enter your password
5. Copy the private key

### Fund Your Wallet

Transfer tFIL to your faucet wallet address. Calculate how much you need:

\`\`\`
Amount needed = (Expected requests × Amount per request) + Gas buffer
Example: (1000 requests × 1 tFIL) + 10 tFIL gas = 1010 tFIL
\`\`\`

## Step 2: Configure the Faucet

Create a `.env` file in the `faucet/` directory:

\`\`\`bash
cd faucet
nano .env
\`\`\`

Add the following configuration:

\`\`\`bash
# YOUR FAUCET WALLET PRIVATE KEY (keep this secret!)
PRIVATE_KEY=0xYOUR_PRIVATE_KEY_HERE

# IPC Testnet RPC
RPC_URL=http://node-1.test.ipc.space:8545

# Amount to distribute per request (in tFIL)
FAUCET_AMOUNT=1

# Rate limiting: 1 request per 24 hours
RATE_LIMIT_WINDOW=86400000
RATE_LIMIT_MAX=1

# Server configuration
PORT=3001
ENABLE_CORS=false
SERVE_STATIC=true
\`\`\`

**Save and exit** (Ctrl+X, then Y, then Enter)

## Step 3: Deploy with Docker

### Install Docker (if not already installed)

\`\`\`bash
# Install Docker
curl -fsSL https://get.docker.com -o get-docker.sh
sudo sh get-docker.sh

# Install Docker Compose
sudo curl -L "https://github.com/docker/compose/releases/latest/download/docker-compose-$(uname -s)-$(uname -m)" -o /usr/local/bin/docker-compose
sudo chmod +x /usr/local/bin/docker-compose

# Add your user to docker group (to run without sudo)
sudo usermod -aG docker $USER

# Log out and back in for changes to take effect
exit
\`\`\`

### Build and Run

\`\`\`bash
# Navigate to faucet directory
cd /path/to/ipc/faucet

# Build and start the faucet
docker-compose up -d

# Check if it's running
docker-compose ps

# View logs
docker-compose logs -f
\`\`\`

You should see output like:
\`\`\`
✅ Wallet initialized
   Address: 0x1234...
💰 Faucet balance: 1000.0 tFIL
   Can serve ~1000 requests
✅ Server running on port 3001
\`\`\`

## Step 4: Configure Firewall (GCP)

### Using gcloud CLI:

\`\`\`bash
gcloud compute firewall-rules create allow-ipc-faucet \
  --allow tcp:3001 \
  --source-ranges 0.0.0.0/0 \
  --description "Allow access to IPC tFIL faucet"
\`\`\`

### Using GCP Console:

1. Go to VPC Network → Firewall
2. Click "CREATE FIREWALL RULE"
3. Name: `allow-ipc-faucet`
4. Direction: Ingress
5. Targets: All instances in the network
6. Source IP ranges: `0.0.0.0/0`
7. Protocols and ports: tcp:3001
8. Click CREATE

## Step 5: Access Your Faucet

### Find Your External IP:

\`\`\`bash
# On GCP VM
curl -H "Metadata-Flavor: Google" http://metadata.google.internal/computeMetadata/v1/instance/network-interfaces/0/access-configs/0/external-ip
\`\`\`

Or check in GCP Console: Compute Engine → VM Instances

### Access the faucet:

Open your browser and go to:
\`\`\`
http://YOUR_EXTERNAL_IP:3001
\`\`\`

## Step 6: Test the Faucet

1. **Open the faucet URL in your browser**
2. **Click "Connect MetaMask"**
3. **Click "Switch to IPC Testnet"** (if not already connected)
4. **Click "Request 1 tFIL"**
5. **Wait for confirmation**

You should see a success message with a transaction hash!

## Step 7: Set Up Monitoring (Optional)

### Set up automatic restarts:

Docker Compose is already configured with `restart: unless-stopped`, so the faucet will automatically restart if it crashes or after server reboots.

### Monitor balance:

Create a simple monitoring script:

\`\`\`bash
nano /home/$USER/check-faucet-balance.sh
\`\`\`

Add:
\`\`\`bash
#!/bin/bash
docker-compose -f /path/to/ipc/faucet/docker-compose.yml logs | grep "Faucet balance" | tail -1
\`\`\`

Make executable:
\`\`\`bash
chmod +x /home/$USER/check-faucet-balance.sh
\`\`\`

### Set up a cron job to check balance daily:

\`\`\`bash
crontab -e
\`\`\`

Add:
\`\`\`
0 9 * * * /home/$USER/check-faucet-balance.sh >> /home/$USER/faucet-balance.log 2>&1
\`\`\`

## Useful Commands

### Check faucet status:
\`\`\`bash
docker-compose ps
\`\`\`

### View logs:
\`\`\`bash
docker-compose logs -f
\`\`\`

### Restart faucet:
\`\`\`bash
docker-compose restart
\`\`\`

### Stop faucet:
\`\`\`bash
docker-compose down
\`\`\`

### Update faucet:
\`\`\`bash
git pull
docker-compose down
docker-compose build --no-cache
docker-compose up -d
\`\`\`

### Check faucet health:
\`\`\`bash
curl http://localhost:3001/api/health
\`\`\`

## Troubleshooting

### Faucet not accessible from browser:

1. Check if Docker container is running:
   \`\`\`bash
   docker-compose ps
   \`\`\`

2. Check firewall rules:
   \`\`\`bash
   gcloud compute firewall-rules list | grep faucet
   \`\`\`

3. Test locally on the VM:
   \`\`\`bash
   curl http://localhost:3001/api/health
   \`\`\`

### Faucet not sending tokens:

1. Check balance:
   \`\`\`bash
   docker-compose logs | grep balance
   \`\`\`

2. Verify private key is set:
   \`\`\`bash
   docker-compose logs | grep "Wallet initialized"
   \`\`\`

3. Test RPC connection:
   \`\`\`bash
   curl -X POST http://node-1.test.ipc.space:8545 \
     -H "Content-Type: application/json" \
     -d '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}'
   \`\`\`

### Rate limit issues:

Rate limits are tracked in-memory. If you restart the container, rate limits reset. To modify rate limits, update `.env` and restart:

\`\`\`bash
docker-compose down
docker-compose up -d
\`\`\`

## Security Checklist

- [ ] Private key is stored in `.env` (not committed to git)
- [ ] `.env` file has restrictive permissions: `chmod 600 .env`
- [ ] Firewall is configured properly
- [ ] Faucet wallet is separate from other wallets
- [ ] Balance monitoring is set up
- [ ] Regular backups of configuration
- [ ] Docker and system packages are up to date

## Next Steps

- Set up HTTPS with a reverse proxy (Nginx or Caddy)
- Configure a domain name for easier access
- Set up monitoring and alerting
- Consider adding CAPTCHA for additional abuse prevention

## Need Help?

- Check the main README.md for detailed documentation
- Review logs: `docker-compose logs -f`
- Visit IPC documentation: https://docs.ipc.space
- Report issues on GitHub

---

**Your faucet should now be running! 🎉**

Access it at: `http://YOUR_EXTERNAL_IP:3001`

