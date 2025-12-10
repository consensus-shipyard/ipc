# IPC Plugins Directory

This directory contains auto-discoverable plugins for IPC.

## Plugin Convention

Each plugin must follow this structure:

```
plugins/
└── your-plugin-name/
    ├── Cargo.toml          # name = "ipc_plugin_your_plugin_name"
    └── src/
        └── lib.rs          # must export: pub fn create_plugin()
```

## Adding a New Plugin

1. Create directory: `mkdir -p plugins/my-plugin/src`
2. Create Cargo.toml with name: `ipc_plugin_my_plugin`
3. Implement `ModuleBundle` trait
4. Export: `pub fn create_plugin() -> Box<dyn ModuleBundle>`
5. Build with: `cargo build --features plugin-my-plugin`

That's it! No code changes to fendermint needed.

## Available Plugins

- **storage-node**: RecallExecutor-based storage node functionality
  - Build with: `--features plugin-storage-node`
  - Provides: RecallExecutor, storage actors, IPLD resolver

## How Discovery Works

The build script in `fendermint/app/build.rs` automatically:
1. Scans this directory
2. Checks which features are enabled (CARGO_FEATURE_PLUGIN_*)
3. Generates glue code to wire plugins
4. Zero hardcoded plugin names in fendermint source!
