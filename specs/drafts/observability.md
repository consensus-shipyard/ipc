---
status: Implementing
authors: @raulk
---

# Observability

## Model

Our mental model when designing our observability logic is as follows.

---
```
💥 An **event** happens at runtime
  ⇒ 📸 A trace object captures the fact, including key data about it
    ⇒ ✍️ The trace gets journaled (if the journal is enabled)
    ⇒ 📊 The trace is applied to mutate metrics
```
---

Once we capture the trace object (hopefully with negligible overhead), and we ensure it is a complete snapshot of the fact, we can call `TraceEngine::emit()`, which can send it to two sinks in parallel:

- The journal.
- The metrics.

| Done | Domain    | Subsystem     | Event                                                                                    | Trace                                                                                                                                                           | Metrics                                                                            |
|------|-----------|---------------|------------------------------------------------------------------------------------------|-----------------------------------------------------------------------------------------------------------------------------------------------------------------|------------------------------------------------------------------------------------|
|      | Topdown   | Parent syncer | Parent RPC calls                                                                         | ParentRpcCalled{source, json-rpc method, status, latency}                                                                                                       | topdown_parent_rpc_call_total{source, method, status}++ (counter)                  |
|      |           |               |                                                                                          |                                                                                                                                                                 | topdown_parent_rpc_call_latency_secs{source, method, status} ← latency (histogram) |
|      | Topdown   | Parent syncer | Latest locally acquired parent finality                                                  | ~NewParentView~ already exists, rename to...ParentFinalityAcquired{source, is_null, block_height, block_hash, commitment_hash, num_msgs, num_validator_changes} | topdown_parent_finality_latest_acquired_height{source} = height (gauge)            |
|      | Topdown   | Parent syncer | Parent finality gossip received                                                          | ParentFinalityPeerVoteReceived{block_height, block_hash, commitment_hash, validator}                                                                            | topdown_parent_finality_voting_latest_received_height{validator} = height (gauge)  |
|      | Topdown   | Parent syncer | Parent finality peer                                                                     | ParentFinalityPeerVoteSent{block_height, block_hash, commitment_hash}                                                                                           | topdown_parent_finality_voting_latest_sent_height = height (gauge)                 |
|      | Topdown   | Parent syncer | Parent finality vote tally new agreement; recorded whenever the latest epoch with quorum | ParentFinalityPeerQuorumReached{block_height, block_hash, commitment_hash, weight}                                                                              | topdown_parent_finality_voting_quorum_height = height (gauge)                      |
|      |           |               |                                                                                          |                                                                                                                                                                 | topdown_parent_finality_voting_quorum_weight = sum of weight over 1 as f32 (gauge) |
|      | Topdown   | Parent syncer | Parent finality committed on chain                                                       | ParentFinalityCommitted{local_height, parent_height, block_hash, proposer}                                                                                      | topdown_parent_finality_committed_height = height (gauge)                          |
|      | Bottom-up | Checkpointing | Bottom-up checkpoint produced                                                            | CheckpointCreated{height, hash, msg_count, config_number}                                                                                                       | bottomup_checkpoint_created_total++ (counter)                                      |
|      |           |               |                                                                                          |                                                                                                                                                                 | bottomup_checkpoint_created_height = height (gauge)                                |
|      |           |               |                                                                                          |                                                                                                                                                                 | bottomup_checkpoint_created_msgcount = msgcount (gauge)                            |
|      |           |               |                                                                                          |                                                                                                                                                                 | bottomup_checkpoint_created_confignum = confignum (gauge)                          |
|      | Bottom-up | Checkpointing | Bottom-up checkpoint signed                                                              | CheckpointSigned{height, hash, validator}                                                                                                                       | bottomup_checkpoint_signed_height{validator} ← height (gauge)                      |
|      | Bottom-up | Checkpointing | Bottom-up checkpoint finalized                                                           | CheckpointFinalized{height, hash}                                                                                                                               | bottomup_checkpoint_finalized_height ← height (gauge)                              |
|      | Consensus | Proposals     | Block proposal received                                                                  | BlockProposalReceived{height, hash, size, tx_count, validator}                                                                                                  | << Check if CometBFT metrics suffice; if not, discuss >>                           |
|      | Consensus | Proposals     | Block proposal sent                                                                      | BlockProposalSent{height, hash, size, tx_count, validator}                                                                                                      | << Check if CometBFT metrics suffice; if not, discuss >>                           |
|      | Consensus | Proposals     | Block proposal accepted                                                                  | BlockProposalAccepted{height, hash, size, tx_count, validator}                                                                                                  | << Check if CometBFT metrics suffice; if not, discuss >>                           |
|      | Consensus | Proposals     | Block proposal rejected                                                                  | BlockProposalRejected{height, size, tx_count, validator, reason}                                                                                                | << Check if CometBFT metrics suffice; if not, discuss >>                           |
|      | Consensus | Commitment    | Block committed                                                                          | BlockCommitted{height, hash}                                                                                                                                    | << Check if CometBFT metrics suffice; if not, discuss >>                           |
|      | Execution | FVM           | Message executed at check stage (CheckTx)                                                | MsgExecCheck{height, from, to, value, param_len, exitcode, error, duration}                                                                                     | exec_fvm_check_execution_time_secs ← duration (histogram)                          |
|      | Execution | FVM           | Message executed at estimate stage (eth_estimateGas)                                     | MsgExecEstimate{height, from, to, value, param_len, exitcode, error, duration}                                                                                  | exec_fvm_estimate_execution_time_secs ← duration (histogram)                       |
|      | Execution | FVM           | Message executed at call stage                                                           | MsgExecCall{height, from, to, value, param_len, exitcode, error, duration}                                                                                      | exec_fvm_call_execution_time_secs ← duration (histogram)                           |
|      | Execution | FVM           | Message executed at apply stage                                                          | MsgExecApply{height, from, to, value, param_len, exitcode, error, duration}                                                                                     | exec_fvm_apply_execution_time_secs ← duration (histogram)                          |
|      | Mpool     | Mpool         | Mpool received (CheckTx)                                                                 | MpoolReceived{message_cid, from, to, value, param_len, gas_limit, fee_cap, premium, accept (bool)}                                                              | << Check if CometBFT metrics suffice; if not, discuss >>                           |
|      | Mpool     | Mpool         | Mpool broadcast                                                                          | MpoolReceived{message_cid, from, to, value, param_len, gas_limit, fee_cap, premium}                                                                             | << Check if CometBFT metrics suffice; if not, discuss >>                           |

