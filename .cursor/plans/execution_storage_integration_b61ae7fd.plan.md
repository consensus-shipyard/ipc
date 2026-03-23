---
name: Execution Storage Integration
overview: Define how execution jobs should consume inputs from storage and publish outputs back through storage references while keeping chain orchestration lightweight.
todos:
  - id: artifactref-schema
    content: Define structured ArtifactRef in shared execution types with migration path for local refs
    status: pending
  - id: worker-fetch-path
    content: Implement storage-backed input fetch in run-executor using existing storage utilities
    status: pending
  - id: worker-publish-path
    content: Upload execution outputs to storage and commit refs + hash on completion
    status: pending
  - id: cli-output-fetch
    content: Add CLI command(s) to inspect and fetch committed output refs
    status: pending
  - id: integration-tests
    content: Add end-to-end test from storage input through execution and committed storage output
    status: pending
isProject: false
---

# Execution-Storage Integration Plan

## Objective

Connect execution jobs to storage artifacts end-to-end so workers fetch inputs from storage, execute deterministically, and publish outputs back as storage-backed refs.

## Phase 1: ArtifactRef contract

- Extend shared execution job schema in `[/Users/karlem/work/ipc/fendermint/actors/blobs/shared/src/execution.rs](/Users/karlem/work/ipc/fendermint/actors/blobs/shared/src/execution.rs)`:
  - Replace plain `Vec<String>` refs with structured refs (scheme, hash/id, optional metadata).
  - Keep backwards-compatible parsing for existing `local://` refs during transition.
- Update actor state handling in `[/Users/karlem/work/ipc/fendermint/actors/blobs/src/state/execution.rs](/Users/karlem/work/ipc/fendermint/actors/blobs/src/state/execution.rs)` to store normalized refs.

## Phase 2: Worker input fetch

- Implement resolver in `[/Users/karlem/work/ipc/ipc-storage/ipc-decentralized-storage/src/bin/node.rs](/Users/karlem/work/ipc/ipc-storage/ipc-decentralized-storage/src/bin/node.rs)`:
  - `local://` for local binaries/dev.
  - `iroh://` or `blob://` for storage-backed inputs.
- Reuse existing storage client/distribution utilities from `[/Users/karlem/work/ipc/ipc-storage/ipc-decentralized-storage/src/objects.rs](/Users/karlem/work/ipc/ipc-storage/ipc-decentralized-storage/src/objects.rs)` for download path and integrity checks.

## Phase 3: Output publication

- After execution success:
  - Persist stdout/stderr/artifacts through storage flow (Iroh upload + blob/object registration).
  - Commit only resulting storage refs + commitment hash in `CompleteJob`.
- Keep large data off-chain; keep chain payload bounded to refs and cryptographic commitment.

## Phase 4: CLI and observability

- In `[/Users/karlem/work/ipc/ipc/cli/src/commands/exec/mod.rs](/Users/karlem/work/ipc/ipc/cli/src/commands/exec/mod.rs)`:
  - Add `exec outputs`/`exec fetch` helper to resolve and download output refs.
  - Show resolved ref type and integrity status.
- Add worker logs/metrics for fetch time, execute time, upload time, and commit latency.

## Phase 5: Guardrails and tests

- Guardrails:
  - allowlist binaries/schemes,
  - timeout/size/env limits,
  - reject unknown ref schemes.
- Tests:
  - actor unit tests for ref validation/state transitions,
  - worker integration test: storage input -> execution -> storage output -> on-chain completion.

```mermaid
flowchart LR
  userSubmit[UserSubmitsJob] --> chainJob[BlobsActorJobPending]
  chainJob --> workerPoll[WorkerPollsAndClaims]
  workerPoll --> inputFetch[FetchInputRefsFromStorage]
  inputFetch --> hostExec[HostBinaryExec]
  hostExec --> outputStore[StoreOutputsToStorage]
  outputStore --> chainComplete[CompleteJobWithOutputRefsAndCommitment]
  chainComplete --> userRead[UserReadsStatusAndFetchesOutputs]
```

## Success criteria

- Jobs reference real storage inputs/outputs (not only local paths).
- Worker does not place raw payloads on-chain.
- Users can fetch outputs through CLI from committed refs.
