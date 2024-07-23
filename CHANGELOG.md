# Changelog

All notable changes to this project will be documented in this file.

### Observability Framework

- Introduced a new observability framework utilizing the `ipc-observability` crate.

  - This framework introduces events and metrics for detailed system monitoring and analysis.
  - Seamlessly integrates with Prometheus for real-time tracking, alerting, and visualization.
  - Simplifies observability integration with ready-to-use macros, structs, and functions.

## Added traces and metrics

- Introduced the `BlockProposalReceived` event, which tracks block proposal reception.
  - Metric: `proposals_block_proposal_received` (CounterVec)
- Added the `BlockProposalSent` event, which tracks block proposal sending.
  - Metric: `proposals_block_proposal_sent` (CounterVec)
- Implemented the `BlockProposalEvaluated` event, which evaluates block proposals.
  - Metrics: `proposals_block_proposal_accepted` (CounterVec), `proposals_block_proposal_rejected` (CounterVec)
- Created the `BlockCommitted` event for tracking committed blocks.
  - Metric: `proposals_block_committed` (CounterVec)
- Added the `MsgExec` event to represent various execution purposes such as Check, Apply, Estimate, and Call.
  - Metrics: `exec_fvm_check_execution_time_secs` (Histogram), `exec_fvm_estimate_execution_time_secs` (Histogram), `exec_fvm_apply_execution_time_secs` (Histogram), `exec_fvm_call_execution_time_secs` (Histogram)
- Introduced the `CheckpointCreated` event for creating bottom-up checkpoints.
  - Metrics: `bottomup_checkpoint_created_total` (IntCounter), `bottomup_checkpoint_created_height` (IntGauge), `bottomup_checkpoint_created_msgcount` (IntGauge), `bottomup_checkpoint_created_confignum` (IntGauge)
- Implemented the `CheckpointSigned` event for signing bottom-up checkpoints.
  - Metric: `bottomup_checkpoint_signed_height` (IntGaugeVec)
- Added the `CheckpointFinalized` event for finalizing bottom-up checkpoints.
  - Metric: `bottomup_checkpoint_finalized_height` (IntGauge)
- Created the `ParentRpcCalled` event to track parent RPC calls.
  - Metrics: `topdown_parent_rpc_call_total` (IntCounterVec), `topdown_parent_rpc_call_latency_secs` (HistogramVec)
- Added the `ParentFinalityAcquired` event for acquiring parent finality.
  - Metric: `topdown_parent_finality_latest_acquired_height` (IntGaugeVec)
- Implemented the `ParentFinalityPeerVoteReceived` event for receiving parent finality peer votes.
  - Metric: `topdown_parent_finality_voting_latest_received_height` (IntGaugeVec)
- Created the `ParentFinalityPeerVoteSent` event for sending parent finality peer votes.
  - Metric: `topdown_parent_finality_voting_latest_sent_height` (IntGauge)
- Introduced the `ParentFinalityPeerQuorumReached` event to signify quorum reach in parent finality.
  - Metrics: `topdown_parent_finality_voting_quorum_height` (IntGauge), `topdown_parent_finality_voting_quorum_weight` (IntGauge)
- Added the `ParentFinalityCommitted` event to track committed parent finality.
  - Metric: `topdown_parent_finality_committed_height` (IntGauge)
- Implemented the `TracingError` event to log tracing errors.
  - Metric: `tracing_errors` (IntCounterVec)

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
