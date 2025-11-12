# Recall Storage Testing Guide (POC Mode)

## Key Test Assumptions

1. **Single validator node** - This guide is designed for a single validator setup, but should work for multi-node configurations
2. **Validator has genesis balance** - The validator key is used in `USER_SK` and `USER_ADDR`, and must have initial tokens from genesis
3. **Subnet setup from genesis** - The subnet must be configured from genesis to deploy Recall contracts (particularly the Blobs Actor)
4. **IPC subnet configuration** - Both Fendermint config and IPC config must have proper subnet configuration
5. **Fendermint Recall settings configured** - The following must be properly configured in Fendermint config (fendermint will not start if missing):
   - Objects service settings (iroh path, resolver RPC address)
   - Recall actor settings
   - Validator key configuration
   - Iroh configuration (storage path, RPC endpoints)
  can refer to fendermint default config file.
6. **Required tools installed** - Assumes `cometbft`, `fendermint`, `cast` (Foundry), `jq`, and `python3` are installed and in PATH
7. **Blobs Actor pre-deployed** - The `BLOBS_ACTOR` address must be available (deployed during genesis or migration)
8. **Local development environment** - All services run on localhost with default ports (26657, 8080, 8545, 4444)

### Configuration

Set environment variables:

```bash
export TENDERMINT_RPC=http://localhost:26657
export OBJECTS_API=http://localhost:8080
export BLOBS_ACTOR=0x6d342defae60f6402aee1f804653bbae4e66ae46
```

---

## 1. Start Services

### Start Fendermint Node

```bash
# Terminal 1: Start CometBFT
cometbft start
# Terminal 2: Start Fendermint
fendermint run
# Terminal 3: Start ETH
fendermint eth run
# Terminal 4: Object service
fendermint objects run --iroh-path `pwd`/iroh --iroh-resolver-rpc-addr 127.0.0.1:4444
```

---

## 3. Buy Storage Credits

Credits are required to store blobs. Purchase credits with tokens:

```bash
# Export private key as hex (with or without 0x prefix)
export USER_SK=<YOUR_PRIVATE_KEY_HEX>
# Export your Ethereum address
export USER_ADDR=<YOUR_ETH_ADDRESS>
# Buy 1 FIL worth of credits
cast send $BLOBS_ACTOR "buyCredit()" \
  --value 0.1ether \
  --private-key $USER_SK \
  --rpc-url http://localhost:8545

# Check your account
cast call $BLOBS_ACTOR "getAccount(address)" $USER_ADDR \
  --rpc-url http://localhost:8545

# it should have data
```
---

## 4. Upload a Blob

Use the HTTP API to upload files to Iroh:

```bash
# Create a test file
echo "Hello, Recall Storage!" > test.txt

BLOB_SIZE=$(stat -f%z test.txt 2>/dev/null || stat -c%s test.txt)
# Upload to Iroh via HTTP API
UPLOAD_RESPONSE=$(curl -s -X POST $OBJECTS_API/v1/objects \
  -F "size=${BLOB_SIZE}" \
  -F "data=@test.txt")

echo $UPLOAD_RESPONSE | jq '.'

# Extract the blob hashes (in base32 format)
# IMPORTANT: Use hash (hash sequence) for addBlob - validators need to resolve the hash sequence
BLOB_HASH_B32=$(echo $UPLOAD_RESPONSE | jq -r '.hash')
METADATA_HASH_B32=$(echo $UPLOAD_RESPONSE | jq -r '.metadata_hash // .metadataHash')
NODE_ID_BASE32=$(curl -s $OBJECTS_API/v1/node | jq -r '.node_id')

# Convert base32 hashes to hex format for Solidity bytes32
export BLOB_HASH=$(python3 -c "
import base64
h = '$BLOB_HASH_B32'.upper()
# Add padding if needed (base32 requires length to be multiple of 8)
padding = (8 - len(h) % 8) % 8
h = h + '=' * padding
decoded = base64.b32decode(h)
if len(decoded) > 32:
    decoded = decoded[:32]
elif len(decoded) < 32:
    decoded = decoded + b'\x00' * (32 - len(decoded))
print('0x' + decoded.hex())
")

export METADATA_HASH=$(python3 -c "
import base64
h = '$METADATA_HASH_B32'.upper()
# Add padding if needed (base32 requires length to be multiple of 8)
padding = (8 - len(h) % 8) % 8
h = h + '=' * padding
decoded = base64.b32decode(h)
if len(decoded) > 32:
    decoded = decoded[:32]
elif len(decoded) < 32:
    decoded = decoded + b'\x00' * (32 - len(decoded))
print('0x' + decoded.hex())
")

echo "Blob Hash (base32): $BLOB_HASH_B32"
echo "Blob Hash (hex): $BLOB_HASH"
echo "Metadata Hash (base32): $METADATA_HASH_B32"
echo "Metadata Hash (hex): $METADATA_HASH"
echo "Source Node: $NODE_ID_BASE32"
```
---

## 5. Register Blob On-Chain

Register the blob with the Blobs Actor:

```bash
# Add 0x prefix to the node ID (already in hex format)
SOURCE_NODE="0x$NODE_ID_BASE32"
echo "Source Node (hex): $SOURCE_NODE"

# Add blob subscription
TX_RECEIPT=$(cast send $BLOBS_ACTOR "addBlob(address,bytes32,bytes32,bytes32,string,uint64,uint64)" \
  "0x0000000000000000000000000000000000000000" \
  $SOURCE_NODE \
  $BLOB_HASH \
  $METADATA_HASH \
  "" \
  $BLOB_SIZE \
  86400 \
  --private-key $USER_SK \
  --rpc-url http://localhost:8545 \
  --json)

# Wait for transaction to be mined
sleep 5

```bash
# Check blob status
BLOB_INFO=$(cast call $BLOBS_ACTOR "getBlob(bytes32)" $BLOB_HASH \
  --rpc-url http://localhost:8545)

cast abi-decode "getBlob(bytes32)((uint64,bytes32,(string,uint64)[],uint8))" $BLOB_INFO

# Status should now be 2 (Resolved) after some time
```

---

## 6. Download the Blob

Download via HTTP API:

```bash
# Download the blob
curl $OBJECTS_API/v1/blobs/${BLOB_HASH#0x}
# You should see the original file
```
