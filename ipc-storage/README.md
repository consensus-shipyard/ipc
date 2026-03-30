# Bucket Storage Guide

## 1. Build IPC

By default, ipc-storage is not enabled. Build with the `ipc-storage` feature:

```bash
cargo build --release -p fendermint_app --features ipc-storage
cargo build --release -p ipc-cli --features ipc-storage
cargo build --release -p ipc-decentralized-storage --bin gateway --bin node
```

Set up your IPC chain as per normal.

## 2. Initialize Storage Config

```bash
./target/release/ipc-cli storage init
```

This generates `~/.ipc/storage.yaml` with defaults. Update it if needed:

```yaml
# Key fields you may want to adjust:
secret-key-file: ./test-network/keys/alice.sk   # your funded key
gateway-url: http://127.0.0.1:8080              # gateway address
tendermint-rpc-url: http://127.0.0.1:26657
eth-rpc-url: http://127.0.0.1:8545
```

## 3. Start Gateway and Node Operator

```bash
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

./target/release/gateway --bls-key-file $BLS_KEY_FILE --secret-key-file $SECRET_KEY_FILE --iroh-path ./iroh_gateway --objects-listen-addr 127.0.0.1:8080
```

## 4. Buy Credits

```bash
./target/release/ipc-cli storage credit buy 0.1
```

## 5. Create a Bucket

```bash
./target/release/ipc-cli storage bucket create
```

This prints the bucket addresses (Actor ID, EVM, robust). Export the address for later use:

```bash
export BUCKET_ADDR=t065  # use the actor ID from the output
```

## 6. Upload Files

```bash
# Upload a single file
echo "Hello from bucket storage!" > myfile.txt
./target/release/ipc-cli storage cp ./myfile.txt "ipc://${BUCKET_ADDR}/documents/myfile.txt"
```

## 7. Query Objects

```bash
# List all objects in a bucket
./target/release/ipc-cli storage ls "ipc://${BUCKET_ADDR}/"

# Get object metadata
./target/release/ipc-cli storage stat "ipc://${BUCKET_ADDR}/documents/myfile.txt"
```

## 8. Read File Contents

```bash
./target/release/ipc-cli storage cat "ipc://${BUCKET_ADDR}/documents/myfile.txt"
```

## 9. Download Files

```bash
# Download a single file
./target/release/ipc-cli storage cp "ipc://${BUCKET_ADDR}/documents/myfile.txt" ./downloaded.txt
```
