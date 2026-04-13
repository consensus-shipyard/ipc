# Security & Economics

## Security Model & Properties

This section provides a unified security perspective on the IPC protocol, building on the [requirements, assumptions, and guarantees](requirements-and-guarantees.md) and the correctness arguments in [Core Mechanisms](core-mechanisms.md).

**Adversary model.** The IPC security model considers the following classes of adversarial behavior.

- **Byzantine faults within subnets.** Up to some threshold of validators may exhibit [Byzantine faults](https://en.wikipedia.org/wiki/Byzantine_fault), as determined by the subnet's consensus mechanism (e.g., up to one-third of voting power for typical BFT-based consensus). Below this threshold, the protocol's security guarantees hold; above it, the subnet may make incorrect state transitions, but the resulting damage is bounded to the subnet's own security domain.

- **Full subnet compromise.** In the worst case, validators may collude beyond the consensus threshold. A compromised subnet could produce arbitrary state, submit false checkpoints, and even steal assets delegated to it. However, it cannot affect sibling subnets or parent chains beyond the subnet's circulating supply, due to the bridge safety invariants enforced by on-chain logic.

- **Cross-chain proxy misbehavior.** Proxies may exhibit arbitrary behavior. A proxy that fabricates data has no effect: all submitted data is verified by the receiving chain's on-chain logic, which rejects invalid submissions. A proxy that withholds data (omission) could stall cross-chain communication but cannot cause incorrect state transitions. Redundancy through multiple independent proxies readily mitigates omission.

- **Out of scope.** The security model does not consider breaks in cryptographic primitives, parent chain compromise (assumed secure for this analysis), or social-layer and client-side attacks.

**Trust hierarchy.** IPC establishes a layered trust structure with the following properties.

- **Rootnet as root of trust.** The rootnet has no parent chain and requires the highest level of security. Each parent chain is assumed to be strictly more secure and more conservative than its subnets, recursively up to the rootnet.

- **Subnet anchoring.** Subnets are anchored into their parent chains through periodic checkpoints, which provide objective, verifiable reference points for the subnet's state. This anchoring does not fully transfer the parent's security properties to the subnet; the subnet's own security depends on its own validators and consensus mechanism. Rather, the anchoring provides an objective basis for verifying the subnet's state from outside the subnet.

- **Upward risk containment.** A subnet cannot compromise its parent chain's integrity. This containment is enforced by on-chain logic on the parent chain, specifically checkpoint and cross-net message verification, as well as per-subnet supply accounting.

- **User risk acceptance.** Users who deposit assets into a subnet accept the security domain of that subnet; their exposure is limited to the assets they have delegated to it.

- **Independent evaluability.** The security of any individual subnet can be assessed based on its own validator set, consensus mechanism, and governance policies, independently of other subnets.

**Security properties.** The IPC protocol offers the following guarantees, each grounded in the mechanisms and assumptions described throughout the protocol description.

- **Checkpoint integrity.** Committed checkpoints faithfully reflect the finalized state of the checkpointed chain. This relies on two complementary factors: the checkpointed chain operating correctly within its security domain, producing genuine checkpoints; and the receiving chain verifying incoming checkpoints against the recognized configuration of the checkpointed chain (e.g., validating quorum signatures against the known validator set). Checkpoint integrity is foundational — all cross-chain operations built on checkpoints, including bridging and message verification, depend on this property.

- **Checkpoint-based objectivity.** Checkpoints committed to a trusted chain serve as objective, independently verifiable reference points to the checkpointed chain's recent state. This mitigates long-range attacks: an adversary cannot rewrite a chain's history beyond the last committed checkpoint without also compromising the chain storing it. Checkpoint-based objectivity also enables trustless bootstrapping, allowing new participants to verify and join a chain by obtaining a recent checkpoint from a trusted chain, without relying on social consensus.

- **Bridge safety.** The lock-mint-burn-release cycle is consistent across chains. On-chain supply accounting enforces the invariant that released amounts never exceed previously locked amounts and burned amounts never exceed previously minted amounts. Each deposit and withdrawal forms a matching pair across both chains. Cross-net messages carrying assets are verified against the corresponding checkpoints before execution.

- **Risk containment.** A compromised subnet can only affect assets and operations that have been delegated to it. It cannot inflate the parent chain's supply or corrupt sibling subnets' state. This is enforced through per-subnet supply accounting and verification of cross-net messages against checkpoints.

- **Liveness.** Proxy or relayer failure can delay cross-chain data flow but cannot cause incorrect state transitions. A subnet that becomes temporarily partitioned from its parent chain continues operating autonomously, achieving local finality; consistency is restored upon reconnection.

**Security boundaries and risk analysis.** The following discusses the consequences of different component failures and violations of protocol assumptions.

- **Subnet compromise.** If too many subnet validators collude, they can cause invalid state transitions and submit false checkpoints, potentially leading to unauthorized extraction of assets from the subnet. While the parent chain would accept these checkpoints if they pass quorum verification, the impact remains capped by on-chain logic that limits asset release to the subnet's circulating supply. The risk does not propagate to the parent chain or sibling subnets. This threat can be effectively mitigated by ensuring robust validator diversity, requiring adequate collateral from subnet validators, and enforcing slashing mechanisms for provable misbehavior (see open questions below).

- **Proxy/relayer failure.** If proxies fail or withhold data, cross-chain communication could stall — checkpoints and cross-net messages delayed. However, no incorrect state can be introduced. To ensure reliability, multiple independent proxies can operate concurrently for redundancy.

- **Temporary partition.** If a subnet becomes disconnected from its parent chain, it continues operating and achieving local finality. But cross-chain operations — deposits, withdrawals, and configuration updates — would stall until connectivity is restored. Staleness of cross-chain state is bounded by the duration of partitioning. Mitigations include resilient network connectivity, monitoring, and automatic reconnection.

- **Parent chain reorg beyond checkpoint.** If the parent chain reverts state beyond what the subnet has already checkpointed, this violates the stable reference points requirement. The subnet may have committed state based on now-reverted parent state. With immediate-finality parent chains, this scenario does not arise. With probabilistic finality, conservative confirmation depth mitigates the risk. Choosing finality criteria conservatively and monitoring for parent chain divergence are the primary mitigations.

**Open questions and limitations.** There are some areas where the current security model could be further developed or refined.

- **Accountability.** No formal accountability mechanism is specified yet. Such a mechanism is critical for deterring validator misbehavior. The protocol will include slashing of collateral upon provable misbehavior as a core deterrent.

- **Fraud and validity proofs.** The protocol achieves simplicity and efficiency by relying on quorum-based trust for checkpoints rather than performing verification of subnets' full state transitions.

- **Multi-hop routing security.** Security of routing cross-net messages through intermediate chains is not yet fully analyzed. Only single-hop messaging (direct parent-to-subnet or subnet-to-parent) is currently enabled.

- **Timing and latency bounds.** Formal bounds on checkpoint latency, finality propagation delay, and partition tolerance are not yet specified.

## Economic Incentives

The IPC protocol involves several participant roles — subnet validators, cross-chain proxies, and users who delegate assets and operations to subnets — each with distinct incentive requirements. Economic mechanisms serve to align these participants' behavior with correct protocol operation and sustained participation. Three main categories of economic mechanisms are relevant: collateral and slashing, transaction fees, and cross-chain proxy incentivization. These mechanisms are highly customizable per subnet; their specific parameters form part of the subnet's configuration.

**Collateral and slashing.** Validator collateral, introduced in [Subnet Lifecycle Management](core-mechanisms.md#subnet-lifecycle-management) and [Joining & Leaving Subnets](workflows.md#joining--leaving-subnets), is a prerequisite for validator participation in collateral-based subnets. Collateral is locked on a chain capable of enforcing it against validators — typically the parent chain, given its higher security, though the choice depends on where configuration authority resides. The key property is that validators cannot unilaterally reclaim their collateral; its release is governed by protocol rules. Collateral thus creates direct financial exposure to the subnet's correct operation, aligning validators' economic interest with honest behavior. The collateral amount may determine a validator's voting power allocation or be uniform across validators, depending on the subnet's policies.

Slashing — partial or full forfeiture of collateral upon provable misbehavior — complements consensus-level fault tolerance with economic consequences. Slashing targets behaviors that are verifiable on-chain or provable to the enforcing chain, such as equivocation. The conditions under which slashing is triggered, the severity of penalties, and the adjudication process — including what constitutes sufficient proof and who evaluates it — are customizable per subnet.

Collateral also serves a lifecycle function: it may gate subnet activation and is subject to unbonding delays upon validator departure, ensuring that validators remain accountable for recent behavior before reclaiming their stake.

*In the current implementation,* collateral is managed through IPC contracts on the parent chain. In collateral-based (permissionless) subnets, voting power is proportional to staked collateral; federated subnets have power assigned by the subnet owner; a static mode fixes power at the initial collateral amounts. Departed validators' collateral enters a timed release queue and must be explicitly claimed. Slashing is not yet implemented.

**Fees.** Subnets define their own fee models as part of their configuration. Transaction fees generally serve two purposes: preventing spam and denial-of-service by imposing a cost on consuming chain resources, and compensating validators for the computational and infrastructure costs of operating the subnet. Fee parameters — including the pricing mechanism (e.g. fixed fees, auction-based pricing, or dynamic gas markets), fee recipient policies, and fee levels — are customizable per subnet. This flexibility enables a range of fee models, from conventional gas markets to subsidized or entirely fee-free operation for specialized or private subnets. The fee model is internal to each subnet; the IPC protocol does not prescribe a specific fee structure.

*In the current implementation,* subnets use an [EIP-1559](https://eips.ethereum.org/EIPS/eip-1559)-style dynamic gas market, with base fee adjustment driven by block utilization and gas premiums routed to the block producer. Validator rewards are additionally supported through an activity-tracking pipeline: per-validator consensus participation is recorded on the subnet, propagated to the parent chain via checkpoints, and used for reward distribution through a pluggable reward contract supplied at subnet deployment. No default reward policy is prescribed. Cross-net message fees are not currently charged.

**Cross-chain proxy incentivization.** Cross-chain proxies bear operational costs, including transaction fees on receiving chains and the infrastructure required for monitoring and data submission. Although the proxy role is permissionless, sustained and reliable operation requires positive economic incentive. In practice, subnet validators often serve as cross-chain proxies as well, since they already maintain the necessary infrastructure and have a direct economic stake in the subnet's cross-chain liveness. Dedicated incentivization mechanisms can additionally support independent proxy operators, improving redundancy and reliability of cross-chain data flow. No specific proxy incentivization mechanism is currently specified in the protocol.

*Currently,* cross-chain proxies receive no on-chain incentives; validators are expected to handle relaying as part of their normal operations.

**Open questions.**
- *Slashing conditions and adjudication.* Specific conditions for slashing, evidence requirements, and adjudication mechanisms remain to be defined.
- *Reward distribution.* Models for distributing rewards to validators — such as block rewards, fee sharing, or inflation-based subsidies — are not yet specified.
- *Cross-chain fee coordination.* It is not yet determined how fees for cross-net message execution on the destination chain are accounted for and settled.
- *Proxy incentivization mechanism.* A concrete mechanism for compensating cross-chain proxies — and its interaction with validator incentives — has not been designed.
- *Economic parameter propagation.* How economic parameters (e.g. fee policies, slashing rules) propagate and interact across the subnet hierarchy is an open area.
- *MEV considerations.* The implications of [maximal extractable value](https://ethereum.org/en/developers/docs/mev/) in cross-chain message ordering have not yet been analyzed.
