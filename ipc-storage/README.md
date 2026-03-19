# Bucket Storage Guide (Path-Based Access)

## Configuration

```bash
# From RECALL_RUN.md
export TENDERMINT_RPC=http://localhost:26657
export OBJECTS_LISTEN_ADDR=http://localhost:8080
export NODE_OPERATION_OBJECT_API=http://localhost:8081
export ETH_RPC=http://localhost:8545
export BLOBS_ACTOR=0x6d342defae60f6402aee1f804653bbae4e66ae46
export ADM_ACTOR=0x7caec36fc8a3a867ca5b80c6acb5e5871d05aa28

# Your credentials
export USER_SK=<YOUR_PRIVATE_KEY_HEX>
```

## 1. Build IPC
By default, ipc-storage is not enabled, build with `ipc-storage` feature to enable it.
```bash
cargo build --release -p fendermint_app --features ipc-storage
```
Setup your ipc chain as per normal.

## 2. Start Gateway and Node Operator
```bash
cargo build --release -p ipc-decentralized-storage --bin gateway --bin node

# prepare to start node
export FM_NETWORK=test
# validator bls key file in hex format
export BLS_KEY_FILE=./test-network/bls_key.hex
# fendermint secret key file
export SECRET_KEY_FILE=./test-network/keys/alice.sk

# register as a storage node operator
./target/release/node register-operator --bls-key-file $BLS_KEY_FILE --secret-key-file $SECRET_KEY_FILE --operator-rpc-url $NODE_OPERATION_OBJECT_API

# start the node
./target/release/node run \
  --secret-key-file ./test-network/bls_key.hex \
  --iroh-path ./iroh_node \
  --iroh-v4-addr 0.0.0.0:11204 \
  --rpc-url http://localhost:26657 \
  --batch-size 10 \
  --poll-interval-secs 5 \
  --max-concurrent-downloads 10 \
  --rpc-bind-addr 127.0.0.1:8081

./target/release/gateway --bls-key-file $BLS_KEY_FILE --secret-key-file $SECRET_KEY_FILE --iroh-path ./iroh_gateway --objects-listen-addr 0.0.0.0:8080

```

Notes:
- `register-operator` now supports delegated (`t410...`) operator keys through the blobs actor `InvokeContract` facade path.
- Operator query methods are also available through the same facade path: `getOperatorInfo(address)` and `getActiveOperators()`.
- Storage node and gateway use delegated (`t410...`) sender path for on-chain interactions; fund delegated operator address before running.

## 3. Launch ipc-dropbox
Launch `ipc-dropbox` in `ipc-storage/ipc-dropbox` with `npm run dev`.

Alternatively, run the below bash script to go through the steps one by one.

## 3. Create a Bucket

First, create a bucket via the ADM (Actor Deployment Manager):

```bash
# Buy 1 FIL worth of credits
cast send $BLOBS_ACTOR "buyCredit()" \
  --value 0.1ether \
  --private-key $USER_SK \
  --rpc-url http://localhost:8545

# Create a new bucket (caller becomes owner)
TX_RESULT=$(cast send $ADM_ACTOR "createBucket()" \
  --private-key $USER_SK \
  --rpc-url $ETH_RPC \
  --json)

echo $TX_RESULT | jq '.'

# Extract bucket address from MachineInitialized event
# Event signature: MachineInitialized(uint8 indexed kind, address machineAddress)
BUCKET_ADDR=$(echo $TX_RESULT | jq -r '.logs[] | select(.topics[0] == "0x8f7252642373d5f0b89a0c5cd9cd242e5cd5bb1a36aec623756e4f52a8c1ea6e") | .data' | cut -c27-66)
BUCKET_ADDR="0x$BUCKET_ADDR"

echo "Bucket created at: $BUCKET_ADDR"
export BUCKET_ADDR
```

## 4. Upload and Register an Object
```bash
# Create a test file
echo "Hello from bucket storage!" > myfile.txt

# Get file size
BLOB_SIZE=$(stat -f%z myfile.txt 2>/dev/null || stat -c%s myfile.txt)

# Upload to Iroh
UPLOAD_RESPONSE=$(curl -s -X POST $OBJECTS_API/v1/objects \
  -F "size=${BLOB_SIZE}" \
  -F "data=@myfile.txt")

echo $UPLOAD_RESPONSE | jq '.'

# Extract hashes
BLOB_HASH_B32=$(echo $UPLOAD_RESPONSE | jq -r '.hash')
METADATA_HASH_B32=$(echo $UPLOAD_RESPONSE | jq -r '.metadata_hash // .metadataHash')
NODE_ID_BASE32=$(curl -s $OBJECTS_API/v1/node | jq -r '.node_id')

# Convert to hex (same as RECALL_RUN.md)
export BLOB_HASH=$(python3 -c "
import base64
h = '$BLOB_HASH_B32'.upper()
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
padding = (8 - len(h) % 8) % 8
h = h + '=' * padding
decoded = base64.b32decode(h)
if len(decoded) > 32:
    decoded = decoded[:32]
elif len(decoded) < 32:
    decoded = decoded + b'\x00' * (32 - len(decoded))
print('0x' + decoded.hex())
")

export SOURCE_NODE="0x$NODE_ID_BASE32"

echo "Blob Hash: $BLOB_HASH"
echo "Metadata Hash: $METADATA_HASH"
echo "Source Node: $SOURCE_NODE"
```

