# Protocol Overview

<details>
<summary><strong>TL;DR</strong></summary>

IPC is a protocol for extending blockchains with autonomous sidechains — subnets — that can provide *faster, cheaper, and more specialized* functionality. Subnet state is periodically anchored into higher-security chains through cryptographic checkpoints that serve as *objective, verifiable trust anchors*. Subnets are full, *decentralized* blockchains that *own their data*, are highly *configurable*, and impose *minimal requirements* on the chains they anchor into. Secured by checkpoints, cross-net messages enable *transfer of assets and information* between chains. The architecture is *recursive*: subnets can themselves be extended with lower-level subnets, with transitive checkpointing enabling *eventual consistency* across them.

</details>

Inter-Planetary Consensus (IPC) is a flexible blockchain **extensibility** and **scalability** solution. In a nutshell, it can be thought of as a **sidechain system** in which users can delegate operations to dedicated *autonomous* blockchains, subnets — typically faster and cheaper — whose state is anchored back into higher-security but less capable chains. Moreover, IPC subnets can be further extended with lower-level subnets, in a **recursive** way.

IPC subnets are meant to be full, sufficiently **decentralized** blockchains **owning their data**, unlike rollups, which still largely rely on centralized sequencers and post significant amount of data to the main chain. Subnets can provide faster finality, higher throughput, cheaper block space, and can host specialized functionality beyond what the parent chain supports. IPC subnets are highly **configurable**; they can be heavily **customized**, **specialized**, and **fine-tuned**. Moreover, IPC strives to impose **minimal requirements** on the target chain's capabilities; in fact, [IPC subnets can even be anchored into Bitcoin](https://arxiv.org/abs/2512.23439), despite its quite limited support for on-chain programmability. These capabilities enable building layered sidechain systems in a **recursive** manner, where lower-level, even more capable or specialized subnets can be anchored into higher-level subnets. Anchoring the subnet state into higher-security chains provides **checkpoint-based objectivity**, which facilitates dynamic joining into subnets or instantiating light clients for them, secured against [long-range attacks](https://ethereum.org/en/developers/docs/consensus-mechanisms/pos/attack-and-defense/#long-range) without relying on social consensus, as opposed to the [weak subjectivity](https://ethereum.org/en/developers/docs/consensus-mechanisms/pos/weak-subjectivity/) approach. IPC chains can **exchange assets and information** through cross-net messages, secured by the checkpointing mechanism. In addition to that, transitively checkpointing into a common chain helps establish **eventual consistency** across autonomous subnets, further facilitating coordination of cross-chain operations.

## Core Concepts

The main concept in IPC is **subnet**, which represents an autonomous blockchain whose state is anchored into another blockchain referred to as the subnet's **parent chain**. Subnets can act as parent chains for other subnets, in a recursive manner. A blockchain that has no parent chain is referred to as **rootnet**. Each subnet in IPC is assigned a **subnet ID**, which acts as an address identifying the subnet as a whole.

A subnet's state is anchored into the parent chain by periodically committing **checkpoints** to the parent chain. Each newly created checkpoint contains a cryptographic reference (e.g. block hash) to a recently finalized head of the subnet's chain and may also contain further cryptographic commitments to the subnet's state. Committed checkpoints thus cryptographically bind the subnet state to the parent chain and serve as an objective trust anchor for determining a recent subnet state. Similarly, subnets also periodically checkpoint their parent chain's finalized state, enabling secure cross-chain bridging in both directions.

Autonomous blockchains in IPC can interact by sending each other **cross-net messages**, which can carry arbitrary information as well as assets. Messages from the parent chain to the subnet are called **top-down messages**; messages from the subnet to the parent chain are called **bottom-up messages**. Messages carrying assets are also referred to as **deposits** (when locking & minting assets, typically top-down) or **withdrawals** (when burning & releasing assets, typically bottom-up).

*Note that, currently, sending cross-net messages is only supported for directly linked chains, i.e. single-hop parent-to-subnet or subnet-to-parent. Moreover, only deposits from and withdrawals back to parent chains are currently supported.*

## How IPC Works

