const hre = require('hardhat')

async function main() {
    const gateway = process.env.GATEWAY
    const accountAddress = await getAccountAddress()

    // Validate environment variables
    if (!gateway) {
        throw new Error('All required environment variables must be provided')
    }

    parentSubnetChainId = 1337

    // Parent SubnetID value
    const parentSubnet = [parentSubnetChainId, []]

    // Deploy ERC20 token
    const ERC20 = await hre.ethers.getContractFactory('USDCTest')
    const erc20Token = await ERC20.deploy();
    await erc20Token.deployed()

    const ipcTokenReplica = await createIpcTokenReplica(
        gateway,
        erc20Token.address,
        parentSubnet,
    )
    // Child SubnetID value
    const subnetID = [parentSubnetChainId, [ipcTokenReplica.address]]

    // Mint tokens
    const mintAmount = hre.ethers.utils.parseUnits('1000', 18) // 1000 tokens
    const transferAmount = hre.ethers.utils.parseUnits('100', 18) // Amount of tokens to transfer and mint

    await erc20Token.mint(mintAmount)

    console.log('ERC20 Token deployed to:', erc20Token.address)
    const ipcTokenController = await deployIpcTokenController(
        gateway,
        erc20Token,
        subnetID,
        ipcTokenReplica,
    )

    //initialize the Token Replica with the Token Controller address.
    await ipcTokenReplica.setController(ipcTokenController.address);

    const receiverAddress = accountAddress // choose to mint proxy tokens to some address on the subnet

    // Define the DEFAULT_CROSS_MSG_FEE
    const DEFAULT_CROSS_MSG_FEE = hre.ethers.utils.parseUnits('10', 'gwei')

    // Approve the IpcTokenController contract to spend tokens on behalf of the deployer
    await erc20Token.approve(ipcTokenController.address, transferAmount)
    await ipcTokenController.lockAndTransfer(receiverAddress, transferAmount, {
        value: DEFAULT_CROSS_MSG_FEE,
    })

    //confirm balances

    const userTokenBalance = await erc20Token.balanceOf(accountAddress)
    console.log(`USDC Token balance of user on controller chain is: ${userTokenBalance}`);
    const controllerTokenBalance = await erc20Token.balanceOf(ipcTokenController.address)
    console.log(`USDC Token balance of controller chain is: ${controllerTokenBalance}`);

    //assert values are correct
    if (userTokenBalance != mintAmount-transferAmount) throw new Error(`User Token Balance incorrect`)
    if (controllerTokenBalance != transferAmount-0) throw new Error(`Controller Token Balance incorrect`)

    console.log(
        `Transfer and mint request made for ${transferAmount} tokens to ${receiverAddress}`,
    )

    console.log(`Simulate call from Gateway to token replica contract`)
    await ipcTokenReplica.mintOnlyOwner(
        accountAddress,
        transferAmount,
    )

    //confirm balances

    let userReplicaTokenBalance = await ipcTokenReplica.balanceOf(accountAddress)
    console.log(`USDC Replica Token balance of user on token replica chain is: ${userReplicaTokenBalance}`)
    if (userReplicaTokenBalance != transferAmount-0) throw new Error(`User Replica Token Balance incorrect`)


    //transfer back to controller
    await ipcTokenReplica.burnAndTransfer(accountAddress, transferAmount, {
        value: DEFAULT_CROSS_MSG_FEE,
    })


    userReplicaTokenBalance = await ipcTokenReplica.balanceOf(accountAddress)
    console.log(`USDC Replica Token balance of user on token replica chain is: ${userReplicaTokenBalance}`);
    if (userReplicaTokenBalance != 0) throw new Error(`User Replica Token Balance incorrect`)

    // simulate xnetmessage on parent net to release original tokens back to the account
    await ipcTokenController.receiveAndUnlockOnlyOwner(
        accountAddress,
        transferAmount,
    )

    // verify that account currently has correct number of original tokens and 0 subnet tokens

    const userBalance = await erc20Token.balanceOf(accountAddress)
    console.log(`Final User Token balance on parent chain: ${userBalance}`);

    const tokenControllerFinalBalance = await erc20Token.balanceOf(ipcTokenController.address)
    console.log(
        `Final USDC Token balance on subnet chain: ${tokenControllerFinalBalance}`
    )

    if (tokenControllerFinalBalance != 0) throw new Error(`Token Controller Final Token Balance incorrect`)
    if (userBalance - mintAmount != 0 ) throw new Error(`User Final Token Balance incorrect`)
}

async function createIpcTokenReplica(
    gateway,
    parentSubnetUSDC,
    parentSubnet,
) {
    const IpcTokenReplica = await ethers.getContractFactory(
        'TestIpcTokenReplica',
        {"libraries" : {"SubnetIDHelper":"0x033b910e8a8f3365B69c84852009c637bA34eE83"}}
    )
    const ipcTokenReplica = await IpcTokenReplica.deploy(
        gateway,
        parentSubnetUSDC,
        parentSubnet,
    )
    console.log('IpcTokenReplica deployed to:', ipcTokenReplica.address)
    return ipcTokenReplica
}

async function deployIpcTokenController(
    gateway,
    erc20Token,
    subnetID,
    ipcTokenReplica,
) {
    // Getting the contract factory for IpcTokenController
    const IpcTokenController = await hre.ethers.getContractFactory(
        'TestIpcTokenController',
        {"libraries" : {"SubnetIDHelper":"0x033b910e8a8f3365B69c84852009c637bA34eE83"}}
    )
    // Deploying IpcTokenController with the new ERC20 token as sourceContract
    const ipcTokenController = await IpcTokenController.deploy(
        gateway,
        erc20Token.address,
        subnetID,
        ipcTokenReplica.address,
    )

    await ipcTokenController.deployed()

    console.log('IpcTokenController deployed to:', ipcTokenController.address)
    return ipcTokenController
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
