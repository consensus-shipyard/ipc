const hre = require('hardhat')

async function main() {
    const gateway = process.env.GATEWAY
    const USDCaddress = process.env.USDC
    const subnetTokenBridgeAddress = process.env.SubnetTokenBridge
    const subnetAddress = process.env.SUBNETADDRESS
    const accountAddress = await getAccountAddress()

    // Validate environment variables
    if (
        !gateway ||
        !USDCaddress ||
        !subnetTokenBridgeAddress ||
        !subnetAddress
    ) {
        throw new Error('All required environment variables must be provided')
    }

    parentSubnetChainId = 314159

    // Parent SubnetID value
    const parentSubnet = [parentSubnetChainId, []]

    // Child SubnetID value
    const subnetID = [parentSubnetChainId, [subnetAddress]]

    const rootnetTokenBridge = await deployRootnetTokenBridge(
        gateway,
        USDCaddress,
        subnetID,
        subnetTokenBridgeAddress,
    )

    const receiverAddress = accountAddress // choose to mint proxy tokens to some address on the subnet
    const transferAmount = hre.ethers.utils.parseUnits('500', 18) // Amount of tokens to transfer and mint

    // Define the DEFAULT_CROSS_MSG_FEE
    const DEFAULT_CROSS_MSG_FEE = hre.ethers.utils.parseUnits('10', 'gwei')

    // Approve the RootnetTokenBridge contract to spend tokens on behalf of the deployer
    await erc20Token.approve(rootnetTokenBridge.address, transferAmount)
    await rootnetTokenBridge.depositToken(receiverAddress, transferAmount, {
        value: DEFAULT_CROSS_MSG_FEE,
    })

    console.log(
        `Transfer and mint request made for ${transferAmount} tokens to ${receiverAddress}`,
    )
}

async function createSubnetTokenBridge(
    gateway,
    parentSubnetUSDC,
    parentSubnet,
) {
    const SubnetTokenBridge = await ethers.getContractFactory(
        'SubnetTokenBridge',
    )
    const subnetTokenBridge = await SubnetTokenBridge.deploy(
        gateway,
        parentSubnetUSDC,
        parentSubnet,
    )
    console.log('SubnetTokenBridge deployed to:', subnetTokenBridge.address)
    return subnetTokenBridge
}

async function deployRootnetTokenBridge(
    gateway,
    erc20Token,
    subnetID,
    subnetTokenBridge,
) {
    // Getting the contract factory for RootnetTokenBridge
    const RootnetTokenBridge = await hre.ethers.getContractFactory(
        'RootnetTokenBridge',
    )
    // Deploying RootnetTokenBridge with the new ERC20 token as sourceContract
    const rootnetTokenBridge = await RootnetTokenBridge.deploy(
        gateway,
        erc20Token,
        subnetID,
        subnetTokenBridge,
    )

    await rootnetTokenBridge.deployed()

    console.log('RootnetTokenBridge deployed to:', rootnetTokenBridge.address)
    return rootnetTokenBridge
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
