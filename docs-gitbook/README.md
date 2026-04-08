# Introduction

Inter-Planetary Consensus (IPC) provides **extensible, customizable** blockchain infrastructure through autonomous sidechains — subnets. Each subnet is a **sovereign** blockchain with its own validators, governance, and data. Subnet state is cryptographically anchored through checkpoints that serve as **objective trust anchors**, securing cross-chain messaging and guarding against long-range attacks. Failures are **contained** within individual security boundaries. The architecture is **recursive**: subnets can themselves be extended with lower-level subnets.

## What IPC enables

- **Scalability** — delegate operations to subnets optimized for performance: faster finality, higher throughput, cheaper block space. Subnets can be launched and terminated programmatically to match demand.

- **Customizability** — tailor the consensus mechanism, execution environment, fee model, governance, and data availability strategy to the application's requirements. Subnets can be permissioned or permissionless.

- **Interoperability** — native cross-chain communication through protocol-level bridging. Asset deposits, withdrawals, and general message passing across chains, without external bridges.

- **Autonomy** — subnets operate as sovereign blockchains, owning their data and governance. No centralized sequencers, no posting bulk data to parent chains. Partition-tolerant: subnets keep running when temporarily disconnected.

- **Objective security** — checkpoint-based objectivity secures against long-range attacks and enables trustless bootstrapping. Risk is contained: failures in one subnet do not propagate to others.

## Documentation overview

- [**Protocol**](protocol/README.md) — how the IPC protocol works: core concepts, mechanisms, architecture, workflows, security model.
- [**Getting Started**](quickstarts/deploy-a-subnet.md) — deploy your first subnet.
- [**Guides**](user-guides/performing-transactions-in-a-subnet.md) — transacting in subnets, customizing and upgrading subnets, deploying an explorer.
- [**Specifications**](../specs/addressing.md) — implementation-level specs.
- [**Reference**](reference/networks.md) — networks, CLI usage, glossary, troubleshooting, FAQ.
