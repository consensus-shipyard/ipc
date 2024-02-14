# 🟡 Introduction

With IPC, dApps can reach planetary scale through recursively scalable subnets, sub-second transactions, robust compute workloads, and highly adaptable WebAssembly runtimes tailored to developer requirements.

## What is IPC?

[Interplanetary Consensus (IPC)](https://www.ipc.space/) is a framework that enables on-demand horizontal scalability of networks, by deploying "subnets" running different consensus algorithms depending on the application's requirements.

Let's break that down.

### What is horizontal scalability and why is it important for dApps?

[Horizontal scalability](https://en.wikipedia.org/wiki/Scalability#Horizontal\_or\_scale\_out) generally refers to the addition of nodes to a system, to increase its performance. For example, adding more nodes to a compute network helps distribute the effort needed to run a single compute task. This reduces cost per task and decreases latency, while improving overall throughput.&#x20;

In web3, horizontal scalability refers to  _scaling_ blockchains, for _desired_ performance. More specifically, _scaling_ the ability of a blockchain to process transactions and achieve consensus, across an increasing number of users, at _desired_ latencies and throughput. IPC is one such scaling solution, alongside other popular layer 2 solutions, like [sidechains](https://ethereum.org/en/developers/docs/scaling/sidechains/) and [rollups](https://ethereum.org/en/developers/docs/scaling/#rollups).&#x20;

For decentralized applications (dApps), there are several key motivations to adopt scaling - performance, decentralization, security. The challenge is that these factors are known to be conflicting goals.&#x20;

### How does IPC achieve horizontal scalability?&#x20;

IPC is a scaling solution intentionally designed to achieve considerable performance, decentralization and security for dApps.

It achieves scaling through the permission-less spawning of new blockchain sub-systems, which are composed of subnets.&#x20;

[Subnets](broken-reference) are organized in a hierarchy, with one parent subnet being able to spawn infinite child subnets. Within a hierarchical subsystem, subnets can seamlessly communicate with each other, reducing the need for cross-chain bridges.

Subnets also have their own specific consensus algorithms, whilst leveraging security features from parent subnets. This allows dApps to use subnets for hosting sets of applications or to [shard](https://en.wikipedia.org/wiki/Shard\_\(database\_architecture\)) a single application, according to its various cost or performance needs. \
\
IPC-powered networks will also be able to dynamically adjust their throughput by spawning and closing temporary subnets as needed.

### How is IPC unique as a scaling solution?

Earlier, we talked about the challenge of scaling solutions to balance performance, security and decentralization. IPC is a standout framework that strikes a considerable balance between these factors, to achieve breakthroughs in scaling.  \


* **Highly customizable without compromising security.** Most L2 scaling solutions today either inherit the L1's security features but don't have their own consensus algorithms (e.g. rollups), or do the reverse (e.g. sidechains). They are also deployed in isolation and require custom bridges or protocols to transfer assets and state between L2s that share a common L1, which are vulnerable to attacks. In contrast, IPC subnets have their own consensus algorithms, inherit security features from the parent subnet and have native cross-net communication, eliminating the need for bridges.&#x20;
* **Multi-chain interoperability.** IPC uses the [Filecoin Virtual Machine (FVM)](https://docs.filecoin.io/smart-contracts/fundamentals/the-fvm) as its transaction execution layer. The FVM is a WASM-based polyglot execution environment for IPLD data and is designed to support smart contracts written in any programming language, compiled to WebAssembly. This enables multi-chain support and gives developers the flexibility to build with familiar tools. Today, IPC is fully compatible with Filecoin and Ethereum and can use either as a rootnet, with more multi-chain support in the roadmap.
* **Compute-Storage Interoperability with Filecoin and more.** IPC is designed to seamlessly integrate with Filecoin and EVM-compatible chains (with more to come), allowing developers to embed IPC subnets within these ecosystems. In particular, IPC unlocks new compute possibilities with the data-centric L1, [Filecoin](https://docs.filecoin.io/basics/what-is-filecoin), which is the largest decentralized storage network. IPC can leverage its storage primitives, like IPLD data integration, to deliver enhanced solutions for data availability and more.
* **Truly modular and flexible design.** IPC’s modular runtime enables the creation of truly flexible blockchains. Developers can run programs written in any language compilable to WebAssembly and leverage FVM. Soon, IPC-powered networks will also be able to dynamically adjust their throughput by spawning and closing temporary subnets as needed.

### How does IPC compare?
