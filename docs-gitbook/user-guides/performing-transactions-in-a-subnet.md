---
description: >-
  The below is a tutorial on how to issue a transaction from a wallet in a
  subnet.
---

# Performing Transactions from a Wallet in a Subnet

### Connecting to a subnet

Assuming the process of launching a custom IPC Subnet with at least one validator node is complete, the custom IPC Subnet is now available for a user (defined here as a wallet on the subnet that is does not necessarily represent a validator) to perform transactions.&#x20;



* A user would begin by launching Metamask, and manually adding the custom IPC Network to their Metamask networks list using “Add a network manually.” &#x20;

\
![](https://lh7-us.googleusercontent.com/ieHnkmUDN4HDPugxp9u5niCXfbmfYBRS7j7r-ptXaplbltPrH24hbg6juhQkywLfboqMNPykw\_qsfBPOOsTYW0OS8km9JApVd5sIie4oOVPDiEH9J8DM8garYaimT0tikqXDp0ro5Mp12JuHehN4zAw)

* Name the network, include the local New RPC URL and relevant ChainID (both of which are provided when you successfully launch the validator node), and name the currency symbol (for Filecoin subnets, tFIL is used). &#x20;

\
![](https://lh7-us.googleusercontent.com/nNdSkpeWuT-a\_4nw0VyAb4SI8nmgOt2fZ5eHKXLj9B4UBW2j0g6TG0uo3G5BHLcNqR78a7NL7gAlcw9QsnpudEm7b2vrnqxDD1a2O1Ti-SrgGOBBfti6E\_da6U3U9thrKP-LUJ9aG0BFI8\_4xVg6bRE)

* Once, the network is added successfully, "Switch to IPC Local Subnet," or whatever you named your subnet.&#x20;

![](https://lh7-us.googleusercontent.com/lIs8jBuP5AmtJNVKuRg2hUp3IJ9tH1\_rSyI8TnLgH5z1gRzzrsl0IimqFqwH3oQOoe1sodRxxy\_XctYJWItilThgwFyOEh06hPKvP06q9n2NR5HtxVQJzQI7QHz2dTPZx31T60wYaAFLI6oeuFbS1Ls)

* Now, an account on this custom IPC Subnet can be imported to MetaMask.  Select “+ Add account or hardware wallet.”  &#x20;

\
![](https://lh7-us.googleusercontent.com/VgEXw2VhyuJvArbAk8h3gkJMYVvAfeZnVlScM8HWlvbjCPE6I9G8Yfm\_sEuEGjiDiRJMu1KMHj32QSJlNmLLy7WEF9toklgPs-Eph\_imTeevRgUwSQHNHGMFl4ez3pDtqwfeEgIiyElBxFufRzihNx8)

* Select "Import account". &#x20;

![](https://lh7-us.googleusercontent.com/rdc0c-ai86iKLyMRalvmfVENO00D8enHHR5tZ3xd7TT2wK9XlwfmC8wKJWgjmKbm8shV7MBwa-vRGAGu8CBAsK6aIPAxixS3zLYhJkbMKmWMbF84I4F1g3KcGL1-JRCC-VcmrL6q-PwbDOCj4\_r7iQw)

* Enter the private key for the account, which is generated when establishing the subnet, and select “import.”&#x20;

\
![](https://lh7-us.googleusercontent.com/Rn\_j2KHo3MT2BkvQV8tIsI1f2GqOup5DkqaPhgaijiQKqBmvKZT9lrKq7FnRc2n8h9WHXEvlGzHelPTfEBhOMA3n6\_YhiDK-ec7cy89\_iYWfHlDVlAqog91ewcicH6mXrKQAUXItAxwYqfLHWjpAeQ4)

Note: The private key can be retrieved on the command line by the user that deployed the subnet using the following command:&#x20;

```
ipc % cat ~/.ipc/validator_1.sk
```

The user now has a wallet address available for transactions in the custom IPC subnet.  &#x20;

### Deploying Smart Contracts in a Subnet

Consider the deployment of a simple smart contract that issues ERC20 tokens in the IPC subnet --these tokens will be acquired by a user's IPC subnet MetaMask wallet.   The steps for achieving this are as follows:&#x20;

* Open Remix and a simple contract .sol file for issuing ERC20 tokens. &#x20;

![](https://lh7-us.googleusercontent.com/U-LqFhgUHn8KefGNSFnGD3m8QHCrH\_-K8rAerxc93H5wEXpuoc4kvJdBstNsgNTddAH92q1tt1Cb4IsR3OEKoE8p5ocj0YHaX4cn4v06IUrcvXKoHKatCeL8y87dEP0OwpCV8siqgBXDMD7N9SDg62A)

* For the environment, connect the IPC Subnet using the “Injected provider - Metamask” custom network.  Connect the relevant account on the IPC Subnet.&#x20;

![](https://lh7-us.googleusercontent.com/39kI4JGa7CfV20X5xHk9AoGPvW-NZR\_95eKvaciYNwXO9v9-WBOKXqP41ZnGaKPvx9ssaBTB-jE0FUX6ikoHTXmzsTyjwvsulnE1krqV1vTd\_32a\_ZdjSpCrWKXfyYesxJGQqkGQhsYYL7blHLeMXGw)

* Select the relevant account and set the gas limit. &#x20;

![](https://lh7-us.googleusercontent.com/DlWKLXaEgbmRa3fgfZA8G2QCTPTGMOSJvVEv8Q\_g19VZS0vHkuSf02BP7I89H7uup3b5TzyW0y6lSLNpCZVBD2tV6YcdlwNELOlVJad2gvixcLP6uynxmbYaWV3ZKviVEdBwU7BD4bVLMDGYrG6E77g)

* Compile and deploy the contract.  Mint new tokens by specifying the address you want the tokens to go to, and the number of tokens you wish to mint.  Click “Transact.”

![](https://lh7-us.googleusercontent.com/XUrv2Iybr9t5MMX4jdLrRBa2wgadtwltUPz4fshnugTMoQvn4wNbnganReV8NFEk1jV9I-NQBw41MVRfU7o3UZ\_rYViza792R4jgDYnxien1FeLrT24ByTYhqtsHDDyoP1C7ujxsEwzGTgN792UtYd8)

* Copy the token address and import the token to the MetaMask wallet on the custom IPC subnet.&#x20;

![](https://lh7-us.googleusercontent.com/mkInwBaiKO8snT-n9h6KaLUTYmB--7EJz47Nm\_Z4HqvcZLX0e4b09eCtKvvCKQW8DwuQlD7BwC887eCw8YuMA2F2INPzFNY6U3mf2PZWMTWuIHjfjlDmRpGCnQjCAf84uFZe4\_LTCV-owYCNxdwRev0)

* The funds of the new token should be displayed, indicating a successful transaction was performed on the IPC subnet.

### Block Explorers

Currently, browsing a custom subnet using a block explorer is not possible.  However, a block explorer that can browse custom subnets should be released soon.   \
&#x20;