### Register object in bucket with a path

```bash
# Add object with a path-based key
# Signature: addObject(bytes32 source, string key, bytes32 hash, bytes32 recoveryHash, uint64 size)
cast send $BUCKET_ADDR "addObject(bytes32,string,bytes32,bytes32,uint64)" \
  $SOURCE_NODE \
  "documents/myfile.txt" \
  $BLOB_HASH \
  $METADATA_HASH \
  $BLOB_SIZE \
  --private-key $USER_SK \
  --rpc-url $ETH_RPC
```

## 5. Query Objects

### Get a single object by path

```bash
# Get object by exact path
# Returns: ObjectValue(bytes32 blobHash, bytes32 recoveryHash, uint64 size, uint64 expiry, (string,string)[] metadata)
cast call $BUCKET_ADDR "getObject(string)((bytes32,bytes32,uint64,uint64,(string,string)[]))" "documents/myfile.txt" --rpc-url $ETH_RPC
```

### List all objects (no filter)

```bash
# List all objects in bucket
cast call $BUCKET_ADDR "queryObjects()(((string,(bytes32,uint64,uint64,(string,string)[]))[],string[],string))" \
  --rpc-url $ETH_RPC
```

### List with prefix (folder-like)

```bash
# List everything under "documents/"
cast call $BUCKET_ADDR "queryObjects(string)(((string,(bytes32,uint64,uint64,(string,string)[]))[],string[],string))" "documents/" --rpc-url $ETH_RPC
```

### List with delimiter (S3-style folder simulation)

```bash
# List top-level "folders" and files
# Returns: Query((string,ObjectState)[] objects, string[] commonPrefixes, string nextKey)
# Where ObjectState = (bytes32 blobHash, uint64 size, uint64 expiry, (string,string)[] metadata)
cast call $BUCKET_ADDR "queryObjects(string,string)(((string,(bytes32,uint64,uint64,(string,string)[]))[],string[],string))" "" "/" \
  --rpc-url $ETH_RPC

# Example response:
# ([], ["documents/", "images/"], "")
#  ^objects at root   ^"folders"   ^nextKey (empty = no more pages)

# Extract blob hash from first object:
# BLOB_HASH=$(cast call ... | jq -r '.[0][0][1][0]')

# List contents of "documents/" folder
cast call $BUCKET_ADDR "queryObjects(string,string)(((string,(bytes32,uint64,uint64,(string,string)[]))[],string[],string))" "documents/" "/" \
  --rpc-url $ETH_RPC
```

### Paginated queries

```bash
# Query with pagination
# queryObjects(prefix, delimiter, startKey, limit)
cast call $BUCKET_ADDR "queryObjects(string,string,string,uint64)" \
  "documents/" \
  "/" \
  "" \
  100 \
  --rpc-url $ETH_RPC

# If nextKey is returned, use it for the next page
cast call $BUCKET_ADDR "queryObjects(string,string,string,uint64)" \
  "documents/" \
  "/" \
  "documents/page2start.txt" \
  100 \
  --rpc-url $ETH_RPC
```

---

## 6. Update Object Metadata

```bash
# Update metadata for an existing object
# Set value to empty string to delete a metadata key
cast send $BUCKET_ADDR "updateObjectMetadata(string,(string,string)[])" \
  "documents/myfile.txt" \
  '[("content-type","text/markdown"),("version","2")]' \
  --private-key $USER_SK \
  --rpc-url $ETH_RPC
```

---

## 7. Delete an Object

```bash
# Delete object by path
cast send $BUCKET_ADDR "deleteObject(string)" "documents/myfile.txt" \
  --private-key $USER_SK \
  --rpc-url $ETH_RPC
```

---

## 8. Download Content

Downloads still go through the Iroh/Objects API using the blob hash:

```bash
# First get the object to retrieve its blob hash
OBJECT_INFO=$(cast call $BUCKET_ADDR "getObject(string)" "documents/myfile.txt" \
  --rpc-url $ETH_RPC)

# Extract blob hash from response and download
# (The blob hash is the first bytes32 in the response)
curl $NODE_OPERATION_OBJECT_API/v1/blobs/${BLOB_HASH#0x}/content
```

---