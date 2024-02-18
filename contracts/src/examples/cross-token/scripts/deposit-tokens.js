const hre = require('hardhat')

async function main() {
    const gateway = process.env.GATEWAY
    const USDCaddress = process.env.USDC
    const subnetTokenBridgeAddress = process.env.SubnetTokenBridge
    const subnetAddress = process.env.SUBNETADDRESS
    const rootTokenBridge = process.env.RootTokenBridge
    const accountAddress = await getAccountAddress()

    const usdcToken = await ethers.getContractAt('USDCMock', USDCaddress)

    const RootTokenBridge = await ethers.getContractAt(
        'RootnetTokenBridge',
        rootTokenBridge,
    )

    const receiverAddress = accountAddress // choose to mint proxy tokens to some address on the subnet
    const transferAmount = hre.ethers.utils.parseUnits('500', 18) // Amount of tokens to transfer and mint

    // Define the DEFAULT_CROSS_MSG_FEE
    const DEFAULT_CROSS_MSG_FEE = hre.ethers.utils.parseUnits('10', 'gwei')

    // Approve the RootnetTokenBridge contract to spend tokens on behalf of the deployer
    await usdcToken.approve(rootTokenBridge, transferAmount)
    await RootTokenBridge.depositToken(receiverAddress, transferAmount, {
        value: DEFAULT_CROSS_MSG_FEE,
    })

    console.log(`${transferAmount} USDC tokens deposited to bridge contract`)
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
