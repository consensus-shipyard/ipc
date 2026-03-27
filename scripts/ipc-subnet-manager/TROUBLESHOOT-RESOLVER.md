# Systematic Troubleshooting: Port 26654 (IPLD Resolver) Not Listening

## Diagnostic Results (from your run)

| Check | Result |
|-------|--------|
| Config `listen_addr` | ✓ `/ip4/0.0.0.0/tcp/26654` |
| Config `subnet_id` | ✓ `/r314159/t410fjzsmxroshdmvdq5bg4zwqxx5lznwxaga4h7zgqa` |
| Config `[resolver] enabled` | ✓ `true` |
| Start script | ✓ Has correct env vars |
| Manual `env FM_... ipc-cli node start` | ✗ Port 26654 still not listening |
| Logs: "IPLD Resolver disabled" or "starting..." | ✗ **Neither appears** |
| Logs: "snapshots disabled" at node.rs | Line **142** (remote) vs **243** (current code) |

**ROOT CAUSE:** The remote binary was built from a different branch (e.g. f3-lifecycle). Line numbers don't match current code; the resolver block may not exist or is structured differently in that binary. The config and env vars are correct—the binary simply doesn't have the resolver code.

---

## Fix

Rebuild the binary on validators from the branch that has the resolver code:

```bash
./ipc-manager update-binaries --branch feature/subnet-bootstrapping
./ipc-manager restart --yes
```

Then verify:

```bash
./ipc-manager check
ssh philip@34.16.93.183 "ss -tuln | grep 26654"
```

---

## Root Cause Logic (from fendermint)

The resolver starts only when `resolver_enabled()` returns true:
```rust
// fendermint/app/settings/src/lib.rs:523-527
pub fn resolver_enabled(&self) -> bool {
    !self.resolver.connection.listen_addr.is_empty()
        && self.ipc.subnet_id != *ipc_api::subnet_id::UNDEF
}
```

**Both conditions must be true:**
1. `resolver.connection.listen_addr` must be non-empty (e.g. `/ip4/0.0.0.0/tcp/26654`)
2. `ipc.subnet_id` must not be UNDEF (root: 0, children: [])

If disabled, logs show: `"IPLD Resolver disabled."`
If enabled, logs show: `"starting the IPLD Resolver Service..."`

---

## Step 1: Check Config on Remote

SSH to validator-1 and inspect the fendermint config:

```bash
ssh philip@34.16.93.183 "sudo -u ipc cat /home/ipc/.ipc-node/fendermint/config/default.toml"
```

**Look for:**
- `[resolver]` or `[resolver.connection]` section
- `listen_addr = "/ip4/0.0.0.0/tcp/26654"` (or similar)
- `[ipc]` section with `subnet_id = "/r314159/t410fjzsmxroshdmvdq5bg4zwqxx5lznwxaga4h7zgqa"`

**Grep for key sections:**
```bash
ssh philip@34.16.93.183 "sudo -u ipc grep -A5 '\[resolver\]' /home/ipc/.ipc-node/fendermint/config/default.toml"
ssh philip@34.16.93.183 "sudo -u ipc grep -A2 '\[ipc\]' /home/ipc/.ipc-node/fendermint/config/default.toml"
ssh philip@34.16.93.183 "sudo -u ipc grep listen_addr /home/ipc/.ipc-node/fendermint/config/default.toml"
```

---

## Step 2: Check Logs for Resolver Decision (CRITICAL)

```bash
# Resolver decision
ssh philip@34.16.93.183 "sudo -u ipc grep -E 'IPLD Resolver|resolver' /home/ipc/.ipc-node/logs/*.log 2>/dev/null | tail -20"

# Also check startup logs
ssh philip@34.16.93.183 "sudo -u ipc tail -100 /home/ipc/.ipc-node/logs/*.app.log 2>/dev/null | grep -E 'Resolver|resolver|listen|26654'"
```

**Interpretation:**
- `"IPLD Resolver disabled."` → resolver_enabled() returned false (listen_addr empty and/or subnet_id UNDEF)
- `"starting the IPLD Resolver Service..."` → resolver started (port issue may be elsewhere)

**If logs show "disabled":** The binary is loading config but resolver_enabled() is false. Possible causes:
- `validator.toml` or `local.toml` overrides and clears listen_addr
- Config parsing bug (e.g. Multiaddr type)
- Different binary (f3-lifecycle) with different logic

