# IPC Faucet

A faucet application for distributing test tokens on IPC subnets.

## Setup

### 1. Configure Environment Variables

Copy the example environment file and edit it with your actual values:

```bash
cp .env.example .env
```

Then edit `.env` and configure:

- **PRIVATE_KEY**: Private key for the faucet wallet that will distribute funds
  - ⚠️ **SECURITY**: Never commit this file or share your private key
  - Make sure the wallet has sufficient funds to distribute

- **RPC_URL**: RPC endpoint for your IPC subnet
  - Local development: `http://localhost:8545`
  - Test network: `http://node-1.test.ipc.space:8545`
  - Production: Your subnet's RPC endpoint

- **FAUCET_AMOUNT**: Amount to send per request (in native token units)
  - Default: `10`

- **RATE_LIMIT_WINDOW**: Time window for rate limiting in milliseconds
  - Default: `86400000` (24 hours)

- **RATE_LIMIT_MAX**: Maximum requests per address within the rate limit window
  - Default: `3`

### 2. Install Dependencies

```bash
npm install
```

### 3. Run the Faucet

```bash
npm start
```

## Security Notes

- ⚠️ **The `.env` file is in `.gitignore` and should NEVER be committed to version control**
- Use a dedicated wallet for the faucet with limited funds
- Configure appropriate rate limits to prevent abuse
- Monitor the faucet wallet balance regularly
- For production use, consider additional security measures like IP-based rate limiting

## Development

The faucet consists of:
- **backend/**: Node.js backend service
- **frontend/**: Web frontend for requesting funds
- **scripts/**: Utility scripts for maintenance

## Troubleshooting

### Faucet wallet has insufficient funds
Top up the wallet associated with the `PRIVATE_KEY` in your `.env` file.

### Rate limit errors
Users are limited to `RATE_LIMIT_MAX` requests per `RATE_LIMIT_WINDOW`. Wait or adjust limits in `.env`.

### Connection errors
Verify the `RPC_URL` in `.env` is correct and the subnet is running.
