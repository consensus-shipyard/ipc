# Deploy smart contract to Mycelium testnet

Before delving into this tutorial, you should have a [basic understanding of IPC](../) and [subnets](../key-concepts/subnets.md). You may want to familiarize yourself with the fundamentals of setting up an IPC subnet, in the [previous quickstart](deploy-a-subnet.md).&#x20;

In this tutorial, we will guide you through the simple process of connecting to the Mycelium Calibration testnet and deploying smart contracts on it.  We will use the ERC20 contract as an example to illustrate the steps, including:

* Connect to the Mycelium Calibration
* Fund your address on the Mycelium Calibration
* Deploy an ERC20 contract on the Mycelium Calibration

## Prerequisites

* [Node](https://nodejs.org/en)
* [Rust](https://www.rust-lang.org/tools/install)
* [VS Code](https://code.visualstudio.com/download)
* [Docker](https://www.docker.com/products/docker-desktop/)
* [MetaMask](https://metamask.io/download/)

## Connect to the Mycelium Calibration

The Filecoin Calibration testnet will be the IPC rootnet to host multiple IPC subnets for testing, each offering distinct features and capacities. Builders have the flexibility to select a subnet that suits their specific requirements for deploying smart contracts. Alternatively, if builders have unique needs, they can set up their own subnet. If you are interested in learning more about IPC rootnet and subnets, you can read more [here](../key-concepts/subnets.md).&#x20;

In this quickstart guide, we will utilize the **public subnet** - **the Mycelium Calibration**, connected to Filecoin Calibration testnet, to deploy and interact with smart contracts.

To begin, the first step is to establish a connection to the Mycelium Calibration, enabling you to seamlessly request tFIL and interact with it.

### Add the subnet to your MetaMask wallet (manually)

Since IPC subnets are EVM-compatible, you can leverage various tools from the Ethereum ecosystem to build and interact with your decentralized applications (dApps).&#x20;

In this step, we will guide you through the process of manually configuring MetaMask to connect to our subnet. By doing so, we can manage tokens and interact with dApps deployed on this specific subnet using MetaMask.

#### **1. Getting RPC URL & Chain ID**

To connect MetaMak to Mycelium Calibration, we need to have its  RPC URL and `chainID` which you can find on [Reference/Networks](../reference/networks.md#mycelium-calibration-network) page.

```bash
# Mycelium Calibration Info
rpc url: "https://api.mycelium.calibration.node.glif.io/"
chainID: "3875628474663538"
```

With the gathered information, you now have all the necessary details to manually add your subnet network to MetaMask.

<figure><img src="../.gitbook/assets/metamask (2).png" alt=""><figcaption><p>Add Canopy network to the MetaMask.</p></figcaption></figure>

## Fund your address on the Mycelium Calibration

Since any IPC subnet operates as a layer2 network on top of Filecoin, it is necessary to transfer some tFIL tokens from the Filecoin Calibration testnet (rootnet) to our wallet within the Mycelium Calibration. This ensures that we have an adequate token balance for performing actions on the subnet.&#x20;

To facilitate this transfer, we will directly request some tFIL on [public faucet](https://faucet.mycelium.calibration.node.glif.io/) for Mycelium Calibration.  Use your wallet address on MetaMask to request tFIL on the faucet, it will send you 30 tFIL which was funded from Calibration.&#x20;

Once the tFIL is confirmed and transferred to the wallet address on Mycelium, we can transfer tokens within that Mycelium very quickly.

There are a couple of ways to check the token balance in the IPC subnet.

*   **On MetaMask**

    ![](../.gitbook/assets/tokenBalances.png)
*   **ETH API**

    We can also send RPC API request to the Mycelium Calibration node to query the wallet balance of a certain wallet address.

    {% code overflow="wrap" %}
    ```sh
    // Request wallet balance
    curl -X POST -H 'Content-Type: application/json' --data '{"jsonrpc":"2.0","method":"eth_getBalance","params":["0xd388aB098ed3E84c0D808776440B48F685198498", "latest"],"id":1}' <http://127.0.0.1:8545>
    ```
    {% endcode %}
*   **`ipc-cli` command**

    Use the `subnet-id` for the Mycelium Calibration.

    ```sh
    ipc-cli wallet balances --subnet /r314159/t410f3psqt4olacthtwmksoxapz2r2hbsohs6kxe636y -w evm
    ```

After acquiring some test tokens in the MetaMask wallet, we can begin working on the smart contract for the IPC subnet.

## Deploy ERC20 contract on the Mycelium Calibration&#x20;

As mentioned earlier, IPC subnets are EVM-compatible, allowing us to utilize various tools and frameworks that support Solidity development for building and deploying smart contracts on the IPC subnets. Let's take Remix and hardhat as examples for developing an ERC20 token contract on the Mycelium Calibration.

{% tabs %}
{% tab title="Remix" %}
We will use Remix & MetaMask for this step. So ensure your MetaMask connects to the Mycelium Calibration & loaded with some tFIL.

#### **1. Create a new workspace** on Remix

Let's go to the [Remix](https://remix.ethereum.org) website and create a new workspace. We will use the ERC20 template from [OpenZeppelin](https://docs.openzeppelin.com/contracts/5.x/erc20) and add a mintable feature to customize the contract.

<figure><img src="../.gitbook/assets/create_workspace (1).png" alt=""><figcaption><p>Create a new workspace</p></figcaption></figure>

Remix will generate a standard Solidity project structure, including an ERC20 token contract template and the necessary libraries from OpenZeppelin.

#### **2. Customize your token contract with the name and symbol**

On the left file explorer section on Remix, open `contracts/MyToken.sol` and modify the name and symbol for the ERC20 token.

<figure><img src="../.gitbook/assets/token (1).png" alt=""><figcaption><p>Customize the token details</p></figcaption></figure>

#### **3. Compile your token contract**

Set the Solidity compiler version to 0.8.20 on the Solidity Compiler page. This will automatically trigger the compilation of your project whenever you make any changes to the smart contract. If the compilation process is successful without any errors, you can proceed to deploy your token contract.

#### **4. Deploy contract to IPC subnet**

In this step, we will utilize MetaMask to sign and send deployment transactions to the Mycelium subnet. Ensure that MetaMask is connected to the Mycelium subnet, and selected `Injected Provider - MetaMask` as the deployment environment in Remix.

<figure><img src="../.gitbook/assets/InjectMM (1).png" alt=""><figcaption><p>Use MetaMask to sign the transaction</p></figcaption></figure>

Set your wallet address (copy from MetaMask) as the initial owner of this ERC20 token when deploying it. Review and confirm the deployment transaction on the MetaMask pop-up window after clicking the **Deploy** button. Once confirmed, the ERC20 token contract will be deployed on the Mycelium Calibration.

<figure><img src="../.gitbook/assets/deployed.png" alt=""><figcaption><p>Deploy the smart contract</p></figcaption></figure>

#### **5. Invoke smart contract on Remix**

After successfully deploying your contract to the Mycelium Calibration, you will be able to see your contract listed in the **Deployed Contracts** section on the left side of Remix. Expand the deployed contract, all the contract methods will be listed for us to try. Let’s try to call the mint function to mint ERC20 tokens in your wallet. We need to specify:

* **to**: the address to receive the minted ERC20 token
* **amount**: the amount of tokens to be minted.

<figure><img src="../.gitbook/assets/invoke (1).png" alt=""><figcaption><p>Invoke smart contract</p></figcaption></figure>

After the transaction is confirmed on the Mycelium Calibration, we will be able to call `balanceOf` to check if the tokens have been successfully minted to our wallet address.
{% endtab %}

{% tab title="HardHat" %}
In addition to using the Remix UI, there are more programmable approaches to develop smart contracts using frameworks like [hardhat](https://hardhat.org/) and [foundry](https://github.com/foundry-rs/foundry). Let's take hardhat as an example to develop and deploy a basic ERC20 token on the Mycelium Calibration.

Before moving forward, ensure we have the following dependencies installed on the machine.

* Node
* MetaMask connects to the IPC subnet & loaded with some tFIL

Then let’s get started.

#### **1. Install & initialize a hardhat project**

To install hardhat, we need to first create an npm project where we can config and install all dependencies.

```sh
mkdir erc20-example
cd erc20-example
npm init
```

After creating the project, let's install hardhat in this project and initialize a hardhat project structure in Javascript.

```sh
npm install --save-dev hardhat

npx hardhat init
```

#### **2. Config hardhat to connect to** the Mycelium Calibration

Now we should have a hardhat JavaScript project with a basic project structure where we can find a `hardhat.config.js` file with all the configurations for this hardhat project.

Considering the security of your project, we will use the `.evn` file to store sensitive information, like wallet private key and smart contract address. Create a `.env` file under your project, add the following code in there, and replace the `<your-wallet-private-key>` with the private key that we can export from your MetaMask wallet.

```jsx
PRIVATE_KEY="<your-wallet-private-key>"
```

Open `hardhat.config.js` with VsCode, we will add IPC network configuration in this file. Make sure you have installed the `dotenv` package in your project by running `npm install dotenv`. Next, let's retrieve the ChainId and URL for the Mycelium Calibration from the [previous step](deploy-smart-contract-to-mycelium-testnet.md#1.-getting-rpc-url-and-chain-id). We will use them to configure the IPC network.

In the `hardhat.config.js` file, add the following code.

```javascript
require("@nomicfoundation/hardhat-toolbox");
require('dotenv').config();
const WalletPK = process.env.PRIVATE_KEY;

/** @type import('hardhat/config').HardhatUserConfig */
module.exports = {
  solidity: "0.8.20", //Update solidity version for Openzeppline contracts
  networks: {
    Mycelium: {
        chainId: 1914449495539888,
        url: "https://api.mycelium.calibration.node.glif.io/",
        accounts: [WalletPK]
    }
  }
};
```

#### **3. Create an ERC20 contract.**

We will use a basic ERC20 example from Openzeppline for this tutorial, so let’s install Openzeppline first with the following command.

```sh
npm install @openzeppelin/contracts
```

After successfully installing OpenZeppelin, we can now utilize the ERC20 contract in our project by importing it directly. Create a file named `IPCERC20.sol` under the `contracts` folder, and add the following code to create a basic mintable ERC20 token contract.

```solidity
// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.20;

import "@openzeppelin/contracts/token/ERC20/ERC20.sol";
import "@openzeppelin/contracts/access/Ownable.sol";

contract IPCERC20 is ERC20, Ownable{
    constructor (address initialOwner)
        ERC20("My first IPC test token on Mycelium", "TT_IPC")
        Ownable(initialOwner)
    {}

    function mint(address to, uint256 amount) public onlyOwner {
        _mint(to, amount);
    }
}
```

#### **4. Compile & deploy your smart contract to the IPC subnet**

Then compile your smart contract with this command.

```jsx
npx hardhat compile

Compiled 8 Solidity files successfully (evm target: paris).
```

Now we are ready to deploy this ERC20 token to the Mycelium Calibration. We can write a deployment script  `script/deploy.js` as follows.

```jsx
const { ethers } = require('hardhat');
require('dotenv').config();
const WalletPK = process.env.PRIVATE_KEY;

async function main() {
  //Connect to the wallet to sign and send transaction
  const wallet = new ethers.Wallet(WalletPK, ethers.provider);
  console.log("Deploying contracts with the account:", wallet.address);
  console.log("Wallet balance is ", ethers.formatEther(await ethers.provider.getBalance(wallet.address)));

  //Get the contract instance and deploy it
  const contractFactory = await ethers.getContractFactory("IPCERC20",wallet);
  const deployedContract = await contractFactory.deploy(wallet.address);
  console.log("Deploy contract tx is sent.");
  await deployedContract.waitForDeployment();
  console.log('IPC ERC20 Token Contract deployed to ', await deployedContract.getAddress());
}

main().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});
```

Using the `hardhat run` command to run a specific deployment script to the Mycelium Calibration network configured in `hardhat.config.js` .

```jsx
npx hardhat run scripts/deploy.js --network Mycelium

Deploying contracts with the account: 0xd388aB098ed3E84c0D808776440B48F685198498
Wallet balance is  18.265091067491548
Deploy contract tx is sent.
IPC ERC20 Token Contract deployed to  0x435A9BDE7A04b1C91a41eAfEf3f7E84dC37a83C4
```

{% hint style="info" %}
<mark style="color:red;">💡</mark> You need to record your token contract address which will be used to interact with it programmatically.
{% endhint %}

#### **5. Interact with your contract**

After deploying the ERC20 token to the Mycelium Calibration and receiving the contract address, we will add it to the `.env` file so that we can directly access it in the Hardhat project.

Open the `.env` file and add the following line, replacing `<contract-address>` with the actual deployed contract address.

```jsx
CONTRACT_ADDRESSS=<replace-with-your-ERC20-contract-addr>
```

So let’s create a new file `scripts/invokeToken.js` in the scripts folder, and then we will write code to:

1. get your wallet account to sign the transaction and pay for the GAS fee
2. get a contract instance for the ERC20 token
3. call the ERC20 contract to get its name, symbol, and the balance for your account
4. mint some ERC20 tokens to your wallet address

```javascript
const { ethers } = require("hardhat")
require('dotenv').config();
const WalletPK = process.env.PRIVATE_KEY;
const ContractAddr = process.env.CONTRACT_ADDRESSS;

async function main() {
    //1. Get wallet account to sign the transaction and pay for the gas fee
    const wallet = new ethers.Wallet(WalletPK, ethers.provider)

    //2. get a contract instance for ERC20 token
    const factory = await ethers.getContractFactory("IPCERC20", wallet);
    const myTokenContract = factory.attach(ContractAddr);

    //3. call the ERC20 contract to get its name, symbol, and the balance for your account
    console.log("Token name is ", await myTokenContract.name());
    console.log("Token symbol is ", await myTokenContract.symbol());
    console.log("My token balance is ", ethers.formatUnits(await myTokenContract.balanceOf(wallet.address)));
		
    //4. mint some ERC20 tokens to your wallet address
    const mintTX = await myTokenContract.mint(wallet.address,ethers.parseUnits("100"));
    console.log(mintTX.hash);
    await mintTX.wait();
    console.log("My new token balance is ", ethers.formatUnits(await myTokenContract.balanceOf(wallet.address)));
}

main().catch((error) => {
    console.error(error);
    process.exitCode = 1;
});
```

Now, let's run the script to interact with your deployed ERC20 token on the Mycelium Calibration. This will help verify that the token contract is deployed and working correctly.

To run the script, execute the following command in your terminal.

```sh
npx hardhat run --network Mycelium scripts/invokeToken.js

Token name is  My first IPC test token
Token symbol is  IPCTT
My balance is  0.0
0xf41b192bbefa2777a1c0984f4d12a32b3e213f94ba1045309f38dd5fa458b0e3
My new balance is  100.0
```
{% endtab %}
{% endtabs %}

Congratulations! You have successfully deployed your first ERC20 contract on the Mycelium Calibration and even interacted with it. You can view your contract on the [explorer](https://explorer.mycelium.calibration.node.glif.io/).\
\
Now, it's time to dive deeper and explore the exciting array of features available on the IPC subnet.
