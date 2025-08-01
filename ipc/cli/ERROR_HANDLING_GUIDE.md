# IPC CLI Error Handling Guide

This guide explains how to use the new user-friendly error handling system in the IPC CLI.

## Overview

The new error handling system provides:
- Clear, actionable error messages for users
- Specific guidance on how to fix issues
- Consistent error formatting across all commands
- Type-safe error handling

## Error Types

The error system is organized into domain-specific error types:

### 1. WalletError
Used for wallet-related operations (import, export, create, etc.)

Examples:
- `WalletError::StdinNotSupported` - When user tries to use stdin
- `WalletError::InvalidKeyFormat` - When key format is incorrect
- `WalletError::NoWallet` - When no wallet is configured

### 2. SubnetError
Used for subnet operations (create, join, leave, etc.)

Examples:
- `SubnetError::SubnetNotFound` - When subnet doesn't exist
- `SubnetError::InsufficientCollateral` - When collateral is too low
- `SubnetError::AlreadyValidator` - When already joined as validator

### 3. NodeError
Used for node operations (init, start, configure)

Examples:
- `NodeError::HomeDirectoryNotFound` - When node home doesn't exist
- `NodeError::CometBftNotInstalled` - When CometBFT is missing
- `NodeError::PortInUse` - When required port is already taken

### 4. CrossMsgError
Used for cross-message operations

Examples:
- `CrossMsgError::InvalidTokenAmount` - When amount format is wrong
- `CrossMsgError::PropagationFailed` - When propagation fails

### 5. ValidationError
Used for input validation

Examples:
- `ValidationError::InvalidEthAddress` - Invalid Ethereum address
- `ValidationError::MissingParameter` - Required parameter missing

### 6. NetworkError
Used for network and RPC issues

Examples:
- `NetworkError::RpcConnectionFailed` - Can't connect to RPC
- `NetworkError::RequestTimeout` - Request timed out

## Implementation Guide

### Step 1: Import Error Types

Add the error types to your command module:

```rust
use crate::errors::{CliError, WalletError, SubnetError, NodeError, ValidationError};
```

### Step 2: Replace Generic Errors

Instead of using `bail!`, `anyhow!`, or `ensure!`, use specific error types:

**Before:**
```rust
bail!("stdin not supported yet");
```

**After:**
```rust
return Err(WalletError::StdinNotSupported {
    wallet_type: arguments.wallet_type.clone(),
}.into());
```

### Step 3: Add Context to Operations

For operations that might fail, provide helpful context:

**Before:**
```rust
let keyinfo = fs::read_to_string(path)
    .with_context(|| format!("Failed to read key file: {:?}", path))?;
```

**After:**
```rust
let keyinfo = fs::read_to_string(&path_buf)
    .map_err(|_| WalletError::CannotReadKeyFile { 
        path: path_buf.clone() 
    })?;
```

### Step 4: Handle External Library Errors

When calling external libraries, map their errors to our types:

```rust
let address = provider.new_evm_key()
    .map_err(|e| CliError::Internal(
        format!("Failed to create EVM wallet: {}", e)
    ))?;
```

## Examples

### Example 1: Wallet Import

```rust
pub fn import_wallet(
    provider: &IpcProvider,
    arguments: &WalletImportArgs,
) -> Result<ImportedWallet> {
    // Validate wallet type
    let wallet_type = WalletType::from_str(&arguments.wallet_type)
        .map_err(|_| WalletError::UnsupportedWalletType {
            wallet_type: arguments.wallet_type.clone(),
        })?;

    // Handle stdin case with user-friendly error
    let keyinfo = match &arguments.path {
        Some(path) => {
            let path_buf = PathBuf::from(path);
            fs::read_to_string(&path_buf)
                .map_err(|_| WalletError::CannotReadKeyFile { path: path_buf })?
        }
        None => {
            return Err(WalletError::StdinNotSupported {
                wallet_type: arguments.wallet_type.clone(),
            }.into())
        }
    };

    // Import with specific error handling for each wallet type
    match wallet_type {
        WalletType::Fvm => {
            provider.import_fvm_key(&keyinfo).map_err(|e| {
                WalletError::InvalidKeyFormat {
                    wallet_type: "fvm".to_string(),
                    details: e.to_string(),
                    expected_format: "Base64-encoded key info JSON".to_string(),
                }
            })?
        }
        WalletType::Evm => {
            // ... similar pattern
        }
    }
}
```

### Example 2: Node Validation

```rust
async fn validate_node_home(home: &Path) -> Result<()> {
    if !home.exists() {
        return Err(NodeError::HomeDirectoryNotFound {
            path: home.to_path_buf(),
        }.into());
    }

    if !home.is_dir() {
        return Err(NodeError::HomeNotDirectory {
            path: home.to_path_buf(),
        }.into());
    }

    // Check for required files
    let config_path = home.join("config.toml");
    if !config_path.exists() {
        return Err(NodeError::NodeNotInitialized {
            path: home.to_path_buf(),
            missing_file: "config.toml".to_string(),
        }.into());
    }

    Ok(())
}
```

### Example 3: Amount Parsing

```rust
let amount = BigInt::from_str_radix(arguments.amount.as_str(), 10)
    .map_err(|_| CrossMsgError::InvalidTokenAmount {
        input: arguments.amount.clone(),
    })?;
```

## Error Message Guidelines

1. **Be Specific**: Tell users exactly what went wrong
2. **Be Actionable**: Tell users how to fix the issue
3. **Include Examples**: Show correct format/usage
4. **Avoid Technical Jargon**: Use plain language
5. **Provide Context**: Include relevant values that caused the error

## Testing Error Messages

When implementing error handling:

1. Test the error case manually to see the message
2. Ensure the message is helpful from a user's perspective
3. Include all necessary information to resolve the issue
4. Avoid exposing internal implementation details

## Common Patterns

### Pattern 1: File Operations
```rust
fs::read_to_string(&path)
    .map_err(|_| WalletError::CannotReadKeyFile { path })?
```

### Pattern 2: Parsing Operations
```rust
WalletType::from_str(&wallet_type)
    .map_err(|_| WalletError::UnsupportedWalletType { wallet_type })?
```

### Pattern 3: Network Operations
```rust
client.request().await
    .map_err(|e| NetworkError::RpcConnectionFailed {
        endpoint: client.endpoint().to_string(),
    })?
```

### Pattern 4: Validation
```rust
if address.len() != 42 || !address.starts_with("0x") {
    return Err(ValidationError::InvalidEthAddress {
        input: address.to_string(),
    }.into());
}
```

## Migration Checklist

When updating a command to use the new error handling:

- [ ] Add error type imports
- [ ] Replace all `bail!` with specific error types
- [ ] Replace all `anyhow!` with specific error types
- [ ] Replace generic `.context()` with specific error mapping
- [ ] Update `ensure!` to use specific validation errors
- [ ] Test error messages are user-friendly
- [ ] Document any new error types added

## Adding New Error Types

If you need a new error variant:

1. Add it to the appropriate error enum in `errors.rs`
2. Follow the existing pattern for error messages
3. Implement Clone for the error type
4. Use `#[error("...")]` to define the user-facing message
5. Include actionable guidance in the message