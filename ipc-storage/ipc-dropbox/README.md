# IPC Decentralized Dropbox

A Dropbox-like web application for storing and managing files on the IPC network.

## Prerequisites

- Node.js 18+
- MetaMask browser extension
- Running IPC network services:
  - Gateway (port 8080)
  - Node (port 8081)
  - Tendermint RPC (port 26657)
  - Ethereum RPC (port 8545)

## Setup

1. Install dependencies:

```bash
npm install
```

2. Copy the environment file and configure:

```bash
cp .env.example .env
```

Edit `.env` with your service URLs if different from defaults.

3. Start the development server:

```bash
npm run dev
```

4. Open http://localhost:3000 in your browser

## Configuration

The following environment variables can be configured:

| Variable | Default | Description |
|----------|---------|-------------|
| `VITE_TENDERMINT_RPC` | `http://localhost:26657` | Tendermint RPC endpoint |
| `VITE_OBJECTS_LISTEN_ADDR` | `http://localhost:8080` | Gateway objects API |
| `VITE_NODE_OPERATION_OBJECT_API` | `http://localhost:8081` | Node operation API |
| `VITE_ETH_RPC` | `http://localhost:8545` | Ethereum RPC endpoint |
| `VITE_BLOBS_ACTOR` | `0x6d342...` | Blobs actor contract address |
| `VITE_ADM_ACTOR` | `0x7caec...` | ADM actor contract address |

## Usage Flow

1. **Connect Wallet**: Click "Connect MetaMask" to connect your wallet. The app will attempt to switch to the IPC network automatically.

2. **Buy Credit**: If you don't have credit, purchase some using FIL. This is required for storage.

3. **Create Bucket**: Create a storage bucket to hold your files. Each bucket is an on-chain smart contract.

4. **Upload Files**: Once you have credit and a bucket, you can:
   - Upload files using the "Upload File" button
   - Create folders for organization
   - Navigate through folders using breadcrumbs

5. **Download Files**: Click the "Download" button next to any file to retrieve it.

## Features

- MetaMask wallet integration
- Credit balance display and purchase
- Bucket creation and management
- File upload to gateway + on-chain registration
- Folder-based navigation (S3-style)
- File download from node

## Tech Stack

- React 18
- TypeScript
- Vite
- ethers.js v6

## Building for Production

```bash
npm run build
```

The built files will be in the `dist` directory.
