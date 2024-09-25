# Changelog

All notable changes to this project will be documented in this file.

## [axon-r05] - 2024-09-24

_Full changelog below._

### ⭐ HIGHLIGHT | Token-bound validator collateral 🪙🔐

Token-bound validator collateral enables subnets to designate an `ERC20` token as the staking collateral for consensus validators, rather than being limited to the parent chain's native token (e.g., `FIL`).

This feature provides subnet operators with the flexibility to use their own tokens for staking, aligning network security with the native token economy. As subnets grow and evolve, this capability becomes increasingly important for subnet autonomy. Validators can stake the subnet’s native token, simplifying participation and reinforcing the subnet's economic model.

By decoupling consensus security from the parent token, subnet operators gain greater control over their own cryptoeconomics, making it easier to integrate staking, rewards, and penalties with their native token. This is essential for future subnet development, ensuring that staking mechanisms can evolve alongside the growth of specialized token economies.

### 🚀 Features

- _(contracts)_ Token-bound validator collateral (#1130)

### 🐛 Bug Fixes

- _(topdown)_ Pull effects up until committed finality. (#887)

### ⚙️ Miscellaneous Tasks

- Remove Python requirement for contracts development (#1144)

## [axon-r04] - 2024-09-18

_Full changelog below._

### ⭐ HIGHLIGHT | Validator Gating 🌁🌉

The Validator Gater feature allows the interception of validator-related actions, such as staking, unstaking, and explicit validator membership adjustments (federated membership), based on user-defined policies. By implementing a custom smart contract that adheres to the `IValidatorGater` interface, developers can enforce custom logic to either permit or deny these actions.

This feature is designed to support both federated and collateral-based networks, providing flexibility to manage validator permissions and validator power assignments through an external gating contract.

### 🚀 Features

- _(contracts)_ Validator gating (#1127)

### 📚 Documentation

- _(docs)_ Validator gating docs (#1127)

### ⚙️ Miscellaneous Tasks

- Fixed typos (#1137)
- Fixed clippy (#1133)

## [axon-r03] - 2024-09-06

_Full changelog below._

### ⭐ HIGHLIGHT | Consistent Genesis 🧬🚀

The Consistent Genesis feature introduces an additional step of sealing the genesis, ensuring the inclusion of the genesis state, including both custom and built-in actors. This step prevents inconsistencies during node initialization. Previously, the genesis process required certain actors to be deployed at runtime when the node started, which could result in a panic and prevent the node from starting. With the Consistent Genesis update, actor code is directly incorporated into the genesis as part of the state tree, ensuring stability and consistency across all node starts.

### 🚀 Features

- _(node)_ Consistent Genesis (#1016)
- _(contracts)_ Improvements to contract deployment scripts (#1108)

### 🐛 Bug Fixes

- _(core)_ Set the default Fendermint log level to INFO (#1123)
- _(ci)_ CI speed-up improvements (#1124)

### 📚 Documentation

- _(docs)_ Moved documentation to monorepo (#1014)
- _(specs)_ Subnet Genesis v2 spec (#1113)
- _(node)_ Updated running docs with Consistent Genesis (#1128)

### ⚙️ Miscellaneous Tasks

- Fixed typos and updated dependencies (#1087, #1106, #1089)
- Fixed clippy/fmt and improved cache usage (#1125)
- Applied Prettier formatting to contracts (#1111)

## [axon-r02] - 2024-07-23

_Full changelog below._

### ⭐ HIGHLIGHTED | Observability Framework 👁️📊

- Introduced a new observability framework utilizing the `ipc-observability` crate.
  - This framework introduces events and metrics for detailed system monitoring and analysis.
  - Integrates with Prometheus for real-time tracking, alerting, and visualization.
  - Simplifies observability integration with ready-to-use macros, structs, and functions.

IPC now emits events during execution. These events are recorded in the Journal, and are transformed into Prometheus metrics. Observability configuration is performed via `config.toml`.

Refer to full observability documentation [here](./docs/fendermint/observability.md).

### New events and metrics

| Domain    | Event                             | Description                                                               | Metric(s) derived                                                                                                                                                                                        |
| :-------- | --------------------------------- | ------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Consensus | `BlockProposalReceived`           | Tracks block proposal reception                                           | `consensus_block_proposal_received_height` (IntGauge)                                                                                                                                                    |
| Consensus | `BlockProposalSent`               | Tracks block proposal sending                                             | `consensus_block_proposal_sent_height` (IntGauge)                                                                                                                                                        |
| Consensus | `BlockProposalEvaluated`          | Records the result from evaluating block proposals                        | `consensus_block_proposal_accepted_height` (IntGauge), `consensus_block_proposal_rejected_height` (IntGauge)                                                                                             |
| Consensus | `BlockCommitted`                  | Tracks committed blocks                                                   | `consensus_block_committed_height` (IntGauge)                                                                                                                                                            |
| Execution | `MsgExec`                         | Represents various message execution paths (Check, Apply, Estimate, Call) | `exec_fvm_check_execution_time_secs` (Histogram), `exec_fvm_estimate_execution_time_secs` (Histogram), `exec_fvm_apply_execution_time_secs` (Histogram), `exec_fvm_call_execution_time_secs` (Histogram) |
| Bottomup  | `CheckpointCreated`               | Records checkpoint creation                                               | `bottomup_checkpoint_created_total` (IntCounter), `bottomup_checkpoint_created_height` (IntGauge), `bottomup_checkpoint_created_msgcount` (IntGauge), `bottomup_checkpoint_created_confignum` (IntGauge) |
| Bottomup  | `CheckpointSigned`                | Records checkpoint signatures                                             | `bottomup_checkpoint_signed_height` (IntGaugeVec)                                                                                                                                                        |
| Bottomup  | `CheckpointFinalized`             | Records checkpoint finalization (quorum reached)                          | `bottomup_checkpoint_finalized_height` (IntGauge)                                                                                                                                                        |
| Topdown   | `ParentRpcCalled`                 | Tracks parent RPC calls in the context of top-down finality               | `topdown_parent_rpc_call_total` (IntCounterVec), `topdown_parent_rpc_call_latency_secs` (HistogramVec)                                                                                                   |
| Topdown   | `ParentFinalityAcquired`          | Records acquisition of new parent finality                                | `topdown_parent_finality_latest_acquired_height` (IntGaugeVec)                                                                                                                                           |
| Topdown   | `ParentFinalityPeerVoteReceived`  | Records peer votes for parent finality                                    | `topdown_parent_finality_voting_latest_received_height` (IntGaugeVec)                                                                                                                                    |
| Topdown   | `ParentFinalityPeerVoteSent`      | Records own votes for parent finality                                     | `topdown_parent_finality_voting_latest_sent_height` (IntGauge)                                                                                                                                           |
| Topdown   | `ParentFinalityPeerQuorumReached` | Records quorum reached in parent finality voting                          | `topdown_parent_finality_voting_quorum_height` (IntGauge), `topdown_parent_finality_voting_quorum_weight` (IntGauge)                                                                                     |
| Topdown   | `ParentFinalityCommitted`         | Tracks parent finality committed on chain                                 | `topdown_parent_finality_committed_height` (IntGauge)                                                                                                                                                    |
| System    | `TracingError`                    | Logs tracing errors                                                       | `tracing_errors` (IntCounterVec)                                                                                                                                                                         |

### 🚀 Features

- _(node)_ New observability architecture + events (#1053)
- _(node)_ New observability bottom up tracing/metrics (#1061)
- _(ethapi)_ Add eth cors settings (#1021)
- _(node)_ File-based observability configuration (#1078)
- _(node)_ Observability docs and changelog section (#1083)

### 🐛 Bug Fixes

- _(ethapi)_ Make `eth_getTransactionReceipt` null for unexecuted/unknown transactions (#1006)

### 🚜 Refactor

- _(node)_ Observability refinements. (#1085)

### 📚 Documentation

- _(specs)_ Ethereum JSON-RPC API (#913)

### ⚙️ Miscellaneous Tasks

- Validate PR titles against conventional commits. (#1075)

## [axon-r01] - 2024-07-15

### Introducing Axon

Hello World! It's early days for IPC. We are starting to enact a proper versioning and changelog practices. The framework will evolve rapidly in the next quarters, and it'll do so in major architectural milestones. With high probability, backwards compatibility between these milestones will **not** be preserved, requiring a manual migration to upgrade from one to the next. To cite a few such expected milestones: IPC modularization, consensus pluggability, Wasm-based client kernel.

We introduce the notion of "product generations" to represent the lifetime of IPC under each of these major architectural iterations. Product generations are named alphabetically A-Z (we certainly don't expect more than 26 generations...) We've kept the naming universe deliberately broad: entities/concepts found in biological, mathematical, or computing networks.

The first product generation is called **_Axon_**!

![image](https://github.com/user-attachments/assets/7f9ac874-acdd-49d2-a409-995c55f6bfd4)

Find more background on these choices / implications here: https://github.com/consensus-shipyard/ipc/issues/1012.

### Axon r01

This is the baseline release of the IPC framework. A variation of this release is powering the networks of Fluence and Basin. Throughout the Axon generation, we do not expect to release crates and therefore we're staying away from adopting semver, resorting instead to simple sequential revision numbers. We're aiming to cut/tag revisions and publish changelogs on a weekly basis, with some flexibility to account for work in progress landing smoothly.

Axon r01 supports these major features (not a comprehensive list):

- CometBFT/Ignite-based consensus (currently on v0.37 but with plans to upgrade to [v0.38](https://github.com/consensus-shipyard/ipc/pull/1004) / v1).
- Wasm- and IPLD-based Filecoin Virtual Machine as an execution layer, supporting custom built-in Wasm actors, custom syscalls, custom gas price lists, and more.
- Ethereum-compatible runtime and JSON-RPC API (quasi-Dencun level, missing [MCOPY](https://github.com/filecoin-project/FIPs/discussions/1025) support).
- Validator membership: federated (proof of authority) and collateral-driven (basis for proof of stake).
- Configurable supply source for subnets: either inheriting the parent root coin, or adopting an ERC20-compatible token for circulating supply / gas.
- L2 subnet creation, with L3+ behind a feature flag until we harden message propagation, response paths, and fault scenarios.
- Asynchronous general message passing across the IPC hierarchy, with result and return data delivery back to the caller.
- Cross-linked security: checkpointing on the parent via the relayer, and committing parent's finality in the subnet.
- Ability to permission and restrict contract deployment in subnets.
- Upgradability: framework actors can be upgraded through contract upgrades, and subnet hard forks can be run with the UpgradeScheduler.
- Validator management through the parent network.
- Automatic chain snapshots, with ability to bootstrap from them.
- Compatibility with the BlockScount explorer and Ethereum wallets out of the box.
- ... and a lot more.

### Join the conversation!

Come ask your questions or give us feedback in the `#ipc` channel on [Filecoin Slack](https://filecoin.io/slack).
