# 🟢 Connecting to a subnet

## Connect to the Mycelium Calibration

The Filecoin Calibration testnet will be the IPC rootnet to host multiple IPC subnets for testing, each offering distinct features and capacities. Builders have the flexibility to select a subnet that suits their specific requirements for deploying smart contracts. Alternatively, if builders have unique needs, they can set up their own subnet. If you are interested in learning more about IPC rootnet and subnets, you can read more [here](broken-reference).

In this quickstart guide, we will utilize the **public subnet** - **the Mycelium Calibration**, connected to Filecoin Calibration testnet, to deploy and interact with smart contracts.

To begin, the first step is to establish a connection to the Mycelium Calibration, enabling you to seamlessly request tFIL and interact with it.

### Add the subnet to your MetaMask wallet (manually)

Since IPC subnets are EVM-compatible, you can leverage various tools from the Ethereum ecosystem to build and interact with your decentralized applications (dApps).

In this step, we will guide you through the process of manually configuring MetaMask to connect to our subnet. By doing so, we can manage tokens and interact with dApps deployed on this specific subnet using MetaMask.

#### **1. Getting RPC URL & Chain ID**

To connect MetaMak to Mycelium Calibration, we need to have its RPC URL and `chainID` which you can find on [Reference/Networks](../../reference/networks.md#mycelium-calibration-network) page.

```bash
# Mycelium Calibration Info
rpc url: "https://api.mycelium.calibration.node.glif.io/"
chainID: "2120099022966061"
```

With the gathered information, you now have all the necessary details to manually add your subnet network to MetaMask.

<figure><img src="../../.gitbook/assets/MM-network.png" alt=""><figcaption><p>Add Canopy network to the MetaMask.</p></figcaption></figure>

## Fund your address on the Mycelium Calibration

Since any IPC subnet operates as a layer2 network on top of Filecoin, it is necessary to transfer some tFIL tokens from the Filecoin Calibration testnet (rootnet) to our wallet within the Mycelium Calibration. This ensures that we have an adequate token balance for performing actions on the subnet.

To facilitate this transfer, we will directly request some tFIL on [public faucet](https://faucet.mycelium.calibration.node.glif.io/) for Mycelium Calibration. Use your wallet address on MetaMask to request tFIL on the faucet, it will send you 30 tFIL which was funded from Calibration.

Once the tFIL is confirmed and transferred to the wallet address on Mycelium, we can transfer tokens within that Mycelium very quickly.

There are a couple of ways to check the token balance in the IPC subnet.

*   **On MetaMask**

    <div align="left">

    <img src="../../.gitbook/assets/metamask (6).png" alt="Wallet Balance on MetaMask" width="371">

    </div>
*   **ETH API**

    We can also send RPC API request to the Mycelium Calibration node to query the wallet balance of a certain wallet address.

    {% code overflow="wrap" %}
    ```sh
    // Request wallet balance
    curl -X POST -H 'Content-Type: application/json' --data '{"jsonrpc":"2.0","method":"eth_getBalance","params":["<WALLET ADDRESS", "latest"],"id":1}' <https://api.mycelium.calibration.node.glif.io/>
    ```
    {% endcode %}
*   **`ipc-cli` command**

    Use the `subnet-id` for the Mycelium Calibration.

    ```sh
    ipc-cli wallet balances --subnet /r314159/t410fx23amesh6qvzfzl744uzdr76vlsysb6nnp3us4q --wallet-type evm
    ```

After acquiring some test tokens in the MetaMask wallet, we can begin working on the smart contract for the IPC subnet.
