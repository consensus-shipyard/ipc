# Glossary

**Activation (subnet).** The lifecycle transition at which a registered subnet becomes operational and its checkpointing relationship with the parent chain begins.

**Activation point (subnet configuration).** A well-defined boundary — such as a checkpoint or epoch boundary — at which configuration changes take effect.

**Bottom-up message.** A cross-net message flowing from a subnet to its parent chain.

**Bridging.** The IPC mechanism for secure transfer of information and assets between connected chains.

**Checkpoint.** A compact cryptographic commitment to a chain's finalized state, submitted to and recorded on a connected chain.

**Checkpoint-based objectivity.** The property that checkpoints committed to a trusted chain serve as objective, independently verifiable reference points to the checkpointed chain's state — without relying on social consensus or trusted third parties.

**Circulating supply (subnet).** The amount of assets currently locked on the parent chain on behalf of a subnet — increased by deposits and decreased by withdrawals.

**Collateral.** Assets locked on the parent chain as a condition for validator participation in a subnet.

**Cross-chain proxy.** An intermediary that observes protocol-relevant state on one IPC chain, prepares the data (e.g. constructing proofs), and submits it to another chain. Untrusted for correctness but responsible for liveness.

**Cross-net message.** The carrier for all bridging operations between IPC chains, specifying source and destination chains, and carrying assets, an arbitrary information payload, or both.

**Deposit.** A cross-net message that locks assets on the origin chain and mints equivalent (wrapped) assets on the destination chain.

**Eventual consistency.** The property that IPC chains transitively checkpointing into a common trusted chain can verify each other's state and coordinate cross-chain operations.

**Finality certificate.** Cryptographic justification — such as quorum signatures or native finality proofs — demonstrating that a checkpoint corresponds to finalized state of the checkpointed chain.

**General (cross-net) message.** A cross-net message carrying an information payload — with or without accompanying assets — that can trigger arbitrary state transitions (e.g. contract invocations) on the destination chain.

**IPC chain.** Any blockchain participating in the IPC protocol, whether a rootnet or a subnet.

**Multi-hop routing.** Routing a cross-net message between non-adjacent chains through intermediate chains via their common ancestor.

**On-chain logic.** The deterministic transition logic of a chain's replicated state. It may manifest as smart contracts, built-in protocol rules, or native data structures, depending on the chain's capabilities.

**Parent chain.** The blockchain into which a subnet's state is anchored via periodic checkpoints. Parent chains are assumed to be more secure and more conservative than their subnets.

**Registration (subnet).** The initial lifecycle step that makes a subnet known to its parent chain by recording the subnet's parameters and assigning it a subnet ID.

**Risk containment.** The guarantee that failures, attacks, or misbehavior within one subnet are confined to that subnet's security domain and do not propagate to parent chains or sibling subnets.

**Rootnet.** An IPC chain with no parent chain, serving as the root of trust for the IPC hierarchy.

**Subnet.** An autonomous blockchain whose state is anchored into a parent chain via periodic checkpoints.

**Subnet configuration.** The operational parameters defining a subnet chain's overall behavior and security — such as the validator set and voting power allocation, consensus parameters, checkpoint period, and fee settings — as opposed to the state of individual accounts or contracts.

**Subnet ID.** An unambiguous, stable identifier assigned to a subnet, used for addressing and routing of cross-net messages as well as subnet discovery.

**Subnet validator node.** The software run by subnet validators that operates the subnet as a blockchain, hosting consensus, networking, and execution infrastructure around the on-chain state transition logic.

**Supply accounting.** Tracking the amounts of assets locked and minted per connected chain to enforce that released or burned amounts never exceed what was previously locked and minted, respectively.

**Termination (subnet).** The lifecycle transition that concludes a subnet's operation, after resolving all cross-chain obligations.

**Top-down message.** A cross-net message flowing from a parent chain to a subnet.

**Validator set.** The set of validators operating a subnet, along with their voting power allocation.

**Withdrawal.** A cross-net message that burns (wrapped) assets on the source chain and releases equivalent locked assets on the origin chain.
