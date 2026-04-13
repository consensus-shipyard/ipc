# Protocol Workflows

This section describes the key workflows of the IPC protocol — the end-to-end processes through which subnets are created and terminated, validators join and leave, assets are deposited and withdrawn, and chains exchange messages.

## Creating & Terminating Subnets

**Creating a subnet.** The process begins with **registration** on the parent chain, which assigns a subnet ID and records the subnet's parameters — such as its permission mode, checkpoint period, and supply asset types. The parent chain maintains the authoritative record of the subnet as its child.

Registration may be followed by a separate **activation** step, depending on the policies in effect. Activation may be gated by configuration-level criteria — for example, a minimum number of validators or a collateral threshold. Until these criteria are met, the subnet remains registered but inactive. Once activation occurs, the subnet's initial configuration is established on the parent chain, and the checkpointing relationship between the subnet and its parent begins; bridging becomes available. The subnet network is normally instantiated as part of activation. In principle, support for anchoring an already-running chain as a new subnet could also be implemented, allowing such chains to integrate into the IPC hierarchy.

**Terminating a subnet.** Termination concludes a subnet's participation in IPC. Before the subnet can be fully terminated, all cross-chain obligations need to be resolved: locked assets released and pending messages processed. Checkpointing and bridging should continue until these obligations are fulfilled. Termination may be initiated explicitly or triggered by lifecycle rules — for example, the validator count dropping below a threshold. The termination process may require the subnet's cooperation.

*In the current implementation,* IPC contracts on the parent deploy a per-subnet contract; registration additionally requires approval from the parent chain's IPC contract owner. Activation occurs automatically when collateral and validator thresholds are met, or when the owner designates a federated validator set. Termination requires all validators to have departed and the parent's recorded circulating supply for the subnet to be zero.

## Joining & Leaving Subnets

**Joining.** The policies governing validator admission may range from open to restricted. Open policies may require locking assets (collateral) on the parent chain as a condition of participation; restricted policies may limit validator set changes to a designated authority. The specific policies are part of the subnet's settings.

Before activation, validator set composition feeds into the subnet's initial configuration, assembling the set that will operate the subnet from its start. After activation, validator set changes follow the rules of [configuration management](core-mechanisms.md#subnet-configuration-management): both the parent chain and the subnet need a consistent view of the current validator set, and cross-chain confirmation ensures this consistency. Changes take effect at designated activation points.

**Leaving.** Departure conditions mirror the joining policy. Any resources committed during joining — such as collateral — become reclaimable according to the applicable rules, which may impose delays or conditions such as waiting for cross-chain confirmation or an unbonding period. Validator departures may also have lifecycle implications: if the departure causes the subnet to fall below its activation thresholds, the subnet may be deactivated.

*In the current implementation,* collateral-based subnets allow permissionless joining by locking collateral via IPC contracts on the parent; federated subnets have their validator set managed by the subnet owner. Post-activation changes are confirmed through bottom-up checkpoints carrying configuration numbers. An active validator limit caps the consensus set, with excess validators placed in a waiting pool. Collateral from departed validators enters a release queue and must be explicitly claimed.

## Depositing & Withdrawing Assets

**Depositing.** To deposit assets into a connected chain, assets are locked on the origin chain — held in custody — and a cross-net message triggers minting of equivalent assets on the destination. The locked assets remain in custody on the origin chain for the lifetime of the deposit. The types of assets accepted are determined by the chain capabilities and bridging settings.

**Withdrawing.** Withdrawals reverse the deposit process: assets are burned on the source chain, and a cross-net message triggers release of equivalent locked assets on the origin. Withdrawal messages are verified against checkpoints before assets are released.

**Supply accounting.** Each side of the bridge tracks the amounts locked and minted per connected chain, enforcing that released or burned amounts never exceed what was previously locked and minted, respectively. This invariant preserves the integrity of the lock-mint-burn-release cycle across the bridge. Initial circulating supply may be established at activation. Participants may commit assets before activation, forming the subnet's starting supply; these commitments can be reclaimed if activation does not proceed.

*In the current implementation,* deposits lock native or [ERC-20](https://eips.ethereum.org/EIPS/eip-20) assets via IPC contracts on the parent and commit a top-down transfer message; the subnet credits the recipient in its native asset after observing parent finality. Withdrawals burn native assets on the subnet and batch the resulting bottom-up messages into periodic checkpoints; IPC contracts on the parent verify the batch against the checkpoint commitment, decrease the subnet's recorded circulating supply, and release assets to the recipient. Pre-activation funding allows participants to commit assets to the subnet's genesis circulating supply.

## General Message Passing

Cross-net messages can carry an information payload, assets, or both. Deposits and withdrawals, described in [Depositing & Withdrawing Assets](#depositing--withdrawing-assets), are a specialization — a single message can also combine a value transfer with an information payload, enabling patterns such as invoking on-chain logic on another chain while simultaneously transferring assets.

**Message flow.** Top-down and bottom-up flows share the same message structure and routing infrastructure. Top-down messages are typically applied on the subnet after it observes and checkpoints finalized parent state. Bottom-up messages are batched and verified against checkpoints on the parent.

**Execution and results.** At the destination, message execution can trigger on-chain logic. Result messages route back to the sender, enabling request-response patterns across chains. Results for result messages are not generated, preventing infinite loops.

**Multi-hop routing.** Messages between non-adjacent chains can route through the hierarchy via their common ancestor. At each intermediate chain, the message is validated and forwarded; the common ancestor serves as the pivot where routing transitions from bottom-up to top-down.

*In the current implementation,* messages use a uniform envelope with kind (transfer, call, result), source and destination addresses, value, and an encoded payload. Top-down messages are committed on the parent and applied on the subnet after parent finality is observed; bottom-up messages are batched per checkpoint period and verified via [Merkle proofs](https://en.wikipedia.org/wiki/Merkle_tree) on the parent. General-purpose messages invoke a handler interface on the destination contract. Only single-hop messaging is currently fully supported.
