# Interpreter Documentation

This directory contains documentation for the Interpreter integration work within the IPC project.

## Overview

The Interpreter integration provides the execution engine for the IPC network, integrating with the Filecoin Virtual Machine (FVM) and managing transaction execution.

## Documentation Index

### Integration
- **[INTERPRETER_INTEGRATION_STATUS.md](INTERPRETER_INTEGRATION_STATUS.md)** - Current integration status and progress
- **[INTERPRETER_FILES_ANALYSIS.md](INTERPRETER_FILES_ANALYSIS.md)** - Analysis of interpreter files and structure

## Quick Links

- [Interpreter Source](../../../fendermint/vm/interpreter/) - Interpreter implementation
- [FVM State Execution](../../../fendermint/vm/interpreter/src/fvm/state/exec.rs) - Core execution logic
- [Module System](../module-system/) - Related module system documentation

## Getting Started

1. Review [INTERPRETER_INTEGRATION_STATUS.md](INTERPRETER_INTEGRATION_STATUS.md) for current status
2. Read [INTERPRETER_FILES_ANALYSIS.md](INTERPRETER_FILES_ANALYSIS.md) for file structure understanding

## Architecture

The interpreter is a core component that:
- Executes smart contract transactions
- Manages FVM integration
- Handles state transitions
- Processes cross-subnet messages
