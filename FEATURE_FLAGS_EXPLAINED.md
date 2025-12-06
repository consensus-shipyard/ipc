# Feature Flags - How They Work

## Current Configuration

In `fendermint/vm/interpreter/Cargo.toml`:

```toml
[features]
default = ["storage-node"]  # ← Default features when no flags specified
bundle = []
storage-node = [
    "dep:storage_node_executor",
    "dep:storage_node_kernel",
    "dep:storage_node_module",
    "dep:fendermint_actor_storage_adm",
    # ... more storage-node dependencies
]
```

## How It Works

### Scenario 1: No Feature Flags (Uses Default)
```bash
cargo build --release
```
- **Result:** Includes `storage-node` feature (because it's in `default`)
- **Compiles:** `storage_node_module` ✅

### Scenario 2: Explicit Feature Flag
```bash
cargo build --release --features storage-node
```
- **Result:** Includes `storage-node` feature (explicitly requested)
- **Compiles:** `storage_node_module` ✅
- **Note:** This works **regardless** of what's in `default`

### Scenario 3: No Default Features
```bash
cargo build --release --no-default-features --features bundle
```
- **Result:** Excludes `storage-node` feature (default disabled, not requested)
- **Compiles:** Only `bundle` feature ❌ (no storage_node_module)

## Your Question: "If storage-node was NOT default, would --features storage-node still work?"

**YES!** Here's the comparison:

### Current Setup (storage-node IS default):
```toml
default = ["storage-node"]
```

| Command | Includes storage-node? |
|---------|----------------------|
| `cargo build` | ✅ Yes (from default) |
| `cargo build --features storage-node` | ✅ Yes (explicit) |
| `cargo build --no-default-features` | ❌ No |
| `cargo build --no-default-features --features storage-node` | ✅ Yes (explicit) |

### If We Changed It (storage-node NOT default):
```toml
default = []  # or default = ["bundle"]
```

| Command | Includes storage-node? |
|---------|----------------------|
| `cargo build` | ❌ No (not in default) |
| `cargo build --features storage-node` | ✅ Yes (explicit) |
| `cargo build --no-default-features` | ❌ No |
| `cargo build --no-default-features --features storage-node` | ✅ Yes (explicit) |

## Key Insight

**`--features` always works, regardless of defaults!**

The `default = [...]` only affects what happens when you **don't** specify `--features` or `--no-default-features`.

Think of it like:
- `default` = "What features should I use if the user doesn't tell me?"
- `--features X` = "I want feature X, period." (overrides everything)
- `--no-default-features` = "Don't use the defaults, only what I explicitly request"

## Practical Examples

### Example 1: Make storage-node opt-in instead of default

**Change:**
```toml
# Before:
default = ["storage-node"]

# After:
default = []
```

**Usage:**
```bash
# Now you MUST explicitly request storage-node:
cargo build --release --features storage-node

# Without it, you get baseline only:
cargo build --release  # No storage-node!
```

### Example 2: Multiple features

```toml
default = ["bundle", "storage-node"]
```

```bash
# Get everything:
cargo build --release

# Get just storage-node (no bundle):
cargo build --release --no-default-features --features storage-node

# Get just bundle (no storage-node):
cargo build --release --no-default-features --features bundle

# Get both explicitly:
cargo build --release --no-default-features --features "bundle,storage-node"
```

## Recommendation for Your Project

**Current setup is good!** Having `storage-node` as default means:

✅ Users get full functionality out of the box
✅ `make` works as expected
✅ Advanced users can still opt-out with `--no-default-features`

**Alternative: Opt-in approach**
```toml
default = ["bundle"]  # Minimal by default
```

This would require users to explicitly add `--features storage-node`, which might be:
- 👍 Good for: Optional experimental features, large dependencies
- 👎 Bad for: Core functionality everyone needs

Your choice depends on whether storage-node is:
- **Core feature** → Keep in `default` ✅ (current)
- **Optional add-on** → Remove from `default`, make opt-in
