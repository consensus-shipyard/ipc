# IPC Documentation

This directory contains comprehensive documentation for the InterPlanetary Consensus (IPC) framework.

## Architecture Documentation

### 📐 [Architecture Overview](./architecture-overview.md)
Comprehensive guide to IPC architecture, components, and their interactions. Includes:
- Detailed component descriptions and responsibilities
- Communication flows (top-down and bottom-up)
- Subnet lifecycle management
- Security model and trust assumptions
- Configuration parameters and deployment scenarios

**Includes 4 diagrams:**
1. Detailed architecture diagram
2. Simplified high-level view
3. Message flow sequences (fund and release)
4. Validator lifecycle state machine

### 📜 [Protocol Capture](./protocol-capture.md)
Protocol capture document describing how IPC works:
- Components (actors, contracts, node, relayer)
- Concepts and mechanisms (messaging, bridging, checkpointing, consensus)
- Security guarantees and trust assumptions
- Configuration, terms, dependencies

### 📋 [Architecture Quick Reference](./architecture-quick-reference.md)
Quick reference card with:
- Core concepts glossary
- CLI command cheat sheet
- Common operations and workflows
- Configuration templates
- Troubleshooting guide
- Performance tuning tips

## User Guides

### 🚀 [Usage Guide](./usage.md)
Complete guide to using the `ipc-cli`:
- Wallet management (create, import, export)
- Subnet operations (join, leave, stake)
- Cross-net messaging (fund, release)
- Running relayers
- Checkpoint monitoring

### 📝 [Contract Documentation](./contracts.md)
Smart contract deployment and interaction:
- Deploying FEVM actors on subnets
- Contract interfaces and ABIs
- Integration examples

### 🏗️ [Deploying Hierarchy](./deploying-hierarchy.md)
Step-by-step guide for deploying your own IPC instance:
- Network setup and prerequisites
- Contract deployment process
- Validator configuration
- Bootstrap node setup

### ⚠️ [Contract Errors](./contract-errors.md)
Comprehensive reference of contract errors:
- Error codes and meanings
- Common causes
- Resolution steps

## Developer Guides

### 💻 [Developers Guide](./developers.md)
For IPC core developers:
- Development setup
- Testing procedures
- Contributing guidelines
- Code structure and patterns

### 🔄 [Subnet Creation Flow](./subnet-creation-flow.md)
Technical details of subnet creation process

### 🔐 [Validator Gater](./validator-gater.md)
Validator permission and access control

### 🎯 [Querying Subnet Permission Mode](./querying-subnet-permission-mode.md)
How to check and interact with subnet permission settings

## Operational Guides

### 🏁 [Node Initialization](./node-init.md)
Setting up a Fendermint node

### ▶️ [Node Start](./node-start.md)
Starting and managing Fendermint nodes

### 📊 [Subnet Initialization](./subnet-init.md)
Initializing subnet infrastructure

### 🌐 [Web App Design](./webapp-design.md)
IPC web application architecture and design

### 🚀 [Quickstart Calibration](./quickstart-calibration.md)
Quick start guide for Calibration testnet

## Visual Reference

All architecture diagrams are available in the documentation and include:

### Main Architecture Diagrams
1. **Detailed Architecture**: Complete component view with internal structures
2. **Simplified Architecture**: High-level layers and communication flows
3. **Message Flows**: Sequence diagrams for fund and release operations
4. **Validator Lifecycle**: State machine for validator operations

### Existing IPC Actor Diagrams
Located in `../../specs/`:
- Architecture design overview
- Subnet registry pattern
- Software stack and relayers
- Create subnet flow
- Top-down messaging
- Stake and membership changes
- Checkpoint submission
- Bottom-up messaging

## Getting Started

### New Users
1. Read the [Architecture Quick Reference](./architecture-quick-reference.md) for core concepts
2. Follow the [Quickstart Calibration](./quickstart-calibration.md) guide
3. Learn common operations from the [Usage Guide](./usage.md)

### Validators
1. Understand the architecture from [Architecture Overview](./architecture-overview.md)
2. Review the [Validator Lifecycle](./architecture-overview.md#subnet-lifecycle)
3. Follow deployment steps in [Deploying Hierarchy](./deploying-hierarchy.md)
4. Set up nodes using [Node Initialization](./node-init.md) and [Node Start](./node-start.md)

### Developers
1. Study the [Architecture Overview](./architecture-overview.md) for system design
2. Review [Developers Guide](./developers.md) for contribution guidelines
3. Understand smart contracts from [Contract Documentation](./contracts.md)
4. Check [Contract Errors](./contract-errors.md) for debugging

### Subnet Creators
1. Understand [Subnet Creation Flow](./subnet-creation-flow.md)
2. Review configuration options in [Architecture Quick Reference](./architecture-quick-reference.md)
3. Follow [Deploying Hierarchy](./deploying-hierarchy.md) guide
4. Configure permissions using [Validator Gater](./validator-gater.md)

## External Resources

- **IPC Website**: https://www.ipc.space/
- **GitHub Repository**: https://github.com/consensus-shipyard/ipc
- **Calibration Faucet**: https://faucet.calibration.fildev.network/funds.html
- **Filecoin Documentation**: https://docs.filecoin.io/
- **CometBFT Documentation**: https://docs.cometbft.com/

## Related Documentation

### Fendermint Documentation
Located in `../fendermint/`:
- [Architecture](../fendermint/architecture.md)
- [IPC Infrastructure](../fendermint/ipc.md)
- [Checkpointing](../fendermint/checkpointing.md)
- [Running Fendermint](../fendermint/running.md)
- [Local Network Testing](../fendermint/localnet.md)

### Specifications
Located in `../../specs/`:
- [IPC Actors](../../specs/ipc-actors.md)
- [Bottom-Up Interaction](../../specs/bottom-up-interaction.md)
- [Top-Down Finality](../../specs/topdown.md)
- [Addressing](../../specs/addressing.md)
- [Executions](../../specs/executions.md)

### Troubleshooting
- [Troubleshooting Subnet Deployment](../troubleshooting-subnet-deployment.md)
- [Contract Errors Reference](./contract-errors.md)

## Documentation Maintenance

This documentation is actively maintained. If you find errors or have suggestions:

1. **Report Issues**: Open an issue on GitHub
2. **Contribute**: Submit a pull request with improvements
3. **Ask Questions**: Join #ipc-help in [Filecoin Slack](https://filecoin.io/slack)

## Document Status

| Document | Status | Last Updated |
|----------|--------|--------------|
| Architecture Overview | ✅ Complete | Jan 2026 |
| Architecture Quick Reference | ✅ Complete | Jan 2026 |
| Usage Guide | ✅ Complete | - |
| Contract Documentation | ✅ Complete | - |
| Deploying Hierarchy | ✅ Complete | - |
| Contract Errors | ✅ Complete | - |
| Developers Guide | ✅ Complete | - |

---

**Navigation**: [Main README](../../README.md) | [Fendermint Docs](../fendermint/) | [Specifications](../../specs/)
