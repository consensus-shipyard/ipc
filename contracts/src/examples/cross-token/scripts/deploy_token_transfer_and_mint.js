const hre = require('hardhat')

async function main() {
    const gateway = process.env.GATEWAY
    const accountAddress = await getAccountAddress()

    // Validate environment variables
    if (!gateway) {
        throw new Error('All required environment variables must be provided')
    }

    // Parent SubnetID value
    const parentSubnet = [0, [] ]


    // Child SubnetID value
    const subnetID = [1, [] ] 


    // Deploy ERC20 token
    const ERC20 = await hre.ethers.getContractFactory('USDCMock')
    const erc20Token = await ERC20.deploy('USDC', 'USDC')
    await erc20Token.deployed()

    const subnetTokenBridge = await createSubnetTokenBridge(gateway, erc20Token.address, parentSubnet);


    // Mint tokens
    const mintAmount = hre.ethers.utils.parseUnits('1000', 18) // 1000 tokens
    await erc20Token.mint(mintAmount)

    console.log('ERC20 Token deployed to:', erc20Token.address)
    const tokenTransferAndMint = await deployTokenTransferAndMint(
        gateway,
        erc20Token,
        subnetID,
        subnetTokenBridge,
    )

    const receiverAddress = accountAddress // choose to mint proxy tokens to some address on the subnet
    const transferAmount = hre.ethers.utils.parseUnits('500', 18) // Amount of tokens to transfer and mint

    // Define the DEFAULT_CROSS_MSG_FEE
    const DEFAULT_CROSS_MSG_FEE = hre.ethers.utils.parseUnits('10', 'gwei')

    // Approve the TokenTransferAndMint contract to spend tokens on behalf of the deployer
    await erc20Token.approve(tokenTransferAndMint.address, transferAmount)
    await tokenTransferAndMint.transferAndMint(
        receiverAddress,
        transferAmount,
        { value: DEFAULT_CROSS_MSG_FEE },
    )

    console.log(
        `Transfer and mint request made for ${transferAmount} tokens to ${receiverAddress}`,
    )

    console.log(`Simulate call to onXNetMessageReceived`)
    await subnetTokenBridge.onXNetMessageReceived(accountAddress, transferAmount)

    const proxyTokenAddress = await subnetTokenBridge.getProxyTokenAddress()
    const SubnetUSDCProxy = await ethers.getContractAt(
        'SubnetUSDCProxy',
        proxyTokenAddress,
    )
    const balance = await SubnetUSDCProxy.balanceOf(accountAddress)
    console.log('balance is ', balance)


    //transfer up subnets
    console.log(1)

    //Approve subnet contract 
    await SubnetUSDCProxy.approve(subnetTokenBridge.address, transferAmount)
    console.log(2)

    //transfer
    await subnetTokenBridge.depositTokens(accountAddress,transferAmount, { value: DEFAULT_CROSS_MSG_FEE });
    console.log(3)

    // todo

    // simulate xnetmessage on parent net to release original tokens back to the account
    await tokenTransferAndMint.onXNetMessageReceived(accountAddress, transferAmount);

    // verify that account currently has correct number of original tokens and 0 subnet tokens

    const finalBalance = await erc20Token.balanceOf(accountAddress);
    console.log("Final USDC Token balance on parent chain: ", finalBalance)

    const subnetFinalBalance = await SubnetUSDCProxy.balanceOf(accountAddress);
    console.log("Final USDC Token balance on subnet chain: ", subnetFinalBalance)


}

async function createSubnetTokenBridge(gateway, parentSubnetUSDC, parentSubnet) {
    const SubnetTokenBridge = await ethers.getContractFactory(
        'SubnetTokenBridge',
    )
    const subnetTokenBridge = await SubnetTokenBridge.deploy(gateway, parentSubnetUSDC, parentSubnet);
    console.log('SubnetTokenBridge deployed to:', subnetTokenBridge.address)
    return subnetTokenBridge
}

async function deployTokenTransferAndMint(
    gateway,
    erc20Token,
    subnetID,
    subnetTokenBridge,
) {
    // Getting the contract factory for TokenTransferAndMint
    const TokenTransferAndMint = await hre.ethers.getContractFactory(
        'TokenTransferAndMint',
    )
    // Deploying TokenTransferAndMint with the new ERC20 token as sourceContract
    const tokenTransferAndMint = await TokenTransferAndMint.deploy(
        gateway,
        erc20Token.address,
        subnetID,
        subnetTokenBridge.address,
    )

    await tokenTransferAndMint.deployed()

    console.log(
        'TokenTransferAndMint deployed to:',
        tokenTransferAndMint.address,
    )
    return tokenTransferAndMint
}

async function getAccountAddress() {
    // Getting a list of accounts
    const accounts = await hre.ethers.getSigners()

    // Assuming the first account is the one you want to use
    const currentAccount = accounts[0]

    // Getting the public address of the current account
    const publicAddress = currentAccount.address
    return publicAddress
}

main()
    .then(() => process.exit(0))
    .catch((error) => {
        console.error(error)
        process.exit(1)
    })