## Fine-grained VM metrics

It might be helpful for debugging/optimization workflows at development time to have a mode that uses the pre-existing FVM traces to populate actor-level call graph metrics on Prometheus. Imagine metrics like:

```promql
exec_fvm_call_stack_step_duration{message_cid, seq, from, to} <- duration (histogram)
```

**Considerations:**

- The cardinality of such a metric is huge, and the histogram type would be lossy. For this reason, we can't treat it as a production-worthy metric. But it might be useful to detect outliers or abnormalities during development.
- We’d need to provide a developer API operation to reset this metric.

## Journal

The journal is a rotational ledger that records chronologically ordered, timestamped trace objects to log files on disk.
For ease of analysis and simplicity, we use JSON formatting for these objects (although we recognize that alternative binary formats such as Protocol Buffers would be more disk-efficient, but they would make the journals unreadable with the naked eye).
The journal is configured via a `[journal]` block within the Fendermint `config.toml` file. Ideally, we'd support dynamic reconfiguration, allowing for real-time application of changes to the journal settings. The configurable parameters include:

- Enable/disable the journal entirely.
- Filter by domains.
- Filter by specific events.
- Journal directory location.
- Journal file rotation policy (per-file size, retention window).

Implementation note: we're not reinventing the wheel here. It’s simply an abstraction over the `tracing` library and its appenders, all of which we already use. Refs:

- Tracing Appender: https://docs.rs/tracing-appender/latest/tracing_appender/rolling/
- EnvFilter Structure: https://docs.rs/tracing-subscriber/latest/tracing_subscriber/filter/struct.EnvFilter.html

For a pretty decent reference, look at the Lotus journal: https://github.com/filecoin-project/lotus/tree/master/journal. We even managed to slap an alerting system on top of it without breaking everything.

## Implementation

The existing metrics code paths are rather contorted and messy due to unnecessary indirection and excessive meta-programming. Read more commentary in the original pull request: https://github.com/consensus-shipyard/ipc/pull/835.

Take for example the `NewParentView` trace.

- Its struct is defined in the `fendermint_vm_event` crate.
- But it gets used in the `fendermint_vm_topdown` crate, where the event happens and the trace object gets emitted.
- But its metrics appliers are defined in the `fendermint_app` crate. Even worse: as a generic `tracing::Layer` which has lost all typing of the original event, so now it's force to introspect and match by string. Also, it only supports u64, and lacks support for labels (which would be hell to introduce on top of these macros).

We propose a radical simplification and cleanup.

- Every crate that produces traces has a conventional `observe` package. This package contains all trace structs and metric definitions. [This code was nice](https://github.com/consensus-shipyard/ipc/blob/a33cfee763cafaafdd00ad38a1367ee262e75fa4/ipld/resolver/src/stats.rs#L6).
- The `observe` package exposes a public `register_metrics(&Registry)` function so that the binary can register its metrics in its Prometheus Registry.
- An `ipc_metrics` crate exposes a simple `Recordable` trait:

    ```rust
    trait Recordable {
        fn record_metrics();
    }
    ```

- All trace structs implement `Recordable` (even if no-op), and they mutate the metrics within their crate only.
- When the emitting code performs `ipc_metrics::TraceEngine::emit(&trace);` and that records the metrics and sends them to the journal (potentially in parallel).

Note: the new `ipc_metrics` crate contains: the `Recordable` trait, `TraceEngine`,`Journal` implementation and configuration, general-purpose metrics utils and macros..
