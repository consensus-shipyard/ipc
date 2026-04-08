# Protocol Architecture

The fundamental mechanisms of the IPC protocol — what the protocol accomplishes from a conceptual perspective — are described in [Core Mechanisms](core-mechanisms.md). This section focuses on the protocol's architecture — the structural components through which those mechanisms manifest.

The IPC protocol can be realized through three kinds of components: **on-chain logic**, **subnet validator nodes**, and **cross-chain proxies**. The distribution of responsibilities among these components and their forms depends on the capabilities of the participating chains and architectural decisions. As a result, a single IPC chain can be realized in multiple architectural roles—for example, a subnet may also act as a parent chain to its own child subnets, thus fulfilling multiple architectural roles.

## On-chain Logic

On-chain logic is the IPC-specific **transition logic of the replicated state** — the chain state — that makes a chain an IPC participant. It may manifest as built-in or user-deployable smart contracts, built-in protocol rules, or state encoded within a chain's native data structures, depending on the chain's capabilities. On-chain logic does not include coordination mechanisms such as the consensus protocol, where individual validators' state may differ; rather, it encompasses the deterministic state transition rules that all validators agree upon and execute identically.

The core role of on-chain logic is **verifying** cross-chain data, **enforcing** protocol rules, and **recording** protocol state. The form and sophistication of these operations varies with the chain's programmability — from rich smart contract environments to minimal scripting — but the core verification and rule enforcement are always present in IPC's on-chain logic.

**Core responsibilities.** On-chain logic is present on both parent chains and subnets: each IPC chain, with respect to each connected chain, performs the following functions.

- *Checkpoint verification and recording* — verifying the authenticity of checkpoints received from connected chains (e.g. by validating quorum signatures or finality certificates against the recognized configuration of the checkpointed chain) and storing the verified checkpoint data as part of the chain's own state. Stored checkpoints serve as trust anchors for all subsequent cross-chain operations.

- *Cross-net message handling* — for outgoing messages, recording and batching messages initiated on this chain with checkpoints; for incoming messages validating them against the relevant checkpoint, then either forwarding them or, if addressed to this chain, executing the corresponding state changes such as asset operations or contract calls.

- *Asset bridging invariant enforcement* — tracking locked and minted assets per connected chain and enforcing that assets released never exceed those previously locked, and assets burned never exceed those previously minted, ensuring the consistency of the lock-mint-burn-release cycle across the bridge.

- *Reflection of connected chain configuration* — maintaining the configuration parameters of connected chains (such as validator sets and finality requirements) necessary for cross-chain data verification. The reflected configuration is updated via cross-chain operations, for example by processing configuration commitments included in checkpoints or by receiving updates through dedicated cross-net messages.

- *Own configuration management* — managing the chain's own operational parameters (such as the validator set, consensus settings, and checkpoint period) as part of the replicated state, governing activation points for configuration transitions.

**Parent-specific responsibilities.** In addition to the above, parent chains carry responsibilities specific to managing their child subnets: maintaining child subnets' identity and registration (the parent acts as a registry), governing subnet lifecycle transitions (activation conditions, operational status, termination procedures), and establishing the initial recognized subnet configuration at activation.

**Architecture-dependent aspects.** Certain aspects of on-chain logic may vary depending on architectural choices. Subnet ID allocation may follow coordinated on-chain assignment or coordination-less derivation. Checkpoint content assembly — constructing the cryptographic commitments that constitute a checkpoint — may be handled by on-chain logic during periodic state transitions or by other components such as validator nodes or cross-chain proxies.

*In the current implementation,* on-chain logic is realized through a combination of user-deployed and built-in native smart contracts executed within a general-purpose VM. On the parent chain, a shared per-chain contract handles cross-net message routing, supply accounting, and parent finality tracking, while per-subnet contracts manage validators, collateral, lifecycle, and checkpoint verification. On subnet chains, equivalent contracts handle parent finality, message processing, and checkpoint assembly, with native contracts supporting auxiliary functions such as gas market management and validator activity tracking.

## Subnet Validator Node

The subnet validator node is the software run by subnet validators that operates the subnet as a blockchain. It hosts the execution environment in which the subnet-side on-chain logic runs. Although the on-chain logic executes within the validator node, they are conceptually distinct: the node provides the consensus, networking, and execution infrastructure around the on-chain state transition logic.