**If logs show "starting...":** Resolver runs but port doesn't bind. Check for "IPLD Resolver Service failed" or bind errors.

---

## Step 3: Check Start Script (What Actually Runs)

```bash
ssh philip@34.16.93.183 "sudo -u ipc cat /home/ipc/.ipc-node/start-node.sh 2>/dev/null || echo 'File not found'"
```

**Verify:** Does it contain `export FM_RESOLVER__CONNECTION__LISTEN_ADDR` and `export FM_IPC__SUBNET_ID`?

---

## Step 4: Check How Node Is Currently Running

```bash
ssh philip@34.16.93.183 "ps aux | grep 'ipc-cli node start' | grep -v grep"
```

**Check:** Is the process started by start-node.sh or by a direct nohup command? (env vars only apply if set before the process starts)

---

## Step 5: Manual Test – Run With Explicit Env Vars

Stop the node, then run manually with env vars to isolate whether config or env is the issue:

```bash
# On validator-1 (34.16.93.183)
ssh philip@34.16.93.183

# Stop existing node
sudo pkill -f "ipc-cli node start" || true
sleep 3

# Run as ipc user with explicit env vars (no wrapper script)
sudo -u ipc env \
  FM_RESOLVER__CONNECTION__LISTEN_ADDR=/ip4/0.0.0.0/tcp/26654 \
  FM_IPC__SUBNET_ID=/r314159/t410fjzsmxroshdmvdq5bg4zwqxx5lznwxaga4h7zgqa \
  /home/ipc/ipc/target/release/ipc-cli node start --home /home/ipc/.ipc-node

# Let it run 15-20 seconds, then Ctrl+C to stop
# In another terminal, check port:
#   ssh philip@34.16.93.183 "ss -tuln | grep 26654"
```

**If port 26654 appears:** Env vars work; the wrapper script or how it's invoked is the problem.
**If port 26654 does NOT appear:** Config or binary (e.g. f3-lifecycle branch) may disable the resolver.

---

## Step 6: Check for Override Configs

Config load order: default.toml → validator.toml → local.toml → env. Later overrides can clear earlier values.

```bash
ssh philip@34.16.93.183 "sudo -u ipc ls -la /home/ipc/.ipc-node/fendermint/config/"
ssh philip@34.16.93.183 "sudo -u ipc cat /home/ipc/.ipc-node/fendermint/config/validator.toml 2>/dev/null || echo 'No validator.toml'"
ssh philip@34.16.93.183 "sudo -u ipc cat /home/ipc/.ipc-node/fendermint/config/local.toml 2>/dev/null || echo 'No local.toml'"
```

## Step 7: Check Binary / Branch

```bash
# Fix safe.directory first, then check branch
ssh philip@34.16.93.183 "sudo -u ipc git -C /home/ipc/ipc config --global --add safe.directory /home/ipc/ipc 2>/dev/null; sudo -u ipc bash -c 'cd /home/ipc/ipc && git branch -v && git log -1 --oneline'"
```

**Note:** If validators run `f3-lifecycle` (or another branch), resolver logic may differ from `feature/subnet-bootstrapping`.

---

## Step 8: Check Default Config Template

If the node was initialized with a different node-init, the default.toml may have been generated without resolver settings:

```bash
ssh philip@34.16.93.183 "sudo -u ipc head -100 /home/ipc/.ipc-node/fendermint/config/default.toml"
```

---

## Summary: Decision Tree

| Config has listen_addr? | Config has subnet_id? | Log says "disabled"? | Likely cause |
|-------------------------|----------------------|----------------------|--------------|
| No / empty               | -                    | Yes                  | Config missing resolver.connection.listen_addr |
| Yes                      | No / UNDEF           | Yes                  | Config missing ipc.subnet_id |
| Yes                      | Yes                  | Yes                  | Env override not applied (script/quoting) or binary differs |
| Yes                      | Yes                  | No ("starting...")   | Resolver starts but port bind fails (e.g. permission, conflict) |

---

## After Finding Root Cause

1. **If config is wrong:** Fix default.toml (or re-run node init with correct node-init.yml)
2. **If env vars not applied:** Fix start script invocation (wrapper script, quoting, or use systemd with Environment=)
3. **If binary/branch differs:** Build from feature/subnet-bootstrapping or adapt to that branch's config
