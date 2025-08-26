# IPC CLI

## Building with UI Assets

The IPC CLI includes an embedded web UI for managing subnet deployments. The frontend assets are embedded at compile time, so they need to be built before compiling the CLI.

### Quick Start

For development and production builds, use these make commands:

```bash
# Build frontend assets only
make build-ui

# Build frontend assets AND IPC CLI (recommended)
make build-with-ui

# From project root (includes contract generation)
make build-with-ui
```

### Manual Build Process

If you prefer to build manually:

```bash
# 1. Build frontend assets
cd ../ipc-ui/frontend
npm ci && npm run build

# 2. Build IPC CLI with embedded assets
cd ../../ipc
cargo build --release
```

### Important Notes

- **Always rebuild after frontend changes**: The frontend is embedded at compile time using `include_dir!`
- **Frontend location**: Assets are built to `ipc-ui/frontend/dist/`
- **Embedded path**: Backend includes files from `$CARGO_MANIFEST_DIR/../../ipc-ui/frontend/dist`

## Usage

Once built, start the UI service:

```bash
./target/release/ipc-cli ui
```

This serves both the frontend and API on `http://localhost:3000`.