# Requirements & Guarantees

IPC operates under specific requirements and assumptions while providing corresponding security and operational guarantees.

## Requirements

- **Stable reference points.**

    IPC chains must produce state commitments that become stable and serve as reliable anchors for checkpoints. Whether achieved through deterministic or probabilistic mechanisms, stable commitments are considered practically final and immutable.

- **Minimal chain capabilities.**

    IPC chains must support: recording arbitrary data on-chain (e.g., checkpoints, cross-net messages); trust-minimized asset bridging (lock-mint-burn-release operations); and enforcement of IPC governance mechanisms (i.e. subnet ID assignment, subnet lifecycle management, and subnet configuration updates). The required on-chain programmability is minimal by design to support diverse blockchain environments.

- **Cross-chain connectivity.**

    Subnets maintain eventual connectivity with their parent chain networks for checkpointing and bridging, tolerating temporary partitions. Continuous connectivity is not required—subnets can operate autonomously during poor connectivity periods and synchronize once connectivity is restored.

## Assumptions

- **Parent chain security dominance.**

    Parent chains are assumed more secure than their subnets. This security relationship holds recursively up to the rootnet.

- **Parent chain conservativeness.**

    Parent chains are assumed to be more conservative in adopting changes than their subnets. Parent chains maintain higher protocol stability while subnets can be heavily specialized or adopt experimental functionality.

- **Bounded subnet security.**

    Subnets are assumed to operate correctly in managing delegated assets and their own state, within subnet's security strength. Users delegating assets accept risks according to the subnet's security parameters.

## Core Guarantees

- **Checkpoint-based objectivity.**

    Checkpoints committed to a trusted chain establish objective reference points to recent state of the checkpointed chain, verifiable without trust assumptions beyond the security of the chain they are committed to. Those reference points objectively mitigate long-range attacks, i.e. attempts to create alternative chain histories beyond the last committed checkpoint.

- **Risk containment.**

    The impact of any subnet misbehavior is limited to assets and operations delegated to that subnet, with no further propagation to parent chains or sibling subnets. Failures within a subnet affect only that subnet's security domain.

- **Eventual consistency.**

    Transitively checkpointing IPC chains into a common trusted chain establishes eventual consistency among them. Chains sharing common trusted checkpoint anchors can verify each other's state and coordinate cross-chain operations.
