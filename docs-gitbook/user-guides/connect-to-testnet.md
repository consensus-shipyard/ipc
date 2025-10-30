---
description: Learn how to connect to the IPC Chain Testnet using MetaMask to interact with the IPC ecosystem.
---

# Connecting to IPC Chain Testnet

The IPC Chain Testnet (also known as "IPC Cal") is a public testnet where you can experiment with IPC features, deploy smart contracts, and test your applications before moving to production.

## Network Details

The IPC Chain Testnet has the following parameters:

* **Network Name**: IPC Cal
* **RPC URL**: `http://node-1.test.ipc.space:8545`
* **Chain ID**: `385401`
* **Currency Symbol**: `tFIL`
* **Block Explorer**: _Coming soon_

## Adding IPC Chain Testnet to MetaMask

Follow these steps to add the IPC Chain Testnet to your MetaMask wallet:

### Step 1: Open MetaMask Network Settings

1. Open your MetaMask extension
2. Click on the network dropdown at the top of MetaMask
3. Click "Add network" or "Add a network manually"

### Step 2: Enter Network Details

Enter the following information in the network configuration form:

![IPC Chain Testnet Configuration](.gitbook/assets/ipc-testnet-metamask.png)

* **Network name**: `IPC Cal`
* **New RPC URL**: `http://node-1.test.ipc.space:8545`
* **Chain ID**: `385401`
* **Currency symbol**: `tFIL`
* **Block explorer URL**: _(Leave empty for now)_

### Step 3: Save and Switch

1. Click "Save" to add the network
2. MetaMask will automatically switch to the IPC Chain Testnet
3. You should now see "IPC Cal" as your active network

## Getting Test Tokens

To interact with the IPC Chain Testnet, you'll need test tFIL tokens.

{% hint style="info" %}
A testnet faucet will be made available soon. In the meantime, please reach out to the IPC team for test tokens.
{% endhint %}

## Verifying Your Connection

Once connected, you can verify your connection by:

1. Checking that your MetaMask shows "IPC Cal" as the active network
2. Your account balance should display in tFIL
3. Try sending a small test transaction to confirm the connection is working

## Next Steps

Now that you're connected to the IPC Chain Testnet, you can:

* Deploy smart contracts using Remix or Hardhat
* Interact with existing IPC subnet contracts
* Test cross-subnet communication features
* Explore IPC-specific functionality

For more information on deploying and interacting with subnets, see:

* [Deploy a subnet](../quickstarts/deploy-a-subnet.md)
* [Performing transactions in a subnet](performing-transactions-in-a-subnet.md)

## Troubleshooting

### Connection Issues

If you're having trouble connecting to the testnet:

* Ensure you've entered the RPC URL correctly: `http://node-1.test.ipc.space:8545`
* Check that the Chain ID is exactly `385401`
* Try refreshing MetaMask or restarting your browser

### Transaction Failures

If transactions are failing:

* Verify you have sufficient tFIL for gas fees
* Check the network status - the testnet may be undergoing maintenance
* Ensure you're connected to the correct network (IPC Cal)

For additional help, see the [Troubleshooting guide](../reference/troubleshooting.md) or reach out to the IPC community.