On a conceptual level, the IPC protocol can be thought of as a combination of [mechanisms](core-mechanisms.md) working together. The checkpointing and bridging mechanisms play a central role in IPC: [**checkpointing**](core-mechanisms.md#checkpointing) ensures that the state of one chain remains anchored into another, whereas [**bridging**](core-mechanisms.md#bridging) secures the transfer of information and assets between chains. The subnet ID, lifecycle, and configuration management are auxiliary governance mechanisms: [**subnet ID management**](core-mechanisms.md#subnet-id-management) governs subnet ID namespaces and assignment of individual subnet IDs; [**subnet lifecycle management**](core-mechanisms.md#subnet-lifecycle-management) governs registration of new subnets, changing their operational status, and eventual termination of subnets; [**subnet configuration management**](core-mechanisms.md#subnet-configuration-management) regulates the process of updating the subnet configuration, such as its validator set and the allocation of voting power.

**Rootnet and Network Extension.** As a prerequisite, IPC assumes a preexisting blockchain network, a rootnet, which all participants have access to and treat as a root of trust. Such rootnet is supposed to be highly decentralized and secure, albeit slow and expensive. Given this basis, the IPC network can be progressively extended with new subnets.

**Subnet Lifecycle Flow.** Each new subnet first needs to obtain a subnet ID through the subnet ID management mechanism and then get registered through the lifecycle management mechanism. Depending on the subnet type and effective policies, the subnet lifecycle management mechanism may require further steps for the subnet to become active, e.g. locking initial subnet validators' collateral for PoS subnets. The activation determines the subnet's initial configuration, which is then handled by and may be updated through the subnet configuration management mechanism. Eventually, the subnet may be terminated, which is, like subnet activation, governed by the subnet lifecycle management mechanism.

**Checkpointing and State Anchoring.** For the most part, individual subnets operate as autonomous blockchains. However, in order to maintain objective trust anchors to subnets, the current state of subnets is cryptographically bound to their parent chains by means of checkpoints that are periodically committed there through the checkpointing mechanism. Conversely, subnets also periodically commit checkpoints of their parent chain's finalized state. Each checkpoint carries a cryptographic commitment (e.g. block hash) to a recently finalized subnet chain's head and may also contain further succinct cryptographic commitments to specific pieces of the subnet's state (e.g. the subnet's configuration, sub-subnet's checkpoints, etc.) The checkpointing mechanism ensures that the checkpoints committed on the parent chain objectively correspond to the subnet's actual state. Moreover, subnets follow their parent chains and also regularly commit checkpoints of recently finalized parent chain state. These checkpoints are used for cross-chain bridging, namely to secure transferring of information and assets from subnets to their parent chains and vice versa. Checkpoints also allow instantly joining and following the corresponding chain, without relying on more subjective trust anchors like social consensus or centralized services.

**Bridging and Cross-Net Messaging.** The bridging mechanism allows secure transfer of information and assets from one chain into another. This mechanism relies on sending chains continuously checkpointing into receiving chains. Both information and asset transfer use the same underlying carrier — cross-net messages. Cross-net messages can be used for depositing and withdrawal of assets as well as for pure general message passing. Depositing of assets from one chain into another is achieved, as usual, by locking assets on the origin chain and minting equivalent (wrapped) assets on the target chain; withdrawing deposited assets back to the origin chain is achieved by burning some of the previously minted assets on the target chain and releasing equivalent locked assets on the origin chain. General messages with information payload can trigger arbitrary on-chain logic.

## Key Capabilities & Properties

The IPC protocol provides the following key capabilities and properties.

<details>
<summary><strong>Dynamic scalability</strong></summary>

IPC enables scaling by allowing users to delegate operations from secure but slow and expensive chains to subnets optimized for performance, which can provide faster finality, higher throughput, and cheaper block space. Checkpoint-based objectivity through periodic state anchoring enables trustless bootstrapping: new subnets are immediately verifiable against their parent chain, so participants can join subnets without relying on trusted sources or social consensus. Thus, new subnets can be launched programmatically to handle increased workload and terminated when no longer needed, allowing the network to elastically adapt to varying demands. Scaling extends in multiple dimensions through parallel sibling subnets and recursive lower-level subnets.

</details>

<details>
<summary><strong>Customizability</strong></summary>

IPC subnets are meant to be highly configurable, allowing builders to tailor virtually every aspect of the blockchain stack to their specific requirements. This includes the consensus mechanism (e.g. enabling fast finality or other performance fine-tuning), the execution environment (supporting custom VM runtimes, e.g. [Wasm](https://webassembly.org/) & [EVM](https://ethereum.org/en/developers/docs/evm/)-compatible [FVM](https://docs.filecoin.io/smart-contracts/fundamentals/the-fvm)), tokenomics and fee structures (e.g. enabling subsidized or fee-free transactions), data availability strategies, and governance models. Subnets can be configured as permissioned (federated) or permissionless (open, with collateralized validators), and can enforce custom policies for validator admission and power allocation. This flexibility extends to the network topology itself, enabling regional or application-specific deployments. Subnets can also be extended with specialized functionality such as decentralized data storage, or compute-over-data workflows that can leverage faster subnet finality and greater computational capacity.

</details>

<details>
<summary><strong>Seamless interoperability</strong></summary>

IPC provides native cross-chain communication through protocol-native bridging secured by the checkpointing mechanism. Cross-net messages support general message passing, as well as asset deposits and withdrawals. Subnets that transitively checkpoint into a common chain establish eventual consistency with each other, facilitating coordination of cross-chain operations. IPC's minimal requirements on parent chain capabilities enable interoperability across heterogeneous environments, including chains with limited programmability. Furthermore, subnets' high customizability allows tailoring them for tighter integration with specific ecosystems, such as leveraging [IPLD](https://ipld.io/) for seamless data exchange with [Filecoin](https://docs.filecoin.io/).

</details>

<details>
<summary><strong>Autonomy &amp; sovereignty</strong></summary>

IPC subnets operating as largely autonomous blockchains own their data and control their own governance. Subnets govern their own lifecycle, configuration, and validator sets through dedicated management mechanisms, without relying on centralized components like sequencers or posting significant amounts of data to parent chains. Subnets can achieve the level of decentralization appropriate to their use case, from small federated deployments to fully open, large validator sets. Subnet autonomy includes partition tolerance: subnets can continue operating and achieving local finality even when temporarily disconnected from their parent chains, with consistency restored upon reconnection.

</details>

<details>
<summary><strong>Objective security</strong></summary>

IPC provides checkpoint-based objectivity by periodically anchoring subnet state into parent chains. Checkpoints serve as objective trust anchors that anyone can verify without relying on social consensus or trusted third parties. This secures against long-range attacks: an attacker cannot rewrite history beyond the last committed checkpoint without also compromising the parent chain. New participants can instantly join or follow subnets by obtaining a recent checkpoint on a parent chain, facilitating trustless light clients and dynamic validator set extensions without weak subjectivity.

</details>

<details>
<summary><strong>Risk containment</strong></summary>

IPC creates natural security boundaries between subnets. Failures, attacks, or misbehavior within one subnet are confined to that subnet's security domain and do not propagate to sibling subnets or compromise the integrity of parent chains. This limits the scope of potential incidents to the assets and operations entrusted to the affected subnet, preventing failures from cascading throughout the network.

</details>
