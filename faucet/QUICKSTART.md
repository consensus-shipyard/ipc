# 🚀 Quick Start Guide

Get your IPC tFIL faucet running in 5 minutes!

## For Local Development

```bash
# 1. Install dependencies
cd faucet
make install

# 2. Generate a wallet
make generate-wallet

# 3. Create .env file
cp env-template.txt .env
nano .env  # Add your PRIVATE_KEY

# 4. Fund your wallet with tFIL
# (Transfer tFIL to the address from step 2)

# 5. Start development servers
make dev

# Visit http://localhost:3000
```

## For Production (Docker)

```bash
# 1. Create .env file
cd faucet
nano .env
```

Add this:
```env
PRIVATE_KEY=0xYOUR_PRIVATE_KEY_HERE
RPC_URL=http://node-1.test.ipc.space:8545
FAUCET_AMOUNT=1
RATE_LIMIT_WINDOW=86400000
RATE_LIMIT_MAX=1
```

```bash
# 2. Start with Docker
make docker-up

# 3. Check logs
make docker-logs

# Visit http://localhost:3001
```

## For GCP Deployment

```bash
# 1. Create VM
gcloud compute instances create ipc-faucet \
  --zone=us-central1-a \
  --machine-type=e2-small \
  --image-family=ubuntu-2204-lts \
  --image-project=ubuntu-os-cloud

# 2. SSH in
gcloud compute ssh ipc-faucet --zone=us-central1-a

# 3. Install Docker
curl -fsSL https://get.docker.com | sudo sh
sudo usermod -aG docker $USER

# 4. Clone and configure
git clone https://github.com/consensus-shipyard/ipc.git
cd ipc/faucet
nano .env  # Add configuration

# 5. Start faucet
docker-compose up -d

# 6. Configure firewall
gcloud compute firewall-rules create allow-ipc-faucet \
  --allow tcp:3001 \
  --source-ranges 0.0.0.0/0
```

## Helpful Commands

```bash
make help              # Show all commands
make check-balance     # Check wallet balance
make docker-logs       # View logs
make docker-restart    # Restart faucet
make status           # Show faucet status
```

## Need Help?

- 📖 Full docs: See [README.md](README.md)
- 🛠️ Setup guide: See [SETUP.md](SETUP.md)
- ☁️ GCP deployment: See [DEPLOY.md](DEPLOY.md)

---

**Made with ❤️ for the IPC community**