**Core blockchain operation.** The validator node's fundamental responsibilities are those of any blockchain node: participating in the consensus protocol to agree on block contents and ordering, executing transactions and producing state transitions (including on-chain IPC logic), and maintaining the subnet's replicated (chain) state. Additionally, the node typically produces justification material — such as block signatures or finality certificates from the consensus mechanism — that can serve as proof of finalized state. This material can later be collected by cross-chain proxies for checkpoint submission to connected chains, or used by connected chains directly for verification.

**Cross-chain interaction.** Beyond core blockchain operation, validator nodes may participate in cross-chain coordination. How cross-chain data enters the subnet's block processing pipeline is an architectural choice, with two principal approaches.

In the *direct observation* approach, validator nodes independently monitor the parent chains and coordinate among themselves — for example, through voting within the consensus process — to agree on which parent chain state should be considered finalized. The agreed-upon finality is then recorded via on-chain logic.

In the *proxy-mediated* approach, validator nodes receive updates from connected chains through cross-chain proxies, accompanied by justification material such as finality certificates and data integrity proofs. The on-chain logic verifies and records this data without the node needing to interact with the connected chains directly. This approach requires the checkpointed chain to produce native finality certificates that the receiving chain's on-chain logic can verify independently.

These approaches can coexist within the same deployment — for instance, a subnet would typically use direct observation for parent chain finality tracking while relying on a separate cross-chain proxy for submitting checkpoints to the parent.

*In the current implementation,* the validator node combines a BFT consensus engine ([CometBFT](https://docs.cometbft.com/)) with a general-purpose VM for execution ([FVM](https://docs.filecoin.io/smart-contracts/fundamentals/the-fvm)). Validators directly poll the parent chain, reaching agreement on parent finality through an internal voting protocol or by verifying the parent chain's native finality certificates. Top-down messages and validator set changes are fetched from the parent and incorporated into block proposals. Bottom-up checkpoint content is assembled by on-chain logic during block execution; submission to the parent chain is handled by a separate cross-chain proxy (relay).

## Cross-Chain Proxy

The cross-chain proxy acts as an intermediary between connected IPC chains, observing protocol-relevant state on one chain, optionally processing or preparing the data, and then submitting it to another chain.

**What the proxy does.** The proxy operates through three main tasks:

- *Observation* — monitoring a chain's finalized state for protocol-relevant data such as checkpoints, cross-net message batches, finality markers, and configuration changes.
- *Preparation* — packaging the observed data for submission to the receiving chain. This may involve collecting justification material (consensus artifacts, finality certificates), constructing data integrity and inclusion proofs, and formatting data according to the receiving chain's verification requirements.
- *Submission* — delivering the prepared data to the receiving chain's network, triggering on-chain protocol operations such as checkpoint recording, message verification and processing, or configuration updates.

**Trust model.** The proxy is untrusted for correctness: all submitted data is ultimately verified by the receiving chain's state transition logic, and the proxy cannot cause acceptance of invalid data, regardless of the receiving chain's programmability level. The proxy may perform computation such as proof construction or data pre-processing, but this constitutes preparation, not authoritative verification. The proxy is, however, responsible for liveness: if no proxy operates, cross-chain data flow stalls and dependent protocol operations are delayed — but no incorrect state can be introduced through proxy failure. The proxy role is normally permissionless; multiple independent proxies can operate concurrently for reliability.

**Deployment and scope.** A proxy is typically necessary for the bottom-up direction, since parent chains are generally passive and do not actively track their subnets' state. For the top-down direction, a proxy may or may not be needed, depending on whether subnet validator nodes directly observe the parent chain. A proxy can operate as a standalone process, or its functionality can be embedded within validator nodes or other infrastructure. A single proxy may serve one direction, both directions, or a specific subset of cross-chain data flows.

*In the current implementation,* a separate proxy process handles the bottom-up direction: it reads BFT-signed block headers and checkpoint data from the subnet and submits them, along with message batch inclusion proofs, to the parent chain's contracts for on-chain verification and processing. No separate proxy handles the top-down direction; validator nodes poll the parent chain directly.
