# IPC Documentation

Welcome to the InterPlanetary Consensus (IPC) documentation. This directory contains comprehensive documentation for the IPC project, organized by topic and feature area.

## Documentation Structure

### [Feature Documentation](features/)
Detailed documentation for specific features implemented in IPC:

- **[Plugin System](features/plugin-system/)** - Plugin architecture and development
- **[Recall System](features/recall-system/)** - Recall implementation and migration
- **[Module System](features/module-system/)** - Module system implementation phases
- **[Storage Node](features/storage-node/)** - Storage node integration
- **[Interpreter](features/interpreter/)** - Interpreter integration
- **[IPC Library](features/ipc-library/)** - IPC library extraction and design

### [IPC Core Documentation](ipc/)
Core IPC usage, deployment, and development guides:

- [Usage Guide](ipc/usage.md) - How to use IPC
- [Deploying Hierarchy](ipc/deploying-hierarchy.md) - Deploy subnet hierarchies
- [Quickstart - Calibration](ipc/quickstart-calibration.md) - Quick start with Calibration testnet
- [Contracts Documentation](ipc/contracts.md) - IPC smart contracts
- [Developer Guide](ipc/developers.md) - Guide for IPC developers

### [Fendermint Documentation](fendermint/)
Fendermint-specific documentation (Tendermint-based subnet peer):

- [Architecture](fendermint/architecture.md) - Fendermint architecture overview
- [Running Fendermint](fendermint/running.md) - How to run Fendermint nodes
- [Checkpointing](fendermint/checkpointing.md) - Checkpointing mechanism
- [Local Network](fendermint/localnet.md) - Running a local test network
- [Observability](fendermint/observability.md) - Monitoring and logging

### [Development Documentation](development/)
General development resources:

- [Build Verification](development/BUILD_VERIFICATION.md) - Verify your build
- [Feature Flags](development/FEATURE_FLAGS_EXPLAINED.md) - Feature flag documentation
- [Testing Results](development/PHASE5_TESTING_RESULTS.md) - Testing outcomes

## Additional Resources

- [Troubleshooting](troubleshooting-subnet-deployment.md) - Common issues and solutions
- [Manual Checks](manual-checks.md) - Manual verification procedures

## External Documentation

- [GitBook Documentation](../docs-gitbook/) - User-facing documentation
- [Specifications](../specs/) - Technical specifications and design documents

## Quick Start

New to IPC? Start here:

1. Read the [main README](../README.md) in the project root
2. Follow the [IPC Quickstart Guide](ipc/quickstart-calibration.md)
3. Review [IPC Usage Documentation](ipc/usage.md)
4. Explore [Feature Documentation](features/) for specific capabilities

## Contributing

When adding new documentation:

1. Place feature-specific docs in the appropriate `features/` subdirectory
2. Update the relevant README.md to reference your new documentation
3. Follow the [documentation conventions](../.cursor/rules/documentation-conventions.mdc)
4. Cross-link related documentation for better navigation

## Getting Help

- Check [Troubleshooting Guide](troubleshooting-subnet-deployment.md)
- Review [FAQ](../docs-gitbook/reference/faq.md) in GitBook docs
- See [IPC CLI Usage](../docs-gitbook/reference/ipc-cli-usage.md) for command reference
