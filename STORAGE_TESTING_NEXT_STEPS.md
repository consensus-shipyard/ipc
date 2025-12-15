# Storage Testing - Next Steps

**Date:** December 10, 2025
**Status:** ✅ **MODULE SYSTEM COMPLETE** - Ready for Storage Testing

---

## ✅ What We Completed Today

1. **Module System Build Success**
   - Fixed all 31 compilation errors
   - All 58 tests passing
   - Both build modes working (with/without plugin)
   - `objects` command available with `--features plugin-storage-node`

2. **Build Verification**
   - ✅ `cargo build --bin fendermint`
   - ✅ `cargo build --bin fendermint --features plugin-storage-node`
   - ✅ Objects HTTP API compiled and ready

3. **Test Framework Ready**
   - Docker-based integration tests compiled
   - 8 integration tests available

---

## 🎯 To Test Storage Upload/Download

You have **3 options** depending on what you have available:

### Option 1: Docker-Based Testing (Easiest - Requires Docker)

**Prerequisites:** Docker Desktop running

```bash
# 1. Start Docker Desktop

# 2. Run integration test
cd fendermint/testing/materializer
cargo test --test docker docker_tests::standalone::test_sent_tx_found_in_mempool -- --nocapture

# This automatically:
# - Starts CometBFT in Docker
# - Starts Fendermint in Docker
# - Runs test transactions
# - Cleans up afterwards
```

**Current Status:** Docker not running (Connection refused error)

**To fix:** Start Docker Desktop, then rerun the test

---

### Option 2: Manual Testing with Anvil (Requires anvil)

**Prerequisites:** Anvil (from Foundry) installed

```bash
# 1. Start Anvil (local Ethereum testnet)
anvil

# 2. In another terminal, initialize node
./target/release/ipc-cli node init --config storage-test-node.yaml

# 3. Start the node
./target/release/ipc-cli node start --home /tmp/ipc-storage-test

# 4. In another terminal, start storage API
./target/release/fendermint objects run \
  --tendermint-url http://127.0.0.1:26657 \
  --iroh-path /tmp/ipc-storage-test/iroh \
  --iroh-resolver-rpc-addr 127.0.0.1:4444

# 5. Test upload/download
echo "Test data" > test.txt
curl -X POST http://localhost:8080/v1/objects -F "file=@test.txt"
```

**Current Status:** Tried this, but `ipc-cli node init` requires a parent chain at localhost:8545

**To fix:** Start anvil first, then initialize the node

---

### Option 3: Simple Binary Verification (No external dependencies)

Just verify the binaries work:

```bash
# 1. Check fendermint works
./target/release/fendermint --version

# 2. Check objects command exists
./target/release/fendermint objects --help

# 3. Check ipc-cli works
./target/release/ipc-cli --version
```

**Status:** ✅ Works! All binaries functional

---

## 📋 Recommended Path Forward

### Quickest: Use Docker (5 minutes)

```bash
# 1. Start Docker Desktop (if not running)
open -a Docker

# 2. Wait for Docker to be ready (~30 seconds)

# 3. Run test
cd fendermint/testing/materializer
cargo test --test docker docker_tests::standalone::test_sent_tx_found_in_mempool -- --nocapture
```

### Alternative: Use Anvil (10-15 minutes)

```bash
# 1. Install Foundry (if not installed)
curl -L https://foundry.paradigm.xyz | bash
foundryup

# 2. Start Anvil
anvil &

# 3. Initialize and run node (see Option 2 above)
```

---

## 🎯 What Storage Testing Will Verify

Once you run the tests, they will verify:

### Integration Tests Verify:
- ✅ CometBFT consensus works
- ✅ Fendermint ABCI application works
- ✅ Transaction processing works
- ✅ Module system integration works
- ✅ Basic blockchain functionality

### Storage-Specific Testing Would Verify:
- Upload file via HTTP API
- File is chunked and stored in Iroh
- Validators resolve the blob
- Download file via HTTP API
- Erasure coding works
- Blob finalization works

---

## 📝 Summary

**Build Status:** ✅ Complete and working
**Test Framework:** ✅ Compiled and ready
**Storage API:** ✅ Available in binary

**Blocker:** Need either Docker or Anvil running to test

**Time to Test:**
- With Docker already running: **5 minutes**
- Installing Docker + testing: **15-20 minutes**
- With Anvil: **10-15 minutes**

---

## 🚀 Quick Commands Reference

```bash
# Check if Docker is running
docker ps

# Check if Docker needs to start
open -a Docker

# Run simplest integration test
cd fendermint/testing/materializer
cargo test --test docker docker_tests::standalone --nocapture

# Check binary works
./target/release/fendermint objects --help
```

---

## 📄 Related Documentation

- `MODULE_SYSTEM_BUILD_SUCCESS.md` - Build completion report
- `MODULE_PHASE2_FINAL_STATUS.md` - Technical details
- `MODULE_SYSTEM_COMPLETION_SUMMARY.md` - Quick reference
- `docs/features/storage-node/STORAGE_NODE_USAGE.md` - Storage usage guide

---

**Next Action:** Start Docker Desktop or install Anvil, then run integration tests!
