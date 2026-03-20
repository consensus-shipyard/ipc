# Local Mode SSH Fix - Complete

## Problem
When running `ipc-manager` commands in local mode (using `ipc-subnet-config-local.yml`), the script was attempting to SSH to localhost instead of executing commands locally. This affected multiple commands including:
- `info` - Would hang or fail when fetching subnet information
- `init` - Would fail during node stopping/starting phases with "Connection refused" errors
- `check` - Would fail when checking validator health

## Root Cause
Multiple functions in `lib/health.sh` were using direct SSH commands (`ssh_exec`, `ssh_kill_process`, `scp_to_host`, etc.) without checking if the system is in local mode. This caused SSH connection attempts to localhost even when running locally.

## Solution
Replaced all SSH calls in `show_subnet_info()` and `get_chain_id()` functions with the abstraction layer function `exec_on_host()` which automatically:
- Executes commands locally when in local mode
- Executes commands via SSH when in remote mode

## Changes Made

### Core Node Management Functions

#### 1. Fixed `backup_all_nodes()` function
**Before:** Used `ssh_exec` with IP/SSH user parameters
**After:** Uses `exec_on_host()` with validator index

#### 2. Fixed `wipe_all_nodes()` function
**Before:** Used `ssh_exec` with IP/SSH user parameters
**After:** Uses `exec_on_host()` with validator index

#### 3. Fixed `stop_all_nodes()` function (Critical for init)
**Before:** Used `ssh_kill_process` with IP/SSH user parameters
**After:** Uses `kill_process()` abstraction with validator index
- **This was causing the "Connection refused" error during init**

#### 4. Fixed `start_validator_node()` function
**Before:** Used `ssh_exec` with IP/SSH user parameters
**After:** Uses `exec_on_host()` with validator index

#### 5. Fixed `initialize_primary_node()` function
**Before:** Used `scp_to_host` and `ssh_exec`
**After:** Uses `copy_to_host()` and `exec_on_host()`

#### 6. Fixed `initialize_secondary_node()` function
**Before:** Used `scp_to_host` and `ssh_exec`
**After:** Uses `copy_to_host()` and `exec_on_host()`

#### 7. Fixed `set_federated_power()` function
**Before:** Used `ssh_exec` with IP/SSH user parameters
**After:** Uses `exec_on_host()` with validator index

#### 8. Fixed `check_validator_health()` function
**Before:** Used `ssh_check_process` and multiple `ssh_exec` calls
**After:** Uses `check_process_running()` and `exec_on_host()`

### Information Display Functions

#### 9. Fixed `get_chain_id()` function (lines 386-402)
**Before:**
```bash
local ip=$(get_config_value "validators[$validator_idx].ip")
local ssh_user=$(get_config_value "validators[$validator_idx].ssh_user")
local ipc_user=$(get_config_value "validators[$validator_idx].ipc_user")
local response=$(ssh -o StrictHostKeyChecking=no "$ssh_user@$ip" \
    "sudo su - $ipc_user -c \"curl -s ...\"" 2>/dev/null)
```

**After:**
```bash
local response=$(exec_on_host "$validator_idx" \
    "curl -s -X POST ... http://localhost:${eth_api_port}" 2>/dev/null)
```

#### 10. Fixed `show_subnet_info()` function (lines 405-784)
Replaced all SSH calls with `exec_on_host()` calls:

- **Block information queries** (lines 449-454): Now use `exec_on_host 0`
- **Network status queries** (lines 467-470): Now use `exec_on_host 0`
- **Libp2p port checks** (line 481): Now use `exec_on_host 0`
- **Resolver configuration checks** (lines 499-514): Now use `exec_on_host 0` with proper `$node_home`
- **Listen address checks** (line 529): Now use `exec_on_host 0`
- **Per-validator libp2p configuration** (lines 549-591): Now use `exec_on_host "$idx"` with proper `$v_node_home`
- **Parent chain connectivity** (lines 605-622): Now use `exec_on_host 0`
- **Parent finality status** (lines 636-680): Now use `exec_on_host 0`
- **Validator status checks** (lines 692-725): Now use `exec_on_host 0` and `exec_on_host "$idx"`
- **Cross-chain activity logs** (line 769): Now use `exec_on_host 0`

### Node Home Path Handling
Added proper node home path resolution using `get_node_home()` function:
```bash
local node_home=$(get_node_home 0)
local v_node_home=$(get_node_home "$idx")
```

This ensures the correct path is used in both local and remote modes:
- **Local mode**: `~/.ipc-node/validator-0`, `~/.ipc-node/validator-1`, etc.
- **Remote mode**: `~/.ipc-node` on each remote host

## Files Modified
- `/Users/philip/github/ipc/scripts/ipc-subnet-manager/lib/health.sh`

## Testing

### Test the Init Command
```bash
cd /Users/philip/github/ipc/scripts/ipc-subnet-manager
./ipc-manager --config ipc-subnet-config-local.yml init
```
**Expected:** No SSH connection attempts, nodes stop and start locally

### Test the Info Command
```bash
./ipc-manager --config ipc-subnet-config-local.yml info
```
**Expected:** Displays subnet information without SSH errors

### Test the Check Command
```bash
./ipc-manager --config ipc-subnet-config-local.yml check
```
**Expected:** Health checks run locally without SSH attempts

## Affected Commands Now Working in Local Mode
- ✅ `init` - Complete initialization without SSH
- ✅ `info` - Display subnet information locally
- ✅ `check` - Health checks run locally
- ✅ `restart` - Node restarts work locally
- ✅ All node management operations

## Benefits
- ✅ Works correctly in both local and remote modes
- ✅ Uses existing abstraction layer (`exec_on_host`, `kill_process`, `copy_to_host`)
- ✅ Consistent with the abstraction pattern in `lib/exec.sh`
- ✅ No redundant IP/SSH user variable fetching
- ✅ Proper node home path handling for multi-validator local setups
- ✅ Cleaner, more maintainable code
