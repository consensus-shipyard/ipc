# Storage CLI Quickstart

This guide walks you through testing the IPC decentralized storage CLI on the test subnet.

## Prerequisites

Build the CLI (macOS, targeting the local machine):

```bash
cargo build --release -p ipc-cli --features ipc-storage
```

Make sure you have an IPC wallet set up (`~/.ipc/config.toml` with an EVM key).
If not, create one:

```bash
./target/release/ipc-cli wallet new --wallet-type evm
./target/release/ipc-cli wallet set-default --wallet-type evm --address <0xYOUR_ADDRESS>
```

## Step 1: Fund your account on the storage subnet

Send tokens from the parent chain (calibnet) into the storage subnet:

```bash
./target/release/ipc-cli cross-msg fund \
  --subnet "/r314159/t410fg32br4ow4kdhp3wssi6c4xumsdpjzhw6y4ydbxq" \
  --from 0xYOUR_ADDRESS \
  --to 0xYOUR_ADDRESS \
  60
```

Wait for the top-down message to be finalized (up to ~3 minutes), then verify your balance:

```bash
curl http://136.115.12.207:8545 \
  -H 'content-type: application/json' \
  --data '{"jsonrpc":"2.0","method":"eth_getBalance","params":["0xYOUR_ADDRESS","latest"],"id":1}'
```

A non-zero `result` means your account is funded.

## Step 2: Initialize the storage client

```bash
./target/release/ipc-cli storage client init \
  --rpc-url http://136.115.12.207:26657 \
  --gateway-url http://136.115.12.207:8080
```

This creates `~/.ipc/storage/client/config.yaml`. The CLI uses your default EVM wallet key for signing transactions.

## Step 3: Run the test suite

```bash
./test.sh
```

The script automatically:
1. Buys storage credit (0.1 FIL)
2. Creates a bucket (or reuses an existing one)
3. Tests all 18 operations: upload, list, stat, cat, download, recursive upload/download, move, delete

Phase 1 (steps 2-12) tests read/write operations immediately.
Phase 2 (steps 13-18) waits 90 seconds for blob finalization, then tests move and delete.

## Manual commands

Once initialized, you can use any storage command directly:

```bash
# Buy storage credit
./target/release/ipc-cli storage client credit buy 0.1

# Create a bucket
./target/release/ipc-cli storage client bucket create

# List buckets
./target/release/ipc-cli storage client bucket list

# Upload a file
./target/release/ipc-cli storage client cp /path/to/file.txt ipc://BUCKET/key.txt --gateway http://136.115.12.207:8080

# Upload a directory
./target/release/ipc-cli storage client cp -r /path/to/dir ipc://BUCKET/prefix --gateway http://136.115.12.207:8080

# List objects
./target/release/ipc-cli storage client ls ipc://BUCKET/

# Get object metadata
./target/release/ipc-cli storage client stat ipc://BUCKET/key.txt

# Read file contents
./target/release/ipc-cli storage client cat ipc://BUCKET/key.txt --gateway http://136.115.12.207:8080

# Download a file
./target/release/ipc-cli storage client cp ipc://BUCKET/key.txt /local/path.txt --gateway http://136.115.12.207:8080

# Move/rename
./target/release/ipc-cli storage client mv ipc://BUCKET/old.txt ipc://BUCKET/new.txt --gateway http://136.115.12.207:8080

# Delete
./target/release/ipc-cli storage client rm --force ipc://BUCKET/key.txt

# Delete recursively
./target/release/ipc-cli storage client rm -r --force ipc://BUCKET/prefix/

# Check credit info
./target/release/ipc-cli storage client credit info
```

Replace `BUCKET` with your bucket address (e.g. `t0123`).

## Notes

- **Blob finalization**: After uploading, blobs take ~10-15 seconds to be finalized by the storage node. Until finalized, delete and move operations will fail with "blob pending finalization".
- **Gateway URL**: The `--gateway` flag is required for commands that transfer data (cp, cat, mv). Read-only commands (ls, stat, credit info, bucket list) only need the RPC.
- **Overwrite**: Use `--overwrite` with `cp` to replace an existing object.
